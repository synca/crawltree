use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use yield_page::{Pages, UriType, config::CrawlerConfigType};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to crawler configuration file
    #[arg(short, long)]
    config: String,

    /// Override idle timeout in seconds
    #[arg(short, long)]
    idle_timeout: Option<u64>,

    /// Override total timeout in seconds
    #[arg(short, long)]
    total_timeout: Option<u64>,

    /// Override max concurrency
    #[arg(short, long)]
    concurrency: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    env_logger::init();

    // Parse command line arguments
    let args = Args::parse();

    // Load configuration from file
    let config_path = PathBuf::from(&args.config);
    let config = CrawlerConfigType::from_file(config_path)?;

    // Print the loaded configuration (for debugging)
    println!(
        "Loaded configuration of type: {:?}",
        std::mem::discriminant(&config)
    );

    // Extract URI from config
    let uri = match &config {
        CrawlerConfigType::Web(web_config) => {
            println!("Web crawler configuration:");
            println!("  Start URL: {}", web_config.start_url);
            println!("  Max concurrency: {}", web_config.max_concurrency);
            println!("  WebDriver URL: {}", web_config.webdriver_url);
            println!(
                "  Number of include patterns: {}",
                web_config.include_patterns.len()
            );
            println!(
                "  Number of exclude patterns: {}",
                web_config.exclude_patterns.len()
            );
            UriType::Web(web_config.start_url.clone())
        }
        CrawlerConfigType::Git(git_config) => {
            println!("Git crawler configuration:");
            println!("  Repository URL: {}", git_config.repo_url);
            println!("  Branch: {}", git_config.branch);
            UriType::Git(git_config.repo_url.clone())
        }
        CrawlerConfigType::Filesystem(fs_config) => {
            println!("Filesystem crawler configuration:");
            println!("  Root directory: {}", fs_config.root_dir);
            println!("  Max depth: {}", fs_config.max_depth);
            UriType::Filesystem(fs_config.root_dir.clone())
        }
        CrawlerConfigType::S3(s3_config) => {
            println!("S3 crawler configuration:");
            println!("  Bucket: {}", s3_config.bucket);
            println!("  Region: {}", s3_config.region);
            println!("  Prefix: {}", s3_config.prefix);
            UriType::S3(s3_config.bucket.clone(), s3_config.region.clone())
        }
    };

    // Create a Pages builder with the URI and configuration
    let mut pages_builder = Pages::new(uri).with_config(config);

    // Apply overrides if specified
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

    // Start the crawler
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
