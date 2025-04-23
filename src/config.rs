use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Configuration for the web crawler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebCrawlerConfig {
    /// URL to start crawling from
    pub start_url: String,

    /// Maximum number of concurrent requests
    #[serde(default = "default_max_concurrency")]
    pub max_concurrency: usize,

    /// Whether to allow crawling external domains/sites
    #[serde(default)]
    pub allow_external: bool,

    /// Regex patterns for URLs to include
    #[serde(default)]
    pub include_patterns: Vec<String>,

    /// Regex patterns for URLs to exclude
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// URL for the WebDriver instance
    #[serde(default = "default_webdriver_url")]
    pub webdriver_url: String,
}

/// Configuration for Git repository crawler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCrawlerConfig {
    /// Git repository URL
    pub repo_url: String,

    /// Branch to clone (defaults to main)
    #[serde(default = "default_git_branch")]
    pub branch: String,

    /// Patterns to include
    #[serde(default)]
    pub include_patterns: Vec<String>,

    /// Patterns to exclude
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

/// Configuration for filesystem crawler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemCrawlerConfig {
    /// Root directory to crawl
    pub root_dir: String,

    /// Maximum recursion depth
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,

    /// File patterns to include
    #[serde(default)]
    pub include_patterns: Vec<String>,

    /// File patterns to exclude
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

/// Configuration for S3 crawler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3CrawlerConfig {
    /// S3 bucket name
    pub bucket: String,

    /// AWS region
    pub region: String,

    /// S3 key prefix
    #[serde(default)]
    pub prefix: String,

    /// File patterns to include
    #[serde(default)]
    pub include_patterns: Vec<String>,

    /// File patterns to exclude
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

/// Enum containing all crawler configuration types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CrawlerConfigType {
    /// Web crawler configuration
    Web(WebCrawlerConfig),

    /// Git crawler configuration
    Git(GitCrawlerConfig),

    /// Filesystem crawler configuration
    Filesystem(FilesystemCrawlerConfig),

    /// S3 crawler configuration
    S3(S3CrawlerConfig),
}

impl CrawlerConfigType {
    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config: Self = serde_json::from_str(&contents)?;
        Ok(config)
    }
}

/// Default value for max_concurrency
fn default_max_concurrency() -> usize {
    5
}

/// Default value for webdriver_url
fn default_webdriver_url() -> String {
    "http://localhost:4444".to_string()
}

/// Default git branch
fn default_git_branch() -> String {
    "main".to_string()
}

/// Default max recursion depth for filesystem crawler
fn default_max_depth() -> usize {
    10
}

impl WebCrawlerConfig {
    /// Create a new configuration with default values
    pub fn new(start_url: &str) -> Self {
        Self {
            start_url: start_url.to_string(),
            max_concurrency: default_max_concurrency(),
            allow_external: false,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            webdriver_url: default_webdriver_url(),
        }
    }
}
