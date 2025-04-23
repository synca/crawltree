use crate::parsers::text;
use crate::parsers::text::TextParserOptions;

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_split_into_paragraphs() {
        // Empty text
        let result = text::split_into_paragraphs("");
        assert_eq!(result.len(), 0);

        // Single line
        let result = text::split_into_paragraphs("Hello world");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["Hello world"]);

        // Multiple lines, single paragraph
        let result = text::split_into_paragraphs("Line 1\nLine 2\nLine 3");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec!["Line 1", "Line 2", "Line 3"]);

        // Multiple paragraphs
        let result = text::split_into_paragraphs("Paragraph 1.\n\nParagraph 2.\n\nParagraph 3.");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], vec!["Paragraph 1."]);
        assert_eq!(result[1], vec!["Paragraph 2."]);
        assert_eq!(result[2], vec!["Paragraph 3."]);

        // Multiple lines per paragraph
        let result = text::split_into_paragraphs("Line 1a\nLine 1b\n\nLine 2a\nLine 2b");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Line 1a", "Line 1b"]);
        assert_eq!(result[1], vec!["Line 2a", "Line 2b"]);

        // Multiple consecutive empty lines
        let result = text::split_into_paragraphs("Paragraph 1.\n\n\n\nParagraph 2.");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec!["Paragraph 1."]);
        assert_eq!(result[1], vec!["Paragraph 2."]);
    }

    #[test]
    fn test_process_paragraph() {
        let paragraph = vec!["Line 1", "Line 2", "Line 3"];

        // Default options (no line break preservation)
        let options = TextParserOptions::default();
        let result = text::process_paragraph(&paragraph, &options);
        assert_eq!(result, "Line 1 Line 2 Line 3");

        // With line break preservation
        let options = TextParserOptions {
            preserve_line_breaks: true,
            ..TextParserOptions::default()
        };
        let result = text::process_paragraph(&paragraph, &options);
        assert_eq!(result, "Line 1\nLine 2\nLine 3");

        // Empty paragraph
        let result = text::process_paragraph(&Vec::<&str>::new(), &options);
        assert_eq!(result, "");
    }

    #[test]
    fn test_join_paragraphs() {
        let paragraphs = vec![
            "Paragraph 1.".to_string(),
            "Paragraph 2.".to_string(),
            "Paragraph 3.".to_string(),
        ];

        // Default options (no paragraph preservation)
        let options = TextParserOptions::default();
        let result = text::join_paragraphs(&paragraphs, &options);
        assert_eq!(result, "Paragraph 1. Paragraph 2. Paragraph 3.");

        // With paragraph preservation
        let options = TextParserOptions {
            preserve_paragraphs: true,
            ..TextParserOptions::default()
        };
        let result = text::join_paragraphs(&paragraphs, &options);
        assert_eq!(result, "Paragraph 1.\n\nParagraph 2.\n\nParagraph 3.");

        // Empty paragraphs array
        let result = text::join_paragraphs(&Vec::<String>::new(), &options);
        assert_eq!(result, "");
    }

    #[test]
    fn test_normalize_whitespace() {
        // Simple text with extra spaces
        let text = "Hello   world!   This   is   a   test.";
        let options = TextParserOptions::default();
        let result = text::normalize_whitespace(text, &options);
        assert_eq!(result, "Hello world! This is a test.");

        // Paragraph with different whitespace settings
        let text = "Paragraph 1.\n\nParagraph 2.  \n\n  Paragraph 3.";

        // With no paragraph preservation
        let options = TextParserOptions::default();
        let result = text::normalize_whitespace(text, &options);
        assert_eq!(result, "Paragraph 1. Paragraph 2. Paragraph 3.");

        // With paragraph preservation
        let options = TextParserOptions {
            preserve_paragraphs: true,
            ..TextParserOptions::default()
        };
        let result = text::normalize_whitespace(text, &options);
        assert_eq!(result, "Paragraph 1.\n\nParagraph 2.\n\nParagraph 3.");

        // With line break preservation
        let text = "Line 1  \nLine 2  \nLine 3";
        let options = TextParserOptions {
            preserve_line_breaks: true,
            ..TextParserOptions::default()
        };
        let result = text::normalize_whitespace(text, &options);
        assert_eq!(result, "Line 1\nLine 2\nLine 3");

        // With whitespace normalization disabled
        let options = TextParserOptions {
            normalize_whitespace: false,
            ..TextParserOptions::default()
        };
        let result = text::normalize_whitespace("Hello   world!", &options);
        assert_eq!(result, "Hello   world!");
    }

    #[test]
    fn test_normalize_whitespace_in_segment() {
        // Basic whitespace normalization
        assert_eq!(
            text::normalize_whitespace_in_segment("Hello   world!"),
            "Hello world!"
        );
        assert_eq!(
            text::normalize_whitespace_in_segment("  Trim  me  "),
            "Trim me"
        );
        assert_eq!(
            text::normalize_whitespace_in_segment("Tabs\tand\tspaces"),
            "Tabs and spaces"
        );

        // Empty and whitespace-only inputs
        assert_eq!(text::normalize_whitespace_in_segment(""), "");
        assert_eq!(text::normalize_whitespace_in_segment("   "), "");
    }
}
