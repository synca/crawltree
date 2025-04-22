use crate::parsers::text;
use crate::parsers::text::TextParserOptions;

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_empty_text() {
        let result = text::parse("");
        assert_eq!(result.content, "");
        assert!(result.links.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let result = text::parse("   \n   \t   \r\n   ");
        assert_eq!(result.content, "");
        assert!(result.links.is_empty());
    }

    #[test]
    fn test_single_line() {
        let result = text::parse("Hello, world!");
        assert_eq!(result.content, "Hello, world!");
        assert!(result.links.is_empty());
    }

    #[test]
    fn test_multiple_lines() {
        let input = "Line 1\nLine 2\nLine 3";
        let result = text::parse(input);
        assert_eq!(result.content, "Line 1 Line 2 Line 3");
        assert!(result.links.is_empty());
    }

    #[test]
    fn test_mixed_whitespace() {
        let input = "  Line 1  \n\n  Line 2  \t\r\n  Line 3  ";
        let result = text::parse(input);
        assert_eq!(result.content, "Line 1 Line 2 Line 3");
        assert!(result.links.is_empty());
    }

    #[test]
    fn test_preserve_punctuation() {
        let input = "Hello, world! This is a test.";
        let result = text::parse(input);
        assert_eq!(result.content, "Hello, world! This is a test.");
        assert!(result.links.is_empty());
    }

    #[test]
    fn test_multiple_spaces_between_words() {
        let input = "Hello    world!    This    is    a    test.";
        let result = text::parse(input);
        assert_eq!(result.content, "Hello world! This is a test.");
        assert!(result.links.is_empty());
    }

    #[test]
    fn test_blank_lines_between_content() {
        let input = "Paragraph 1.\n\n\nParagraph 2.\n\nParagraph 3.";
        let result = text::parse(input);
        assert_eq!(result.content, "Paragraph 1. Paragraph 2. Paragraph 3.");
        assert!(result.links.is_empty());
    }

    #[test]
    fn test_urls_in_text() {
        let input = "Check out https://example.com for more information.\nOr visit http://test.org/page.html";
        let result = text::parse(input);
        assert_eq!(
            result.content,
            "Check out https://example.com for more information. Or visit http://test.org/page.html"
        );
        // The text parser should not extract links, even if URLs are present in the text
        assert!(result.links.is_empty());
    }

    #[test]
    fn test_long_text() {
        let mut input = String::new();
        for i in 1..100 {
            input.push_str(&format!("Line {} with some text.\n", i));
        }

        let result = text::parse(&input);

        // Verify that all lines are preserved (but newlines are replaced with spaces)
        for i in 1..100 {
            let expected_fragment = format!("Line {} with some text.", i);
            assert!(result.content.contains(&expected_fragment));
        }

        // Ensure no extra spaces between lines
        assert!(!result.content.contains("  "));

        // No links should be extracted
        assert!(result.links.is_empty());
    }
}

#[cfg(test)]
mod advanced_options_tests {
    use super::*;

    #[test]
    fn test_preserve_paragraphs() {
        let input = "Paragraph 1.\n\nParagraph 2.\n\n\n\nParagraph 3.";

        // With default options (paragraphs not preserved)
        let result = text::parse(input);
        assert_eq!(result.content, "Paragraph 1. Paragraph 2. Paragraph 3.");

        // With paragraph preservation - should have exactly one empty line between paragraphs
        let options = TextParserOptions {
            preserve_paragraphs: true,
            ..TextParserOptions::default()
        };
        let result = text::parse_with_options(input, &options);
        assert_eq!(
            result.content,
            "Paragraph 1.\n\nParagraph 2.\n\nParagraph 3."
        );
    }

    #[test]
    fn test_preserve_line_breaks() {
        let input = "Line 1\nLine 2\nLine 3";

        // With default options (line breaks not preserved)
        let result = text::parse(input);
        assert_eq!(result.content, "Line 1 Line 2 Line 3");

        // With line break preservation
        let options = TextParserOptions {
            preserve_line_breaks: true,
            ..TextParserOptions::default()
        };
        let result = text::parse_with_options(input, &options);
        assert_eq!(result.content, "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_both_paragraphs_and_line_breaks() {
        let input = "Line 1a\nLine 1b\n\n\n\nLine 2a\nLine 2b";

        // Preserve both paragraphs and line breaks
        let options = TextParserOptions {
            preserve_paragraphs: true,
            preserve_line_breaks: true,
            ..TextParserOptions::default()
        };
        let result = text::parse_with_options(input, &options);

        // Should normalize multiple blank lines to a single blank line
        assert_eq!(result.content, "Line 1a\nLine 1b\n\nLine 2a\nLine 2b");
    }

    #[test]
    fn test_no_whitespace_normalization() {
        let input = "Hello    world!    How   are   you?";

        // With default options (whitespace normalized)
        let result = text::parse(input);
        assert_eq!(result.content, "Hello world! How are you?");

        // Without whitespace normalization
        let options = TextParserOptions {
            normalize_whitespace: false,
            ..TextParserOptions::default()
        };
        let result = text::parse_with_options(input, &options);
        assert_eq!(result.content, "Hello    world!    How   are   you?");
    }

    #[test]
    fn test_more_complex_document() {
        let input = "
            # Document Title
            
            
            This is the first paragraph with multiple
            lines that should be joined together.
            
            ## Section 1
            
            
            - List item 1
            - List item 2
            
            
            
            Check out https://example.com for more information.
        ";

        // Parse with various options
        let options_standard = TextParserOptions::default();
        let result_standard = text::parse_with_options(input, &options_standard);

        let options_preserve_breaks = TextParserOptions {
            preserve_line_breaks: true,
            ..TextParserOptions::default()
        };
        let result_breaks = text::parse_with_options(input, &options_preserve_breaks);

        let options_preserve_paragraphs = TextParserOptions {
            preserve_paragraphs: true,
            ..TextParserOptions::default()
        };
        let result_paragraphs = text::parse_with_options(input, &options_preserve_paragraphs);

        // Verify that all important content is preserved in all parsing modes
        for result in [&result_standard, &result_breaks, &result_paragraphs] {
            assert!(result.content.contains("Document Title"));
            assert!(result.content.contains("first paragraph"));
            assert!(result.content.contains("Section 1"));
            assert!(result.content.contains("List item 1"));
            assert!(result.content.contains("List item 2"));
            assert!(result.content.contains("https://example.com"));
        }

        // Verify specific formatting for each mode
        assert!(!result_standard.content.contains("\n")); // No newlines in standard mode
        assert!(result_breaks.content.contains("\n")); // Should have newlines in breaks mode
        assert!(result_paragraphs.content.contains("\n\n")); // Should have paragraph breaks

        // In paragraph mode, there should never be more than one empty line
        assert!(!result_paragraphs.content.contains("\n\n\n"));
    }
}
