use crate::parsers::ParseResult;

/// Configuration options for text parsing
#[derive(Debug, Clone, Copy)]
pub struct TextParserOptions {
    /// Whether to preserve paragraph structure (insert double newlines between paragraphs)
    pub preserve_paragraphs: bool,
    /// Whether to preserve single newlines within paragraphs
    pub preserve_line_breaks: bool,
    /// Whether to collapse multiple spaces into a single space
    pub normalize_whitespace: bool,
    /// Whether to detect URL-like patterns in text
    /// (doesn't extract as links, just prevents breaking URLs)
    pub detect_urls: bool,
}

impl Default for TextParserOptions {
    fn default() -> Self {
        Self {
            preserve_paragraphs: false,
            preserve_line_breaks: false,
            normalize_whitespace: true,
            detect_urls: true,
        }
    }
}

/// Parses plain text content (no links) with default options
///
/// This function normalizes text content by:
/// - Trimming whitespace from each line
/// - Removing empty lines
/// - Replacing newlines with spaces
/// - Normalizing multiple spaces into single spaces
/// - Preserving punctuation and URLs
pub fn parse(text: &str) -> ParseResult {
    parse_with_options(text, &TextParserOptions::default())
}

/// Parses text with specific options
pub fn parse_with_options(text: &str, options: &TextParserOptions) -> ParseResult {
    // Handle empty input
    if text.trim().is_empty() {
        return ParseResult::content_only(String::new());
    }

    let paragraphs = split_into_paragraphs(text);
    let processed_paragraphs = process_paragraphs(&paragraphs, options);
    let result = join_paragraphs(&processed_paragraphs, options);

    ParseResult::content_only(result)
}

//
// Core text processing functions
//

/// Splits text into paragraphs based on empty lines
pub fn split_into_paragraphs(text: &str) -> Vec<Vec<&str>> {
    let lines: Vec<&str> = text.lines().collect();
    let mut paragraphs: Vec<Vec<&str>> = Vec::new();
    let mut current_paragraph: Vec<&str> = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            // Found an empty line, which marks a paragraph boundary
            if !current_paragraph.is_empty() {
                paragraphs.push(current_paragraph);
                current_paragraph = Vec::new();
            }
        } else {
            // Non-empty line goes into the current paragraph
            current_paragraph.push(trimmed);
        }
    }

    // Add the last paragraph if it's not empty
    if !current_paragraph.is_empty() {
        paragraphs.push(current_paragraph);
    }

    paragraphs
}

/// Processes paragraphs according to options
pub fn process_paragraphs(paragraphs: &[Vec<&str>], options: &TextParserOptions) -> Vec<String> {
    paragraphs
        .iter()
        .map(|para| process_paragraph(para, options))
        .collect()
}

/// Processes a single paragraph according to options
pub fn process_paragraph(paragraph: &[&str], options: &TextParserOptions) -> String {
    if paragraph.is_empty() {
        return String::new();
    }

    if options.preserve_line_breaks {
        // Join the lines with newlines
        paragraph.join("\n")
    } else {
        // Join the lines with spaces
        paragraph.join(" ")
    }
}

/// Joins processed paragraphs into a single string
pub fn join_paragraphs(paragraphs: &[String], options: &TextParserOptions) -> String {
    if paragraphs.is_empty() {
        return String::new();
    }

    let result = if options.preserve_paragraphs {
        // Join paragraphs with double newlines
        paragraphs.join("\n\n")
    } else {
        // Join paragraphs with spaces
        paragraphs.join(" ")
    };

    normalize_whitespace(&result, options)
}

/// Normalizes whitespace in the text according to options
pub fn normalize_whitespace(text: &str, options: &TextParserOptions) -> String {
    if !options.normalize_whitespace {
        return text.to_string();
    }

    if !options.preserve_paragraphs && !options.preserve_line_breaks {
        // If not preserving any structure, normalize all whitespace
        return text.split_whitespace().collect::<Vec<_>>().join(" ");
    }

    if options.preserve_paragraphs && !options.preserve_line_breaks {
        // If preserving paragraphs but not line breaks,
        // normalize whitespace within paragraphs only
        let paragraphs = text.split("\n\n").collect::<Vec<_>>();
        let normalized_paragraphs = paragraphs
            .iter()
            .map(|para| {
                // Normalize whitespace within each paragraph
                normalize_whitespace_in_segment(para)
            })
            .collect::<Vec<_>>();

        return normalized_paragraphs.join("\n\n");
    }

    if options.preserve_line_breaks {
        // If preserving line breaks, normalize whitespace within each line
        let lines = text.lines().collect::<Vec<_>>();
        let normalized_lines = lines
            .iter()
            .map(|line| {
                if line.trim().is_empty() {
                    // Preserve empty lines exactly
                    line.to_string()
                } else {
                    // Normalize whitespace within each line
                    normalize_whitespace_in_segment(line)
                }
            })
            .collect::<Vec<_>>();

        return normalized_lines.join("\n");
    }

    // Default fallback
    text.to_string()
}

/// Normalizes whitespace within a single line or paragraph
pub fn normalize_whitespace_in_segment(segment: &str) -> String {
    segment.split_whitespace().collect::<Vec<_>>().join(" ")
}
