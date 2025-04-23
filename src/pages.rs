use crate::UriType;
use crate::config::{
    CrawlerConfigType, FilesystemCrawlerConfig, GitCrawlerConfig, S3CrawlerConfig, WebCrawlerConfig,
};
use std::path::Path;
use tokio::sync::mpsc;

/// Builder struct for configuring and running page crawling operations
pub struct Pages {
    // The URI to crawl
    uri: UriType,

    // The base configuration
    config: Option<CrawlerConfigType>,
}

impl Pages {
    /// Create a new Pages builder with the specified URI
    pub fn new(uri: UriType) -> Self {
        Self { uri, config: None }
    }

    /// Apply a configuration
    pub fn with_config(mut self, config: CrawlerConfigType) -> Self {
        self.config = Some(config);
        self
    }

    /// Load configuration from a JSON file
    pub fn with_config_file<P: AsRef<Path>>(
        mut self,
        path: P,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config = CrawlerConfigType::from_file(path)?;
        self.config = Some(config);
        Ok(self)
    }

    /// Apply configuration from a JSON string
    pub fn with_config_str(mut self, json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = CrawlerConfigType::from_json(json)?;
        self.config = Some(config);
        Ok(self)
    }

    /// Override the max concurrency setting
    pub fn with_max_concurrency(mut self, value: usize) -> Self {
        // Update the config if it exists, otherwise create a new one based on URI type
        if let Some(config) = &mut self.config {
            match config {
                CrawlerConfigType::Web(cfg) => cfg.max_concurrency = value,
                CrawlerConfigType::Git(cfg) => cfg.max_concurrency = value,
                CrawlerConfigType::Filesystem(cfg) => cfg.max_concurrency = value,
                CrawlerConfigType::S3(cfg) => cfg.max_concurrency = value,
            }
        } else {
            // Create a new config based on URI type
            self.config = Some(match &self.uri {
                UriType::Web(url) => {
                    let mut cfg = WebCrawlerConfig::new(url);
                    cfg.max_concurrency = value;
                    CrawlerConfigType::Web(cfg)
                }
                UriType::Git(url) => {
                    let mut cfg = GitCrawlerConfig::new(url);
                    cfg.max_concurrency = value;
                    CrawlerConfigType::Git(cfg)
                }
                UriType::Filesystem(path) => {
                    let mut cfg = FilesystemCrawlerConfig::new(path);
                    cfg.max_concurrency = value;
                    CrawlerConfigType::Filesystem(cfg)
                }
                UriType::S3(bucket, region) => {
                    let mut cfg = S3CrawlerConfig::new(bucket, region);
                    cfg.max_concurrency = value;
                    CrawlerConfigType::S3(cfg)
                }
            });
        }
        self
    }

    /// Override the idle timeout setting
    pub fn with_idle_timeout(mut self, seconds: u64) -> Self {
        if let Some(config) = &mut self.config {
            match config {
                CrawlerConfigType::Web(cfg) => cfg.idle_timeout_secs = seconds,
                CrawlerConfigType::Git(cfg) => cfg.idle_timeout_secs = seconds,
                CrawlerConfigType::Filesystem(cfg) => cfg.idle_timeout_secs = seconds,
                CrawlerConfigType::S3(cfg) => cfg.idle_timeout_secs = seconds,
            }
        } else {
            // Create a new config based on URI type with default values
            self.config = Some(match &self.uri {
                UriType::Web(url) => {
                    let mut cfg = WebCrawlerConfig::new(url);
                    cfg.idle_timeout_secs = seconds;
                    CrawlerConfigType::Web(cfg)
                }
                UriType::Git(url) => {
                    let mut cfg = GitCrawlerConfig::new(url);
                    cfg.idle_timeout_secs = seconds;
                    CrawlerConfigType::Git(cfg)
                }
                UriType::Filesystem(path) => {
                    let mut cfg = FilesystemCrawlerConfig::new(path);
                    cfg.idle_timeout_secs = seconds;
                    CrawlerConfigType::Filesystem(cfg)
                }
                UriType::S3(bucket, region) => {
                    let mut cfg = S3CrawlerConfig::new(bucket, region);
                    cfg.idle_timeout_secs = seconds;
                    CrawlerConfigType::S3(cfg)
                }
            });
        }
        self
    }

    /// Override the total timeout setting
    pub fn with_total_timeout(mut self, seconds: u64) -> Self {
        if let Some(config) = &mut self.config {
            match config {
                CrawlerConfigType::Web(cfg) => cfg.total_timeout_secs = seconds,
                CrawlerConfigType::Git(cfg) => cfg.total_timeout_secs = seconds,
                CrawlerConfigType::Filesystem(cfg) => cfg.total_timeout_secs = seconds,
                CrawlerConfigType::S3(cfg) => cfg.total_timeout_secs = seconds,
            }
        } else {
            // Create a new config based on URI type with default values
            self.config = Some(match &self.uri {
                UriType::Web(url) => {
                    let mut cfg = WebCrawlerConfig::new(url);
                    cfg.total_timeout_secs = seconds;
                    CrawlerConfigType::Web(cfg)
                }
                UriType::Git(url) => {
                    let mut cfg = GitCrawlerConfig::new(url);
                    cfg.total_timeout_secs = seconds;
                    CrawlerConfigType::Git(cfg)
                }
                UriType::Filesystem(path) => {
                    let mut cfg = FilesystemCrawlerConfig::new(path);
                    cfg.total_timeout_secs = seconds;
                    CrawlerConfigType::Filesystem(cfg)
                }
                UriType::S3(bucket, region) => {
                    let mut cfg = S3CrawlerConfig::new(bucket, region);
                    cfg.total_timeout_secs = seconds;
                    CrawlerConfigType::S3(cfg)
                }
            });
        }
        self
    }

    /// Start the crawling process and generate a stream of pages
    pub async fn generate(
        self,
    ) -> Result<mpsc::Receiver<crate::PageData>, Box<dyn std::error::Error>> {
        // If we have a configuration, use it, otherwise create one from the URI
        let config = if let Some(ref config) = self.config {
            config.clone()
        } else {
            // Create a default configuration based on the URI type
            match &self.uri {
                UriType::Web(url) => CrawlerConfigType::Web(WebCrawlerConfig::new(url)),
                UriType::Git(url) => CrawlerConfigType::Git(GitCrawlerConfig::new(url)),
                UriType::Filesystem(path) => {
                    CrawlerConfigType::Filesystem(FilesystemCrawlerConfig::new(path))
                }
                UriType::S3(bucket, region) => {
                    CrawlerConfigType::S3(S3CrawlerConfig::new(bucket, region))
                }
            }
        };

        // Get the appropriate receiver based on the configuration
        match crate::start_crawler_with_config(&config).await {
            Some(rx) => Ok(rx),
            None => Err("No valid crawler configuration found".into()),
        }
    }
}
