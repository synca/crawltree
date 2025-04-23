use clap::Parser;
use yield_page::Pages;
use yield_page::results::PageData;

mod args;
use args::{Args, convert_uri_type};

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    // Parse command-line arguments
    let args = Args::parse();

    ::log::info!("Starting crawler for URI: {}", args.uri);

    // Convert from CLI argument URI type to internal URI type
    let uri_type = convert_uri_type(args.type_, &args.uri);

    // Print WebDriver info message for web URIs
    if let yield_page::UriType::Web(_) = &uri_type {
        println!("Note: Web crawling requires a WebDriver server (e.g., ChromeDriver).");
        println!(
            "Set WEBDRIVER_URL environment variable if not using the default http://localhost:4444"
        );
    };

    // Create a Pages builder with the specified parameters
    let pages = Pages::new(uri_type)
        .with_max_concurrency(args.concurrency)
        .with_idle_timeout(args.idle_timeout)
        .with_total_timeout(args.total_timeout);

    // Start the crawler and get a receiver for pages
    let mut rx = match pages.generate().await {
        Ok(rx) => rx,
        Err(e) => {
            ::log::error!("Failed to start crawler: {}", e);
            return;
        }
    };

    // Process pages as they come in
    let mut pages_processed = 0;
    let start_time = std::time::Instant::now();
    ::log::info!("Started processing pages at {:?}", start_time);

    while let Some(page) = rx.recv().await {
        pages_processed += 1;
        process_page(&page, pages_processed);
    }

    let duration = start_time.elapsed();
    ::log::info!(
        "Crawling complete - processed {} pages in {:.2} seconds",
        pages_processed,
        duration.as_secs_f64()
    );
}

// Example function to process a page
fn process_page(page: &PageData, count: i32) {
    ::log::info!("Processed page {}: {}", count, page.url);
    ::log::debug!("Page has {} links", page.links.len());

    // In a real application, you would do something with the page data here
    // For example, save it to a database, index it for search, etc.
}
