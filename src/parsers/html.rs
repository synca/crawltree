use crate::parsers::ParseResult;
use scraper::{Html, Selector};

/// Parses HTML content to extract text and links
pub fn parse(html: &str) -> ParseResult {
    let doc = Html::parse_document(html);

    // Extract text content
    let content_selector = Selector::parse("body").unwrap();
    let text = doc
        .select(&content_selector)
        .flat_map(|n| n.text())
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    // Extract links
    let link_selector = Selector::parse("a").unwrap();
    let links = doc
        .select(&link_selector)
        .filter_map(|e| e.value().attr("href"))
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    // Log the number of links found
    ::log::debug!("HTML parser found {} links", links.len());
    if !links.is_empty() {
        ::log::debug!(
            "First few links: {:?}",
            links.iter().take(5).collect::<Vec<_>>()
        );
    }

    ParseResult::new(text, links)
}

/// Parses HTML content but only extracts text (no links)
pub fn parse_text_only(html: &str) -> ParseResult {
    let doc = Html::parse_document(html);

    // Extract text content
    let content_selector = Selector::parse("body").unwrap();
    let text = doc
        .select(&content_selector)
        .flat_map(|n| n.text())
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    ParseResult::content_only(text)
}

/// Parses HTML content and only extracts links (no text)
pub fn parse_links_only(html: &str) -> Vec<String> {
    let doc = Html::parse_document(html);

    // Extract links
    let link_selector = Selector::parse("a").unwrap();
    doc.select(&link_selector)
        .filter_map(|e| e.value().attr("href"))
        .map(|s| s.to_string())
        .collect()
}
