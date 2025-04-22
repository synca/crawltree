use regex::Regex;
use serde::{Deserialize, Serialize};
use url::Url;

/// Configuration for URL filtering in crawlers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlFilterConfig {
    /// Whether to allow crawling external domains/sites
    #[serde(default = "default_allow_external")]
    pub allow_external: bool,

    /// Domain restriction for crawling (if None, all domains are allowed if allow_external is true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_domain: Option<String>,

    /// Path prefix restriction (if None, all paths are allowed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_path_prefix: Option<String>,

    /// Regex patterns for URLs to include (if empty, all URLs are included unless excluded)
    #[serde(default)]
    pub include_patterns: Vec<String>,

    /// Regex patterns for URLs to exclude (these take precedence over include patterns)
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

/// Default value for allow_external field (false for safety)
fn default_allow_external() -> bool {
    false
}

impl Default for UrlFilterConfig {
    fn default() -> Self {
        Self {
            allow_external: false,
            required_domain: None,
            required_path_prefix: None,
            include_patterns: Vec::new(),
            exclude_patterns: vec![
                // Common file types to exclude by default
                r"\.(jpg|jpeg|png|gif|css|js|ico|svg|woff|woff2|ttf|eot|pdf)$".to_string(),
                // Common directories to exclude
                r"/_sources/".to_string(),
            ],
        }
    }
}

/// URL filter that uses regex patterns and other rules to determine which URLs to crawl
#[derive(Debug)]
pub struct UrlFilter {
    config: UrlFilterConfig,
    include_regexes: Vec<Regex>,
    exclude_regexes: Vec<Regex>,
}

impl Default for UrlFilter {
    fn default() -> Self {
        Self::new(UrlFilterConfig::default()).expect("Default regex patterns should be valid")
    }
}

impl UrlFilter {
    /// Create a new URL filter from configuration
    pub fn new(config: UrlFilterConfig) -> Result<Self, regex::Error> {
        // Compile regex patterns
        let mut include_regexes = Vec::with_capacity(config.include_patterns.len());
        for pattern in &config.include_patterns {
            include_regexes.push(Regex::new(pattern)?);
        }

        let mut exclude_regexes = Vec::with_capacity(config.exclude_patterns.len());
        for pattern in &config.exclude_patterns {
            exclude_regexes.push(Regex::new(pattern)?);
        }

        Ok(Self {
            config,
            include_regexes,
            exclude_regexes,
        })
    }

    /// Create a new URL filter with custom configuration
    pub fn with_config(config: UrlFilterConfig) -> Result<Self, regex::Error> {
        Self::new(config)
    }

    /// Determine if a URL should be crawled based on all filtering rules
    pub fn should_crawl(&self, url: &Url, _base_url: Option<&Url>) -> bool {
        // Check domain restrictions
        if !self.is_in_domain_scope(url) {
            return false;
        }

        // Check path prefix
        if !self.is_in_path_scope(url) {
            return false;
        }

        // Check regex exclusions (these take precedence)
        let url_str = url.as_str();
        for regex in &self.exclude_regexes {
            if regex.is_match(url_str) {
                return false;
            }
        }

        // If include patterns are specified, at least one must match
        if !self.include_regexes.is_empty() {
            let mut included = false;
            for regex in &self.include_regexes {
                if regex.is_match(url_str) {
                    included = true;
                    break;
                }
            }
            if !included {
                return false;
            }
        }

        // If we've reached here, the URL passed all filters
        true
    }

    /// Check if a URL should be parsed for links (some text-based files shouldn't be parsed)
    pub fn should_parse_links(&self, url: &Url) -> bool {
        // Don't parse text files, YAML files, etc. for links
        let url_str = url.as_str();

        // Common text-based files that shouldn't be parsed for links
        let no_parse_patterns = [r"\.txt$", r"\.ya?ml$", r"/_sources/"];

        for pattern in &no_parse_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(url_str) {
                    return false;
                }
            }
        }

        true
    }

    /// Check if a URL is within the allowed domain scope
    fn is_in_domain_scope(&self, url: &Url) -> bool {
        // If external domains are allowed and no specific domain is required, all domains are allowed
        if self.config.allow_external && self.config.required_domain.is_none() {
            return true;
        }

        // Otherwise, check if the domain matches the required domain
        if let Some(required_domain) = &self.config.required_domain {
            if let Some(url_domain) = url.domain() {
                return url_domain == required_domain;
            }
            return false; // No domain in URL but domain required
        }

        // If we get here, allow_external is false and no required_domain
        // In this case, we should reject all external domains
        false
    }

    /// Check if a URL is within the required path scope
    fn is_in_path_scope(&self, url: &Url) -> bool {
        if let Some(prefix) = &self.config.required_path_prefix {
            url.path().starts_with(prefix)
        } else {
            true // No path restriction
        }
    }

    /// Create a normalized version of the URL (e.g., removing fragments)
    pub fn normalize_url(&self, url: &Url) -> Url {
        let mut normalized = url.clone();
        normalized.set_fragment(None);
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_filter() {
        let filter = UrlFilter::default();

        // Common file types should be excluded
        let image_url = Url::parse("https://example.com/image.jpg").unwrap();
        assert!(!filter.should_crawl(&image_url, None));

        // HTML files would be included with the right domain settings
        // but are rejected due to default allow_external:false
        let html_url = Url::parse("https://example.com/page.html").unwrap();
        let result = filter.should_crawl(&html_url, None);
        assert!(
            !result,
            "HTML URL should be rejected with default settings due to external domain"
        );

        // Test with a filter that does allow this domain
        let config = UrlFilterConfig {
            allow_external: true,  // Allow external URLs
            required_domain: None, // No domain restriction
            required_path_prefix: None,
            include_patterns: vec![],
            exclude_patterns: vec![
                // Same default excludes
                r"\.(jpg|jpeg|png|gif|css|js|ico|svg|woff|woff2|ttf|eot|pdf)$".to_string(),
                r"/_sources/".to_string(),
            ],
        };
        let filter_allowing_external = UrlFilter::new(config).unwrap();
        assert!(filter_allowing_external.should_crawl(&html_url, None));
    }

    #[test]
    fn test_domain_restriction() {
        let config = UrlFilterConfig {
            allow_external: false,
            required_domain: Some("example.com".to_string()),
            required_path_prefix: None,
            include_patterns: vec![],
            exclude_patterns: vec![],
        };
        let filter = UrlFilter::new(config).unwrap();

        // Correct domain should be allowed
        let correct_domain = Url::parse("https://example.com/page").unwrap();
        assert!(filter.should_crawl(&correct_domain, None));

        // Different domain should be excluded
        let wrong_domain = Url::parse("https://other.com/page").unwrap();
        assert!(!filter.should_crawl(&wrong_domain, None));
    }

    #[test]
    fn test_path_restriction() {
        let config = UrlFilterConfig {
            allow_external: true,
            required_domain: None,
            required_path_prefix: Some("/docs".to_string()),
            include_patterns: vec![],
            exclude_patterns: vec![],
        };
        let filter = UrlFilter::new(config).unwrap();

        // Correct path should be allowed
        let correct_path = Url::parse("https://example.com/docs/page").unwrap();
        assert!(filter.should_crawl(&correct_path, None));

        // Different path should be excluded
        let wrong_path = Url::parse("https://example.com/blog/post").unwrap();
        assert!(!filter.should_crawl(&wrong_path, None));
    }

    #[test]
    fn test_regex_patterns() {
        let config = UrlFilterConfig {
            allow_external: true,
            required_domain: None,
            required_path_prefix: None,
            include_patterns: vec![r"/docs/.*\.html$".to_string()],
            exclude_patterns: vec![r"/docs/draft/".to_string()],
        };
        let filter = UrlFilter::new(config).unwrap();

        // Matching include pattern should be allowed
        let included = Url::parse("https://example.com/docs/page.html").unwrap();
        assert!(filter.should_crawl(&included, None));

        // Non-matching include pattern should be excluded
        let not_included = Url::parse("https://example.com/docs/page.txt").unwrap();
        assert!(!filter.should_crawl(&not_included, None));

        // Matching exclude pattern should be excluded even if it matches include
        let excluded = Url::parse("https://example.com/docs/draft/page.html").unwrap();
        assert!(!filter.should_crawl(&excluded, None));
    }

    #[test]
    fn test_should_parse_links() {
        let filter = UrlFilter::default();

        // Text files should not be parsed for links
        let text_url = Url::parse("https://example.com/document.txt").unwrap();
        assert!(!filter.should_parse_links(&text_url));

        // YAML files should not be parsed for links
        let yaml_url = Url::parse("https://example.com/config.yaml").unwrap();
        assert!(!filter.should_parse_links(&yaml_url));

        // HTML files should be parsed for links
        let html_url = Url::parse("https://example.com/page.html").unwrap();
        assert!(filter.should_parse_links(&html_url));
    }
}
