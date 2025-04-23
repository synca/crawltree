#![allow(clippy::too_many_arguments)]

// Re-export modules
pub mod config;
pub mod crawlers;
pub mod filter;
pub mod parsers;
pub mod results;
pub mod utils;

// Re-export commonly used types for convenience
pub use results::PageData;

use std::time::Duration;
use tokio::sync::mpsc;

/// Types of URIs that can be crawled
#[derive(Debug, Clone)]
pub enum UriType {
    /// Web URLs
    Web(String),
    /// Git repositories
    Git(String),
    /// Local filesystem
    Filesystem(String),
    /// Amazon S3 bucket
    S3(String, String), // Bucket name, region
}

/// Main builder for page generation from different URI types
pub struct Pages {
    uri_type: UriType,
    max_concurrency: usize,
    idle_timeout: Option<Duration>,
    total_timeout: Option<Duration>,
}

impl Pages {
    /// Create a new Pages builder with the given URI type
    pub fn new(uri_type: UriType) -> Self {
        Self {
            uri_type,
            max_concurrency: 4, // Default concurrency
            idle_timeout: None,
            total_timeout: None,
        }
    }

    /// Set the maximum number of concurrent crawlers
    pub fn with_max_concurrency(mut self, max_concurrency: usize) -> Self {
        self.max_concurrency = max_concurrency;
        self
    }

    /// Set the idle timeout (crawler stops if no new pages for this duration)
    pub fn with_idle_timeout(mut self, timeout_seconds: u64) -> Self {
        self.idle_timeout = Some(Duration::from_secs(timeout_seconds));
        self
    }

    /// Set the total timeout (maximum runtime)
    pub fn with_total_timeout(mut self, timeout_seconds: u64) -> Self {
        self.total_timeout = Some(Duration::from_secs(timeout_seconds));
        self
    }

    /// Set the configuration from a CrawlerConfigType
    pub fn with_config(mut self, config: config::CrawlerConfigType) -> Self {
        // Configure the builder based on the provided configuration
        match &config {
            config::CrawlerConfigType::Web(web_config) => {
                self.max_concurrency = web_config.max_concurrency;
            }
            config::CrawlerConfigType::Git(_) => {
                // Set Git-specific options
            }
            config::CrawlerConfigType::Filesystem(_) => {
                // Set Filesystem-specific options
            }
            config::CrawlerConfigType::S3(_) => {
                // Set S3-specific options
            }
        }
        self
    }

    /// Load configuration from a file
    pub fn with_config_file(
        self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config = config::CrawlerConfigType::from_file(path)?;
        Ok(self.with_config(config))
    }

    /// Load configuration from a string
    pub fn with_config_str(self, config_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = serde_json::from_str(config_str)?;
        Ok(self.with_config(config))
    }

    /// Start the crawler and get a receiver for pages
    pub async fn generate(self) -> Result<mpsc::Receiver<PageData>, Box<dyn std::error::Error>> {
        match self.uri_type {
            UriType::Web(url_str) => {
                // Create web crawler configuration
                let mut web_config = config::WebCrawlerConfig::new(&url_str);
                web_config.max_concurrency = self.max_concurrency;

                // Override the WebDriver URL with an environment variable if provided
                if let Ok(webdriver_url) = std::env::var("WEBDRIVER_URL") {
                    if !webdriver_url.is_empty() {
                        web_config.webdriver_url = webdriver_url;
                    }
                }

                // Start the web crawler
                let receiver = crawlers::web::start(&web_config).await;
                Ok(receiver)
            }
            UriType::Git(_) => {
                // Placeholder for Git implementation
                unimplemented!("Git crawler not yet implemented")
            }
            UriType::Filesystem(_) => {
                // Placeholder for Filesystem implementation
                unimplemented!("Filesystem crawler not yet implemented")
            }
            UriType::S3(_, _) => {
                // Placeholder for S3 implementation
                unimplemented!("S3 crawler not yet implemented")
            }
        }
    }
}
