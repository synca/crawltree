use crate::parsers::{Parser, ParserType, text::TextParserOptions};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_parser_type() {
        // Test HTML
        let html_content = "<html><body><p>Hello, world!</p><a href=\"https://example.com\">Link</a></body></html>";
        let result = Parser::parse(html_content, ParserType::Html);
        assert_eq!(result.content, "Hello, world! Link");
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.links[0], "https://example.com");

        // Test Text
        let text_content = "Line 1\nLine 2\nLine 3";
        let result = Parser::parse(text_content, ParserType::Text);
        assert_eq!(result.content, "Line 1 Line 2 Line 3");
        assert_eq!(result.links.len(), 0);
    }

    #[test]
    fn test_parse_from_url() {
        // Test HTML URL
        let html_content = "<html><body><p>Hello, world!</p><a href=\"https://example.com\">Link</a></body></html>";
        let result = Parser::parse_from_url(html_content, "https://example.org/page");
        assert_eq!(result.content, "Hello, world! Link");
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.links[0], "https://example.com");

        // Test Text URL
        let text_content = "Line 1\nLine 2\nLine 3";
        let result = Parser::parse_from_url(text_content, "https://example.org/file.txt");
        assert_eq!(result.content, "Line 1 Line 2 Line 3");
        assert_eq!(result.links.len(), 0);
    }

    #[test]
    fn test_parse_with_text_options() {
        // Test text parsing with options
        let text_content = "Paragraph 1.\n\n\n\nParagraph 2.\n\n\nParagraph 3.";

        // Default options
        let result = Parser::parse(text_content, ParserType::Text);
        assert_eq!(result.content, "Paragraph 1. Paragraph 2. Paragraph 3.");

        // Preserve paragraphs - should normalize to exactly one blank line
        let options = TextParserOptions {
            preserve_paragraphs: true,
            ..TextParserOptions::default()
        };
        let result = Parser::parse_with_text_options(text_content, ParserType::Text, &options);
        assert_eq!(
            result.content,
            "Paragraph 1.\n\nParagraph 2.\n\nParagraph 3."
        );
    }

    #[test]
    fn test_parse_from_url_with_text_options() {
        // Test URL-based parsing with text options
        let text_content = "Line 1\nLine 2\nLine 3";

        // Default options
        let result = Parser::parse_from_url(text_content, "https://example.org/file.txt");
        assert_eq!(result.content, "Line 1 Line 2 Line 3");

        // Preserve line breaks
        let options = TextParserOptions {
            preserve_line_breaks: true,
            ..TextParserOptions::default()
        };
        let result = Parser::parse_from_url_with_text_options(
            text_content,
            "https://example.org/file.txt",
            &options,
        );
        assert_eq!(result.content, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_different_file_extensions() {
        // Check that different file extensions are correctly mapped to parser types
        let types = [
            ("file.txt", ParserType::Text),
            ("file.html", ParserType::Html),
            ("file.yml", ParserType::Text),
            ("file.pdf", ParserType::Pdf),
            ("file.jpg", ParserType::Other), // Image should be "other"
            ("https://example.org/page", ParserType::Html), // URL without extension should be HTML
        ];

        for (url, expected_type) in types {
            let parser_type = ParserType::from_url(url);
            assert_eq!(
                parser_type as u8, expected_type as u8,
                "URL '{}' should be parsed as {:?}",
                url, expected_type
            );
        }
    }
}
