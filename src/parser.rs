use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;

pub struct MarkdownParser {
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, content: &str) -> Result<String> {
        let mut html = String::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        let mut in_blockquote = false;
        let mut blockquote_content = String::new();

        while i < lines.len() {
            let line = lines[i].trim();

            if line.is_empty() && in_blockquote {
                html.push_str(&format!("<blockquote>{}</blockquote>\n", self.parse_inline(&blockquote_content)));
                blockquote_content.clear();
                in_blockquote = false;
                i += 1;
                continue;
            }

            match self.identify_element(line) {
                Element::Blockquote => {
                    in_blockquote = true;
                    blockquote_content.push_str(&line[2..]);
                    blockquote_content.push_str("<br>");
                    i += 1;
                }
                Element::HorizontalRule => {
                    html.push_str("<hr>\n");
                    i += 1;
                }
                Element::Heading(level) => {
                    html.push_str(&self.parse_heading(line, level));
                    i += 1;
                }
                Element::UnorderedList => {
                    let (parsed, consumed) = self.parse_list(&lines[i..], false);
                    html.push_str(&parsed);
                    i += consumed;
                }
                Element::OrderedList => {
                    let (parsed, consumed) = self.parse_list(&lines[i..], true);
                    html.push_str(&parsed);
                    i += consumed;
                }
                Element::CodeBlock => {
                    let (parsed, consumed) = self.parse_code_block(&lines[i..]);
                    html.push_str(&parsed);
                    i += consumed;
                }
                Element::Table => {
                    let (parsed, consumed) = self.parse_table(&lines[i..]);
                    html.push_str(&parsed);
                    i += consumed;
                }
                Element::Paragraph => {
                    if !line.is_empty() {
                        html.push_str(&self.parse_paragraph(line));
                    }
                    i += 1;
                }
                Element::MathBlock => {
                    let (parsed, consumed) = self.parse_math_block(&lines[i..]);
                    html.push_str(&parsed);
                    i += consumed;
                }
            }
        }

        if in_blockquote && !blockquote_content.is_empty() {
            html.push_str(&format!("<blockquote>{}</blockquote>\n", self.parse_inline(&blockquote_content)));
        }

        Ok(html)
    }

    fn identify_element(&self, line: &str) -> Element {
        lazy_static! {
            static ref HEADING_RE: Regex = Regex::new(r"^#{1,6}\s").unwrap();
            static ref UNORDERED_LIST_RE: Regex = Regex::new(r"^[\s]*[-*]\s").unwrap();
            static ref ORDERED_LIST_RE: Regex = Regex::new(r"^[\s]*\d+\.\s").unwrap();
            static ref CODE_BLOCK_RE: Regex = Regex::new(r"^```").unwrap();
            static ref TABLE_RE: Regex = Regex::new(r"^[|].*[|]$").unwrap();
            static ref BLOCKQUOTE_RE: Regex = Regex::new(r"^>\s").unwrap();
            static ref HORIZONTAL_RULE_RE: Regex = Regex::new(r"^-{3,}$|^_{3,}$|^\*{3,}$").unwrap();
            static ref MATH_BLOCK_RE: Regex = Regex::new(r"^\$\$").unwrap();
        }

        if let Some(captures) = HEADING_RE.find(line) {
            return Element::Heading(captures.as_str().trim().len());
        }

        if BLOCKQUOTE_RE.is_match(line) {
            return Element::Blockquote;
        }

        if HORIZONTAL_RULE_RE.is_match(line) {
            return Element::HorizontalRule;
        }

        if UNORDERED_LIST_RE.is_match(line) {
            return Element::UnorderedList;
        }

        if ORDERED_LIST_RE.is_match(line) {
            return Element::OrderedList;
        }

        if CODE_BLOCK_RE.is_match(line) {
            return Element::CodeBlock;
        }

        if TABLE_RE.is_match(line) {
            return Element::Table;
        }

        if MATH_BLOCK_RE.is_match(line) {
            return Element::MathBlock;
        }

        Element::Paragraph
    }

    fn parse_heading(&self, line: &str, level: usize) -> String {
        let content = line.trim_start_matches(|c| c == '#').trim();
        format!("<h{}>{}</h{}>\n", level, content, level)
    }

    fn parse_list(&self, lines: &[&str], ordered: bool) -> (String, usize) {
        let mut result = String::new();
        let mut consumed = 0;
        let mut current_indent = 0;
        let mut list_stack = vec![];

        let list_type = if ordered { "ol" } else { "ul" };
        result.push_str(&format!("<{}>\n", list_type));
        list_stack.push((list_type, 0));

        while consumed < lines.len() {
            let line = lines[consumed].trim_start();
            if line.is_empty() {
                break;
            }

            let indent = (lines[consumed].len() - line.len()) / 2 * 2;
            let is_list_item = if ordered {
                line.contains(". ")
            } else {
                line.contains("- ") || line.contains("* ")
            };

            if !is_list_item {
                break;
            }

            if indent > current_indent {
                let new_list_type = if ordered { "ol" } else { "ul" };
                result.push_str(&format!("<{}>\n", new_list_type));
                list_stack.push((new_list_type, indent));
                current_indent = indent;
            } else if indent < current_indent {
                while let Some((list_type, level)) = list_stack.last() {
                    if *level > indent {
                        result.push_str(&format!("</{}>\n", list_type));
                        list_stack.pop();
                        if let Some((_, prev_level)) = list_stack.last() {
                            current_indent = *prev_level;
                        } else {
                            current_indent = 0;
                        }
                    } else {
                        break;
                    }
                }
            }

            let content = if ordered {
                line.splitn(2, ". ").nth(1).unwrap_or("").trim()
            } else {
                line.splitn(2, |c| c == '-' || c == '*').nth(1).unwrap_or("").trim()
            };

            result.push_str(&format!("<li>{}</li>\n", self.parse_inline(content)));
            consumed += 1;
        }

        while let Some((list_type, _)) = list_stack.pop() {
            result.push_str(&format!("</{}>\n", list_type));
        }

        (result, consumed)
    }

    fn parse_code_block(&self, lines: &[&str]) -> (String, usize) {
        let mut result = String::new();
        let mut consumed = 0;
        let mut in_code_block = false;

        for line in lines {
            if line.starts_with("```") {
                if !in_code_block {
                    let language = line.trim_start_matches('`').trim().to_string();
                    if !language.is_empty() {
                        result.push_str(&format!(r#"<pre><code class="language-{}">"#, language));
                    } else {
                        result.push_str(r#"<pre><code>"#);
                    }
                    in_code_block = true;
                } else {
                    result.push_str("</code></pre>\n");
                    consumed += 1;
                    break;
                }
            } else if in_code_block {
                let escaped_line = line
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('"', "&quot;")
                    .replace('\'', "&#39;");
                result.push_str(&escaped_line);
                result.push('\n');
            }
            consumed += 1;
        }

        (result, consumed)
    }

    fn parse_paragraph(&self, line: &str) -> String {
        format!("<p>{}</p>\n", self.parse_inline(line))
    }

    fn parse_inline(&self, text: &str) -> String {
        lazy_static! {
            static ref BOLD_RE: Regex = Regex::new(r"\*\*(.+?)\*\*|__(.+?)__").unwrap();
            static ref ITALIC_RE: Regex = Regex::new(r"\*(.+?)\*|_(.+?)_").unwrap();
            static ref CODE_RE: Regex = Regex::new(r"`(.+?)`").unwrap();
            static ref LINK_RE: Regex = Regex::new(r"\[(.+?)\]\((.+?)\)").unwrap();
            static ref IMAGE_RE: Regex = Regex::new(r"!\[(.+?)\]\((.+?)\)").unwrap();
            static ref STRIKETHROUGH_RE: Regex = Regex::new(r"~~(.+?)~~").unwrap();
            static ref TASK_LIST_RE: Regex = Regex::new(r"^\[([x ])\](.+)$").unwrap();
            static ref INLINE_MATH_RE: Regex = Regex::new(r"\$([^$]+?)\$").unwrap();
        }

        let mut result = text.to_string();

        result = INLINE_MATH_RE.replace_all(&result, r#"<span class="math-inline">$$$1$$</span>"#).to_string();

        result = IMAGE_RE.replace_all(&result, r#"<img src="$2" alt="$1">"#).to_string();

        result = BOLD_RE.replace_all(&result, "<strong>$1$2</strong>").to_string();
        result = ITALIC_RE.replace_all(&result, "<em>$1$2</em>").to_string();
        result = CODE_RE.replace_all(&result, "<code>$1</code>").to_string();
        result = LINK_RE.replace_all(&result, r#"<a href="$2">$1</a>"#).to_string();
        result = STRIKETHROUGH_RE.replace_all(&result, "<del>$1</del>").to_string();

        result = TASK_LIST_RE.replace_all(&result, |caps: &regex::Captures| {
            let checked = caps[1].contains('x');
            format!(
                r#"<input type="checkbox" disabled{}>{}"#,
                if checked { " checked" } else { "" },
                &caps[2]
            )
        }).to_string();

        result
    }

    fn parse_table(&self, lines: &[&str]) -> (String, usize) {
        let mut result = String::from("<table>\n");
        let mut consumed = 0;

        if consumed < lines.len() {
            let cells = self.split_table_row(lines[consumed]);
            result.push_str("<thead>\n<tr>\n");
            for cell in cells {
                result.push_str(&format!("<th>{}</th>\n", self.parse_inline(cell.trim())));
            }
            result.push_str("</tr>\n</thead>\n");
            consumed += 1;
        }

        if consumed < lines.len() && lines[consumed].contains('|') && lines[consumed].contains('-') {
            consumed += 1;
        }

        result.push_str("<tbody>\n");
        while consumed < lines.len() {
            let line = lines[consumed];
            if !line.contains('|') {
                break;
            }

            let cells = self.split_table_row(line);
            result.push_str("<tr>\n");
            for cell in cells {
                result.push_str(&format!("<td>{}</td>\n", self.parse_inline(cell.trim())));
            }
            result.push_str("</tr>\n");
            consumed += 1;
        }
        result.push_str("</tbody>\n</table>\n");

        (result, consumed)
    }

    fn split_table_row<'a>(&self, line: &'a str) -> Vec<&'a str> {
        line.trim()
            .trim_matches('|')
            .split('|')
            .collect()
    }

    fn parse_math_block(&self, lines: &[&str]) -> (String, usize) {
        let mut result = String::new();
        let mut consumed = 0;
        let mut in_math_block = false;
        let mut math_content = String::new();

        for line in lines {
            if line.trim() == "$$" {
                if !in_math_block {
                    in_math_block = true;
                } else {
                    result.push_str(r#"<div class="math-block">$$"#);
                    result.push_str(&math_content.trim());
                    result.push_str("$$</div>\n");
                    consumed += 1;
                    break;
                }
            } else if in_math_block {
                math_content.push_str(line);
                math_content.push('\n');
            }
            consumed += 1;
        }

        (result, consumed)
    }
}

enum Element {
    Heading(usize),
    UnorderedList,
    OrderedList,
    CodeBlock,
    Table,
    Paragraph,
    Blockquote,
    HorizontalRule,
    MathBlock,
}

#[cfg(test)]
mod tests {
    use super::*;

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


        let input = "- Item 1\n- Item 2\n  - Nested item\n- Item 3";
        let result = parser.parse(input).unwrap();
        assert!(result.contains("<ul>"));
        assert!(result.contains("<li>Item 1</li>"));
        assert!(result.contains("<li>Nested item</li>"));


        let input = "1. First\n2. Second\n   1. Nested\n3. Third";
        let result = parser.parse(input).unwrap();
        assert!(result.contains("<ol>"));
        assert!(result.contains("<li>First</li>"));
        assert!(result.contains("<li>Nested</li>"));


        let input = "- Item 1\n  1. Nested ordered\n  2. Another ordered\n- Item 2";
        let result = parser.parse(input).unwrap();
        assert!(result.contains("<ul>"));
        assert!(result.contains("<ol>"));
        assert!(result.contains("<li>Nested ordered</li>"));
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
