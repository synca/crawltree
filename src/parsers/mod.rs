pub mod html;
pub mod text;

#[cfg(test)]
mod tests;

/// Enum to represent different types of content parsers
#[derive(Debug, Clone, Copy)]
pub enum ParserType {
    /// HTML parser
    Html,
    /// Plain text parser
    Text,
    /// PDF parser (placeholder for future implementation)
    Pdf,
    /// Other formats (placeholder for future implementation)
    Other,
}

impl ParserType {
    /// Determines the parser type based on the URL or file path
    pub fn from_url(url: &str) -> Self {
        // Text files should not be parsed for links
        if url.ends_with(".txt") {
            ::log::debug!("Classifying as Text: {}", url);
            ParserType::Text
        } else if url.ends_with(".yaml") || url.ends_with(".yml") {
            ::log::debug!("Classifying as Text (YAML): {}", url);
            ParserType::Text
        } else if url.ends_with(".pdf") {
            ::log::debug!("Classifying as PDF: {}", url);
            ParserType::Pdf
        } else if url.contains("/_sources/") {
            // Special rule from UrlFilter.should_parse_links
            ::log::debug!("Classifying as Text (_sources): {}", url);
            ParserType::Text
        } else if url.ends_with(".jpg")
            || url.ends_with(".jpeg")
            || url.ends_with(".png")
            || url.ends_with(".gif")
            || url.ends_with(".css")
            || url.ends_with(".js")
        {
            // Media files and web assets should be classified as Other
            ::log::debug!("Classifying as Other: {}", url);
            ParserType::Other
        } else {
            // Default to HTML for most URLs
            ::log::debug!("Classifying as HTML: {}", url);
            ParserType::Html
        }
    }

    /// Returns if the parser should extract links
    pub fn should_extract_links(&self) -> bool {
        matches!(self, ParserType::Html)
    }
}

/// Result of parsing content
pub struct ParseResult {
    /// Extracted text content
    pub content: String,
    /// Extracted links (if applicable)
    pub links: Vec<String>,
}

impl ParseResult {
    /// Creates a new parse result with the given content and links
    pub fn new(content: String, links: Vec<String>) -> Self {
        Self { content, links }
    }

    /// Creates a new parse result with content only (no links)
    pub fn content_only(content: String) -> Self {
        Self {
            content,
            links: Vec::new(),
        }
    }
}

/// Main parser that delegates to specific format parsers
pub struct Parser;

impl Parser {
    /// Parse content based on the parser type
    pub fn parse(content: &str, parser_type: ParserType) -> ParseResult {
        match parser_type {
            ParserType::Html => html::parse(content),
            ParserType::Text => text::parse(content),
            ParserType::Pdf => {
                // Placeholder for PDF parsing (not implemented yet)
                ParseResult::content_only("PDF parsing not implemented yet".to_string())
            }
            ParserType::Other => {
                // Default handling for unknown formats - just treat as plain text
                text::parse(content)
            }
        }
    }

    /// Parse content with specific text parser options
    pub fn parse_with_text_options(
        content: &str,
        parser_type: ParserType,
        text_options: &text::TextParserOptions,
    ) -> ParseResult {
        match parser_type {
            ParserType::Html => html::parse(content),
            ParserType::Text => text::parse_with_options(content, text_options),
            ParserType::Pdf => {
                // Placeholder for PDF parsing (not implemented yet)
                ParseResult::content_only("PDF parsing not implemented yet".to_string())
            }
            ParserType::Other => {
                // Default handling for unknown formats - just treat as plain text
                text::parse_with_options(content, text_options)
            }
        }
    }

    /// Determine parser type from URL and then parse content
    pub fn parse_from_url(content: &str, url: &str) -> ParseResult {
        let parser_type = ParserType::from_url(url);
        Self::parse(content, parser_type)
    }

    /// Determine parser type from URL and then parse content with text options
    pub fn parse_from_url_with_text_options(
        content: &str,
        url: &str,
        text_options: &text::TextParserOptions,
    ) -> ParseResult {
        let parser_type = ParserType::from_url(url);
        Self::parse_with_text_options(content, parser_type, text_options)
    }
}
