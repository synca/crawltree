use crate::config::WebCrawlerConfig;
use crate::filter::{UrlFilter, UrlFilterConfig};
use crate::parsers::{self, ParserType};
use crate::results::PageData;
use fantoccini::{Client, ClientBuilder};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore, mpsc};
use tokio::time::timeout;
use url::Url;

/// Starts an async web crawl and returns a receiver that yields PageData as discovered.
///
/// # Arguments
///
/// * `config` - Web crawler configuration
pub async fn start(config: &WebCrawlerConfig) -> mpsc::Receiver<PageData> {
    ::log::info!("Starting web crawler for: {}", config.start_url);

    let root_url = Url::parse(&config.start_url).expect("Invalid start URL");

    // Create URL filter configuration based on the start URL and config options
    let url_filter = create_url_filter(&root_url, config);

    // Create channels for communication
    let (crawl_tx, crawl_rx) = mpsc::channel::<String>(10000);
    let (result_tx, result_rx) = mpsc::channel::<PageData>(10000);

    // Initialize shared state
    let visited = Arc::new(Mutex::new(HashSet::new()));
    let crawl_rx = Arc::new(Mutex::new(crawl_rx));
    let web_semaphore = Arc::new(Semaphore::new(config.max_concurrency));
    let active_workers = Arc::new(Mutex::new(0));

    // Queue the initial URL
    crawl_tx.send(config.start_url.clone()).await.unwrap();

    // Start worker threads
    spawn_workers(
        config.max_concurrency,
        root_url,
        url_filter,
        crawl_tx.clone(),
        crawl_rx,
        result_tx,
        visited,
        web_semaphore,
        active_workers,
        &config.webdriver_url,
    );

    // Drop the original sender to signal when all workers are done
    drop(crawl_tx);

    result_rx
}

/// Backward compatibility function that uses default settings
pub async fn start_web_crawler(
    start_url: &str,
    max_concurrency: usize,
) -> mpsc::Receiver<PageData> {
    let mut config = WebCrawlerConfig::new(start_url);
    config.max_concurrency = max_concurrency;

    start(&config).await
}

/// Creates a URL filter based on the root URL and configuration
fn create_url_filter(root_url: &Url, config: &WebCrawlerConfig) -> Arc<UrlFilter> {
    let mut exclude_patterns =
        vec![r"\.(jpg|jpeg|png|gif|css|js|ico|woff|woff2|ttf|eot|svg|pdf)$".to_string()];

    // Add any user-defined exclude patterns
    exclude_patterns.extend(config.exclude_patterns.clone());

    let filter_config = UrlFilterConfig {
        allow_external: config.allow_external,
        required_domain: if !config.allow_external {
            root_url.domain().map(|d| d.to_string())
        } else {
            None
        },
        required_path_prefix: if !config.allow_external {
            Some(root_url.path().to_string())
        } else {
            None
        },
        include_patterns: config.include_patterns.clone(),
        exclude_patterns,
    };

    Arc::new(UrlFilter::new(filter_config).expect("Invalid regex pattern"))
}

/// Spawns worker threads to process URLs and returns a task that monitors completion
fn spawn_workers(
    max_concurrency: usize,
    root_url: Url,
    url_filter: Arc<UrlFilter>,
    crawl_tx: mpsc::Sender<String>,
    crawl_rx: Arc<Mutex<mpsc::Receiver<String>>>,
    result_tx: mpsc::Sender<PageData>,
    visited: Arc<Mutex<HashSet<String>>>,
    web_semaphore: Arc<Semaphore>,
    active_workers: Arc<Mutex<usize>>,
    webdriver_url: &str,
) {
    // Reduce number of initial workers - we'll use lazy initialization
    // so extra workers don't unnecessarily connect to WebDriver
    let num_workers = max_concurrency;

    // Now let's try a different approach - use a separate channel to signal worker completion
    let (completion_tx, mut completion_rx) = mpsc::channel::<()>(num_workers);

    // We need a mechanism to handle the case where a page has no links at all
    let initial_page_processed = Arc::new(Mutex::new(false));
    let initial_page_processed_clone = initial_page_processed.clone();

    for i in 0..num_workers {
        spawn_worker(
            i,
            webdriver_url.to_string(),
            root_url.clone(),
            Arc::clone(&url_filter),
            crawl_tx.clone(),
            Arc::clone(&crawl_rx),
            result_tx.clone(),
            Arc::clone(&visited),
            Arc::clone(&web_semaphore),
            Arc::clone(&active_workers),
            completion_tx.clone(),
            initial_page_processed.clone(),
        );
    }

    // Drop the sender we created - each worker has its own copy
    drop(completion_tx);

    // Return a task that monitors worker completion
    tokio::spawn(async move {
        // For the special case where there are no links at all, we need to ensure
        // we don't wait forever. Add a timeout for the initial page.
        let timeout_duration = tokio::time::Duration::from_secs(10);
        let _ = tokio::time::timeout(timeout_duration, async {
            // Wait for initial page to be processed
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            // If it's been 5 seconds and no page processed, we probably have an empty page
            let page_processed = {
                let flag = initial_page_processed_clone.lock().await;
                *flag
            };

            if !page_processed {
                // After 5 seconds with no page processed, assume we have no links
                ::log::info!("No links found, closing result channel early");
                drop(result_tx.clone());
            }
        })
        .await;

        // Wait for all workers to complete
        let mut completed_workers = 0;
        while let Some(_) = completion_rx.recv().await {
            completed_workers += 1;
            ::log::debug!(
                "Worker completed. {} of {} workers done.",
                completed_workers,
                num_workers
            );

            if completed_workers == num_workers {
                ::log::info!("All {} worker threads have completed", num_workers);
                // Once all workers are done, drop the result sender to close the channel
                drop(result_tx);
                break;
            }
        }
    });
}

/// Spawns a single worker
///
/// Creates an async task that will process URLs from the queue until
/// the queue is empty or an error occurs.
fn spawn_worker(
    worker_id: usize,
    webdriver_url: String,
    root_url: Url,
    url_filter: Arc<UrlFilter>,
    crawl_tx: mpsc::Sender<String>,
    crawl_rx: Arc<Mutex<mpsc::Receiver<String>>>,
    result_tx: mpsc::Sender<PageData>,
    visited: Arc<Mutex<HashSet<String>>>,
    web_semaphore: Arc<Semaphore>,
    active_workers: Arc<Mutex<usize>>,
    completion_tx: mpsc::Sender<()>,
    initial_page_processed: Arc<Mutex<bool>>,
) {
    ::log::trace!("Spawning worker {}", worker_id);

    tokio::spawn(async move {
        // Mark this worker as active
        increment_active_worker(worker_id, &active_workers).await;

        // Main processing loop - we'll connect to WebDriver only when needed
        if let Err(_) = worker_processing_loop(
            worker_id,
            None, // No client yet - will connect lazily when needed
            &webdriver_url,
            &root_url,
            &url_filter,
            &crawl_tx,
            &crawl_rx,
            &result_tx,
            &visited,
            &web_semaphore,
            Some(&initial_page_processed),
        )
        .await
        {
            ::log::warn!("Worker {} loop terminated with an error", worker_id);
        }

        // Worker is now complete - no client cleanup needed as it's handled in the processing loop
        decrement_active_worker(worker_id, &active_workers).await;

        // Signal that this worker is complete
        if let Err(e) = completion_tx.send(()).await {
            ::log::error!(
                "Worker {} failed to send completion signal: {}",
                worker_id,
                e
            );
        } else {
            ::log::debug!("Worker {} signaled completion", worker_id);
        }
    });
}

/// Increments the active worker counter
async fn increment_active_worker(worker_id: usize, active_workers: &Arc<Mutex<usize>>) {
    let mut active = active_workers.lock().await;
    *active += 1;
    ::log::debug!("Worker {} started, total active: {}", worker_id, *active);
}

/// Connects to the WebDriver instance
async fn connect_to_webdriver(worker_id: usize, webdriver_url: &str) -> Option<Client> {
    // Try to connect to the specified WebDriver URL
    match ClientBuilder::native().connect(webdriver_url).await {
        Ok(client) => {
            ::log::debug!(
                "Worker {} connected to WebDriver at {}",
                worker_id,
                webdriver_url
            );
            return Some(client);
        }
        Err(e) => {
            ::log::error!(
                "Worker {} failed to connect to WebDriver at {}: {}",
                worker_id,
                webdriver_url,
                e
            );
        }
    }

    // If we couldn't connect, try with common alternative URLs
    let fallback_urls = [
        "http://localhost:9515", // ChromeDriver default
        "http://localhost:4723", // Appium default
        "http://localhost:9222", // Chrome debug port default
        "http://127.0.0.1:4444", // Try with IP instead of localhost
    ];

    for url in fallback_urls.iter() {
        if *url == webdriver_url {
            continue; // Skip if it's the same as the one we already tried
        }

        ::log::info!(
            "Worker {} trying fallback WebDriver URL: {}",
            worker_id,
            url
        );
        match ClientBuilder::native().connect(url).await {
            Ok(client) => {
                ::log::debug!(
                    "Worker {} connected to fallback WebDriver at {}",
                    worker_id,
                    url
                );
                return Some(client);
            }
            Err(_) => {
                // Don't log error for fallbacks to avoid log spam
            }
        }
    }

    ::log::error!(
        "Worker {} failed to connect to any WebDriver servers",
        worker_id
    );
    ::log::error!(
        "Make sure a WebDriver server is running or set the WEBDRIVER_URL environment variable"
    );
    None
}

/// Main processing loop for a worker
///
/// Continuously processes URLs from the queue until the queue is empty
/// or an error occurs.
async fn worker_processing_loop(
    worker_id: usize,
    client_opt: Option<Client>,
    webdriver_url: &str,
    root_url: &Url,
    url_filter: &Arc<UrlFilter>,
    crawl_tx: &mpsc::Sender<String>,
    crawl_rx: &Arc<Mutex<mpsc::Receiver<String>>>,
    result_tx: &mpsc::Sender<PageData>,
    visited: &Arc<Mutex<HashSet<String>>>,
    web_semaphore: &Arc<Semaphore>,
    initial_page_processed: Option<&Arc<Mutex<bool>>>,
) -> Result<(), ()> {
    ::log::debug!("Worker {} starting processing loop", worker_id);

    // We'll connect to the WebDriver only if/when we actually have a URL to process
    let mut client_opt = client_opt;

    while let Some(url) = get_next_url(worker_id, crawl_rx).await {
        // Skip already visited URLs
        if !mark_url_as_visited(worker_id, &url, visited).await {
            continue;
        }

        // Acquire a permit from the semaphore before making a web request
        let _permit = web_semaphore.acquire().await.unwrap();
        ::log::debug!("Worker {} acquired web semaphore for: {}", worker_id, url);

        // Lazily initialize the WebDriver client if we don't have one yet
        if client_opt.is_none() {
            ::log::debug!("Worker {} connecting to WebDriver", worker_id);
            match connect_to_webdriver(worker_id, webdriver_url).await {
                Some(client) => client_opt = Some(client),
                None => {
                    // Failed to connect - release the permit and try another URL
                    continue;
                }
            }
        }

        // We now have a client - unwrap safely
        let client = client_opt.as_mut().unwrap();

        // Process the URL
        let scrape_result = process_url(worker_id, client, &url, webdriver_url).await;

        if let Some(page) = scrape_result {
            if !process_discovered_page(
                worker_id,
                &url,
                page,
                root_url,
                url_filter,
                result_tx,
                crawl_tx,
                visited,
                initial_page_processed,
            )
            .await
            {
                // Clean up client before returning error
                if let Some(client) = client_opt {
                    if let Err(e) = client.close().await {
                        ::log::warn!("Worker {} failed to close client: {}", worker_id, e);
                    }
                }
                return Err(());
            }
        } else {
            ::log::error!("Worker {} failed to scrape: {}", worker_id, url);
        }
    }

    // Close the client if we had one
    if let Some(client) = client_opt {
        if let Err(e) = client.close().await {
            ::log::warn!("Worker {} failed to close client: {}", worker_id, e);
        }
    }

    ::log::debug!(
        "Worker {} completed processing loop - no more URLs to process",
        worker_id
    );
    Ok(())
}

/// Gets the next URL to process from the queue
async fn get_next_url(
    worker_id: usize,
    crawl_rx: &Arc<Mutex<mpsc::Receiver<String>>>,
) -> Option<String> {
    let mut rx = crawl_rx.lock().await;

    // Use progressively increasing timeouts for workers
    // Worker 0 gets a longer timeout (for initial page processing)
    // Higher-numbered workers timeout faster to avoid long sequential shutdowns
    let timeout_duration = if worker_id == 0 {
        tokio::time::Duration::from_secs(5) // 5 seconds for worker 0
    } else {
        // Progressively shorter timeouts for higher worker IDs
        // This helps avoid the long serial shutdown seen in the logs
        let base_timeout: u64 = 5;
        let reduced_timeout = base_timeout.saturating_sub(worker_id.min(4) as u64);
        tokio::time::Duration::from_secs(reduced_timeout)
    };

    let url_result = tokio::time::timeout(timeout_duration, rx.recv()).await;

    // If we timed out, return None to end the worker
    let url = match url_result {
        Ok(result) => result, // Got a value before timeout
        Err(_) => {
            // Timed out waiting for a URL
            ::log::info!(
                "Worker {} timed out waiting for new URLs, assuming no more URLs",
                worker_id
            );
            return None;
        }
    };

    match &url {
        Some(url_str) => {
            ::log::trace!("Worker {} processing: {}", worker_id, url_str);
        }
        None => {
            ::log::info!(
                "Worker {} received channel close signal - no more URLs to process",
                worker_id
            );
        }
    }

    url
}

/// Checks if a URL has been visited and marks it as visited if not
async fn mark_url_as_visited(
    worker_id: usize,
    url: &str,
    visited: &Arc<Mutex<HashSet<String>>>,
) -> bool {
    let mut seen = visited.lock().await;
    if seen.contains(url) {
        ::log::trace!("Worker {} skipping already visited: {}", worker_id, url);
        return false;
    }
    seen.insert(url.to_string());
    true
}

/// Processes a URL by attempting to scrape it, with reconnection handling
async fn process_url(
    worker_id: usize,
    client: &mut Client,
    url: &str,
    webdriver_url: &str,
) -> Option<PageData> {
    let mut reconnect_attempted = false;
    let mut scrape_result = None;

    for attempt in 0..2 {
        if attempt > 0 {
            // If this is a retry, reconnect first
            reconnect_attempted = attempt_reconnect(worker_id, client, webdriver_url).await;
            if !reconnect_attempted {
                break;
            }
        }

        scrape_result = scrape(client, url, worker_id).await;

        // If scrape succeeded or it's not a session error, break the retry loop
        if scrape_result.is_some() || !reconnect_attempted {
            break;
        }
    }

    if scrape_result.is_some() {
        ::log::debug!("Worker {} completed scraping: {}", worker_id, url);
    }

    scrape_result
}

/// Attempts to reconnect the WebDriver client
async fn attempt_reconnect(worker_id: usize, client: &mut Client, webdriver_url: &str) -> bool {
    ::log::warn!(
        "Worker {} attempting to reconnect WebDriver session",
        worker_id
    );
    match ClientBuilder::native().connect(webdriver_url).await {
        Ok(new_client) => {
            *client = new_client;
            ::log::info!("Worker {} successfully reconnected to WebDriver", worker_id);
            true
        }
        Err(e) => {
            ::log::error!(
                "Worker {} failed to reconnect to WebDriver: {}",
                worker_id,
                e
            );
            false
        }
    }
}

/// Processes a successfully scraped page and its discovered links
async fn process_discovered_page(
    worker_id: usize,
    url: &str,
    page: PageData,
    root_url: &Url,
    url_filter: &Arc<UrlFilter>,
    result_tx: &mpsc::Sender<PageData>,
    crawl_tx: &mpsc::Sender<String>,
    visited: &Arc<Mutex<HashSet<String>>>,
    initial_page_processed: Option<&Arc<Mutex<bool>>>,
) -> bool {
    // Send the page data to the result channel
    if let Err(e) = result_tx.send(page.clone()).await {
        ::log::error!("Worker {} failed to send result: {}", worker_id, e);
        return false;
    }

    // If this is the initial page, mark it as processed
    if let Some(flag) = initial_page_processed {
        let mut processed = flag.lock().await;
        *processed = true;
        ::log::debug!("Marked initial page as processed");
    }

    // Process discovered links
    for link in page.links.iter() {
        if let Ok(resolved) = Url::parse(url).and_then(|base| base.join(link)) {
            // Use the URL filter to determine if we should crawl this link
            if !url_filter.should_crawl(&resolved, Some(root_url)) {
                ::log::debug!("URL filter rejected: {}", resolved);
                continue;
            }
            ::log::debug!("URL filter accepted: {}", resolved);

            // Normalize the URL (e.g., remove fragments)
            let normalized = url_filter.normalize_url(&resolved).to_string();

            // Check if we've already visited or queued this URL
            let should_send = {
                let seen = visited.lock().await;
                !seen.contains(&normalized)
            };

            if should_send {
                ::log::info!("Queuing link for crawling: {}", normalized);
                if let Err(e) = crawl_tx.send(normalized).await {
                    ::log::error!("Worker {} failed to send link: {}", worker_id, e);
                    return false;
                }
            } else {
                ::log::debug!("Skipping already visited or queued link: {}", normalized);
            }
        }
    }

    true
}

/// Decrements the active worker counter
async fn decrement_active_worker(worker_id: usize, active_workers: &Arc<Mutex<usize>>) {
    let mut active = active_workers.lock().await;
    *active -= 1;
    ::log::debug!(
        "Worker {} shutting down, remaining active: {}",
        worker_id,
        *active
    );
}

/// Scrapes a URL and returns the page data
async fn scrape(client: &Client, url: &str, worker_id: usize) -> Option<PageData> {
    // Add a worker-specific timeout to prevent individual scraping operations from hanging indefinitely
    let worker_start = std::time::Instant::now();
    ::log::debug!("SCRAPE: {}", url);

    // Determine the appropriate parser type based on the URL
    let parser_type = ParserType::from_url(url);
    let should_parse_links = parser_type.should_extract_links();

    // Add timeout for the entire scrape operation
    let scrape_result = timeout(tokio::time::Duration::from_secs(45), async {
        if !should_parse_links {
            scrape_text_file(client, url, worker_id, worker_start).await
        } else {
            scrape_html_page(client, url, worker_id, worker_start).await
        }
    })
    .await;

    match scrape_result {
        Ok(result) => result,
        Err(_) => {
            ::log::error!("Timeout scraping: {}", url);
            None
        }
    }
}

/// Scrapes a text-based file (non-HTML)
async fn scrape_text_file(
    client: &Client,
    url: &str,
    worker_id: usize,
    worker_start: std::time::Instant,
) -> Option<PageData> {
    ::log::debug!("Special handling for text-based file: {}", url);

    // Navigate to the URL
    match client.goto(url).await {
        Ok(_) => {}
        Err(e) => {
            return handle_navigation_error(e, "accessing text file", worker_id, url);
        }
    };

    // Get the page source
    let source = match client.source().await {
        Ok(source) => source,
        Err(e) => {
            return handle_navigation_error(e, "getting source for text file", worker_id, url);
        }
    };

    // Parse the content using our unified Parser interface with text options
    let text_options = parsers::text::TextParserOptions {
        preserve_paragraphs: true, // Keep paragraph structure with exactly one empty line
        preserve_line_breaks: false, // Don't preserve every line break
        normalize_whitespace: true, // Remove extra whitespace
        detect_urls: true,         // Keep URLs intact
    };
    let parser_result =
        parsers::Parser::parse_from_url_with_text_options(&source, url, &text_options);

    // Log processing time for debugging
    let elapsed = worker_start.elapsed().as_secs_f64();
    ::log::debug!(
        "Worker {} processed text file {} in {:.2} seconds",
        worker_id,
        url,
        elapsed
    );

    Some(PageData {
        url: url.to_string(),
        title: None, // Add missing title field
        content: parser_result.content,
        links: parser_result.links, // Will be empty for text files
    })
}

/// Scrapes an HTML page
async fn scrape_html_page(
    client: &Client,
    url: &str,
    worker_id: usize,
    worker_start: std::time::Instant,
) -> Option<PageData> {
    // Navigate to the URL
    match client.goto(url).await {
        Ok(_) => {}
        Err(e) => {
            return handle_navigation_error(e, "accessing", worker_id, url);
        }
    };

    // Get the page source
    let html = match client.source().await {
        Ok(source) => source,
        Err(e) => {
            return handle_navigation_error(e, "getting source for", worker_id, url);
        }
    };

    // Parse the HTML content using our unified Parser interface with custom text options
    // for any text content inside the HTML
    let text_options = parsers::text::TextParserOptions {
        preserve_paragraphs: true, // Keep paragraph structure with exactly one empty line
        preserve_line_breaks: false, // Don't preserve every line break
        normalize_whitespace: true, // Remove extra whitespace
        detect_urls: true,         // Keep URLs intact
    };
    let parser_result =
        parsers::Parser::parse_from_url_with_text_options(&html, url, &text_options);

    // Log the number of links found
    ::log::info!("Found {} links in {}", parser_result.links.len(), url);

    // Log processing time for debugging
    let elapsed = worker_start.elapsed().as_secs_f64();
    ::log::debug!(
        "Worker {} processed HTML {} in {:.2} seconds",
        worker_id,
        url,
        elapsed
    );

    Some(PageData {
        url: url.to_string(),
        title: None, // Add missing title field
        content: parser_result.content,
        links: parser_result.links,
    })
}

/// Handles errors that occur during navigation or page source retrieval
fn handle_navigation_error(
    error: fantoccini::error::CmdError,
    context: &str,
    worker_id: usize,
    url: &str,
) -> Option<PageData> {
    if error.to_string().contains("Unable to find session") {
        ::log::warn!(
            "Worker {} lost session while {} {}",
            worker_id,
            context,
            url
        );
    } else {
        ::log::error!("Failed to {} {}: {}", context, url, error);
    }
    None
}
