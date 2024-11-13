#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::MarkdownParser;
    use std::path::PathBuf;

    #[test]
    fn test_basic_markdown_parsing() {
        let parser = MarkdownParser::new();
        let input = "# Test Heading\n\nThis is a paragraph.";
        let result = parser.parse(input).unwrap();
        assert!(result.contains("<h1>Test Heading</h1>"));
        assert!(result.contains("<p>This is a paragraph.</p>"));
    }

    #[test]
    fn test_inline_formatting() {
        let parser = MarkdownParser::new();
        let input = "**Bold** and *italic* text";
        let result = parser.parse(input).unwrap();
        assert!(result.contains("<strong>Bold</strong>"));
        assert!(result.contains("<em>italic</em>"));
    }

    #[test]
    fn test_lists() {
        let parser = MarkdownParser::new();
        let input = "- Item 1\n- Item 2\n\n1. First\n2. Second";
        let result = parser.parse(input).unwrap();
        assert!(result.contains("<ul>"));
        assert!(result.contains("<ol>"));
        assert!(result.contains("<li>Item 1</li>"));
        assert!(result.contains("<li>First</li>"));
    }

    #[test]
    fn test_code_blocks() {
        let parser = MarkdownParser::new();
        let input = "```rust\nfn main() {}\n```";
        let result = parser.parse(input).unwrap();
        assert!(result.contains(r#"<pre><code class="language-rust">"#));
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_blockquotes() {
        let parser = MarkdownParser::new();
        let input = "> This is a quote\n> Second line";
        let result = parser.parse(input).unwrap();
        assert!(result.contains("<blockquote>"));
        assert!(result.contains("This is a quote"));
    }

    #[test]
    fn test_math() {
        let parser = MarkdownParser::new();
        let input = "$$\nE = mc^2\n$$";
        let result = parser.parse(input).unwrap();
        assert!(result.contains(r#"<div class="math-block">"#));
        assert!(result.contains("E = mc^2"));
    }
}
