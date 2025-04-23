use clap::{Parser, ValueEnum};
use yield_page::UriType;

#[derive(Parser, Debug)]
#[command(name = "yield-page")]
#[command(author = "Ryan Northey <ryan@synca.io>")]
#[command(about = "Crawler that yields pages from various URI types")]
#[command(version)]
pub struct Args {
    /// Source URI to crawl (web URL, git repo, file path, etc.)
    pub uri: String,

    /// URI type (web, git, file, s3)
    #[arg(short, long, value_enum, default_value_t = UriTypeArg::Web)]
    pub type_: UriTypeArg,

    /// Number of concurrent crawlers
    #[arg(short, long, default_value_t = 4)]
    pub concurrency: usize,

    /// Idle timeout in seconds (crawler stops if no new pages for this duration)
    #[arg(long, default_value_t = 300)] // 5 minutes
    pub idle_timeout: u64,

    /// Total timeout in seconds (maximum runtime)
    #[arg(long, default_value_t = 1200)] // 20 minutes
    pub total_timeout: u64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum UriTypeArg {
    Web,
    // Uncomment these as they're implemented
    // Git,
    // File,
    // S3,
}

/// Convert from CLI argument URI type to internal URI type
pub fn convert_uri_type(arg_type: UriTypeArg, uri: &str) -> UriType {
    match arg_type {
        UriTypeArg::Web => UriType::Web(uri.to_string()),
        // Add other URI types as they're implemented
    }
}
