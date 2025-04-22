use clap::Parser;
use std::error::Error;
use yield_page::{Pages, UriType};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL to crawl
    #[arg(short, long)]
    url: String,

    /// JSON configuration string
    #[arg(short, long)]
    config: Option<String>,

    /// Path to JSON configuration file
    #[arg(short, long)]
    config_file: Option<String>,

    /// Maximum concurrency level
    #[arg(short, long)]
    concurrency: Option<usize>,

    /// Idle timeout in seconds
    #[arg(short, long)]
    idle_timeout: Option<u64>,

    /// Total runtime timeout in seconds
    #[arg(short, long)]
    total_timeout: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    let args = Args::parse();

    println!("Starting crawler for URL: {}", args.url);

    // Create a Pages builder with the URI
    let uri = UriType::Web(args.url);
    let mut pages_builder = Pages::new(uri);

    // Apply configuration from file if specified
    if let Some(config_file) = args.config_file {
        println!("Loading configuration from file: {}", config_file);
        pages_builder = pages_builder.with_config_file(config_file)?;
    }

    // Apply configuration from string if specified (overrides file config)
    if let Some(config_str) = args.config {
        println!("Applying configuration from string");
        pages_builder = pages_builder.with_config_str(&config_str)?;
    }

    // Apply command-line overrides
    if let Some(concurrency) = args.concurrency {
        println!("Overriding max concurrency: {}", concurrency);
        pages_builder = pages_builder.with_max_concurrency(concurrency);
    }

    if let Some(idle_timeout) = args.idle_timeout {
        println!("Overriding idle timeout: {}s", idle_timeout);
        pages_builder = pages_builder.with_idle_timeout(idle_timeout);
    }

    if let Some(total_timeout) = args.total_timeout {
        println!("Overriding total timeout: {}s", total_timeout);
        pages_builder = pages_builder.with_total_timeout(total_timeout);
    }

    // Start the crawling process
    let mut rx = pages_builder.generate().await?;

    // Process pages as they come in
    let mut pages_crawled = 0;
    let start_time = std::time::Instant::now();
    println!("Starting processing pages at {:?}", start_time);

    while let Some(page) = rx.recv().await {
        pages_crawled += 1;
        println!("Received page {}: {}", pages_crawled, page.url);
    }

    let duration = start_time.elapsed();
    println!(
        "Crawling complete. Processed {} pages in {:.2} seconds.",
        pages_crawled,
        duration.as_secs_f64()
    );

    Ok(())
}
