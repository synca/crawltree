use crate::filter::UrlFilter;
use crate::parsers::ParseResult;
use std::sync::Arc;
use url::Url;

// Define a base trait for crawlers
pub trait Crawler {
    /// Initialize the crawler with a root URL
    fn init(&mut self, root_url: Url);

    /// Start the crawling process
    fn start(&mut self);

    /// Set the URL filter for determining which URLs to crawl
    fn set_url_filter(&mut self, filter: Arc<UrlFilter>);

    /// Process a discovered page
    fn process_page(&self, url: &str, parse_result: ParseResult);
}
