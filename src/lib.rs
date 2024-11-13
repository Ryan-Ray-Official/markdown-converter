pub mod converter;
pub mod error;

pub mod parser;

pub mod utils;

#[cfg(test)]
mod tests {
    use crate::parser::MarkdownParser;

    const EXAMPLE_MD: &str = include_str!("../example.md");

    #[test]
    fn test_example_markdown() {
        let parser = MarkdownParser::new();
        let result = parser.parse(EXAMPLE_MD).unwrap();

        assert!(result.contains("<h1>Markdown Test File</h1>"));
        assert!(result.contains("<h2>Text Formatting</h2>"));
        assert!(result.contains("<h6>Level 6 Heading</h6>"));

        assert!(result.contains("<strong>Bold text</strong>"));
        assert!(result.contains("<em>Italic text</em>"));
        assert!(result.contains("<del>Strikethrough text</del>"));
        assert!(result.contains("<code>Inline code</code>"));

        assert!(result.contains("<ul>"));
        assert!(result.contains("<ol>"));
        assert!(result.contains("<li>Subitem 1.1</li>"));
        assert!(result.contains("<li>First item</li>"));

        assert!(result.contains(r#"<a href="https://www.rust-lang.org/""#));
        assert!(result.contains("<img src="));

        assert!(result.contains("<table>"));
        assert!(result.contains("<th>Syntax</th>"));
        assert!(result.contains("<td>Text</td>"));

        assert!(result.contains("<blockquote>"));
        assert!(result.contains("This is a blockquote"));

        assert!(result.contains(r#"<pre><code class="language-rust">"#));
        assert!(result.contains("println!("));

        assert!(result.contains("<hr>"));

        assert!(result.contains(r#"<div class="math-block">"#));
        assert!(result.contains("\\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}"));
        assert!(result.contains("\\nabla \\times \\vec{\\mathbf{B}}"));
    }
}
