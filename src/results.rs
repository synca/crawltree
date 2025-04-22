use serde::{Deserialize, Serialize};

/// Represents a discovered page with its URL and content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageData {
    /// URL of the page
    pub url: String,

    /// Title of the page (if available)
    pub title: Option<String>,

    /// Extracted text content
    pub content: String,

    /// Links discovered on the page (as strings)
    pub links: Vec<String>,
}

impl PageData {
    /// Create a new page data instance
    pub fn new(url: String, title: Option<String>, content: String, links: Vec<String>) -> Self {
        Self {
            url,
            title,
            content,
            links,
        }
    }
}
