use fantoccini::{Client, ClientBuilder};
use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::time::timeout;
use url::Url;

#[derive(Serialize, Debug, Clone)]
pub struct PageData {
    pub url: String,
    pub content: String,
    pub links: Vec<String>,
}

/// Starts an async crawl and returns a receiver that yields PageData as discovered.
pub async fn start_crawler(start_url: &str, max_concurrency: usize) -> mpsc::Receiver<PageData> {
    println!("Starting crawler for: {}", start_url);

    let root_url = Url::parse(start_url).expect("Invalid start URL");
    let root_domain = root_url.domain().unwrap().to_string();
    let root_path = root_url.path().to_string();

    let (crawl_tx, mut crawl_rx) = mpsc::channel::<String>(10000);
    let (result_tx, result_rx) = mpsc::channel::<PageData>(10000);

    let visited = Arc::new(Mutex::new(HashSet::new()));
    crawl_tx.send(start_url.to_string()).await.unwrap();

    let crawl_rx = Arc::new(Mutex::new(crawl_rx));

    // Create a single WebDriver client to be shared among workers
    let client_builder = ClientBuilder::native();
    let client = client_builder
        .connect("http://webdriver:4444/")
        .await
        .expect("Failed to connect to WebDriver");
    let client = Arc::new(client);

    for i in 0..max_concurrency {
        let crawl_rx = Arc::clone(&crawl_rx);
        let crawl_tx = crawl_tx.clone();
        let result_tx = result_tx.clone();
        let visited = Arc::clone(&visited);
        let root_domain = root_domain.clone();
        let root_path = root_path.clone();
        let client = Arc::clone(&client);

        println!("Spawning worker {}", i);
        tokio::spawn(async move {

            while let Some(url) = {
                let mut rx = crawl_rx.lock().await;
                rx.recv().await
            } {
                println!("Worker {} processing: {}", i, url);
                {
                    let mut seen = visited.lock().await;
                    if seen.contains(&url) {
                        println!("Worker {} skipping already visited: {}", i, url);
                        continue;
                    }
                    seen.insert(url.clone());
                }

                if let Some(page) = scrape(&client, &url).await {
                    if let Err(e) = result_tx.send(page.clone()).await {
                        eprintln!("Worker {} failed to send result: {}", i, e);
                        break;
                    }

                    for link in page.links {
                        if let Ok(resolved) = Url::parse(&url).and_then(|base| base.join(&link)) {
                            let skip_exts =
                                ["pdf", "jpg", "jpeg", "png", "gif", "css", "js", "ico"];
                            if skip_exts.iter().any(|ext| resolved.path().ends_with(ext)) {
                                continue;
                            }

                            // Check domain first
                            if let Some(domain) = resolved.domain() {
                                if domain != root_domain {
                                    continue;
                                }
                            }
                            
                            // Check if URL is within our allowed path
                            if !resolved.path().starts_with(&root_path) {
                                continue;
                            }
                            let mut normalized = resolved.clone();
                            normalized.set_fragment(None);
                            let link_str = normalized.to_string();

                            let should_send = {
                                let seen = visited.lock().await;
                                !seen.contains(&link_str)
                            };
                            if should_send {
                                println!("TX: {}", link_str);
                                if let Err(e) = crawl_tx.send(link_str).await {
                                    eprintln!("Worker {} failed to send link: {}", i, e);
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // Remove individual client close
            println!("Worker {} shutting down", i);
        });
    }

    // Drop the original sender to signal when all workers are done
    drop(crawl_tx);
    
    // Spawn a cleanup task to close the client when all workers are done
    let client_cleanup = Arc::clone(&client);
    tokio::spawn(async move {
        // Wait a bit to ensure all workers have started
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        // Wait for all workers to finish by checking if the client is the only reference
        while Arc::strong_count(&client_cleanup) > 1 {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        
        // Now close the client
        if let Ok(client) = Arc::try_unwrap(client_cleanup) {
            client.close().await.ok();
            println!("WebDriver client closed");
        }
    });
    
    result_rx
}

async fn scrape(client: &Client, url: &str) -> Option<PageData> {
    println!("SCRAPE: {}", url);
    
    // Add timeout for the entire scrape operation
    let scrape_result = timeout(tokio::time::Duration::from_secs(30), async {
        client.goto(url).await.ok()?;
        let html = client.source().await.ok()?;
        let doc = Html::parse_document(&html);

        let content_selector = Selector::parse("body").unwrap();
        let text = doc
            .select(&content_selector)
            .flat_map(|n| n.text())
            .collect::<Vec<_>>()
            .join(" ");

        let link_selector = Selector::parse("a").unwrap();
        let links = doc
            .select(&link_selector)
            .filter_map(|e| e.value().attr("href"))
            .map(|s| s.to_string())
            .collect();

        Some(PageData {
            url: url.to_string(),
            content: text,
            links,
        })
    }).await;
    
    match scrape_result {
        Ok(result) => result,
        Err(_) => {
            eprintln!("Timeout scraping: {}", url);
            None
        }
    }
}
