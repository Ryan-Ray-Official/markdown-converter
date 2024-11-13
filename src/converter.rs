use crate::error::ConverterError;
use crate::parser::MarkdownParser;
use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use log::{error, info};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

pub struct MarkdownConverter {
    input_path: PathBuf,
    output_path: PathBuf,
    css_path: Option<PathBuf>,
    syntax_highlight: bool,
    generate_toc: bool,
    minify: bool,
}

impl MarkdownConverter {
    pub fn new(
        input_path: PathBuf,
        output_path: Option<PathBuf>,
        css_path: Option<PathBuf>,
        syntax_highlight: bool,
        generate_toc: bool,
        minify: bool,
    ) -> Result<Self> {
        let output_path = output_path.unwrap_or_else(|| {
            let mut output = input_path.clone();
            output.set_extension("html");
            output
        });

        Ok(Self {
            input_path,
            output_path,
            css_path,
            syntax_highlight,
            generate_toc,
            minify,
        })
    }

    pub fn convert(&self) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Converting markdown to HTML...");

        let content =
            fs::read_to_string(&self.input_path).map_err(ConverterError::InputFileError)?;

        let parser = MarkdownParser::new();
        let mut html = parser.parse(&content)?;

        if self.syntax_highlight {
            html = self.apply_syntax_highlighting(&html)?;
        }

        if self.generate_toc {
            html = self.generate_table_of_contents(&html)?;
        }

        html = self.add_css(&html)?;

        if self.minify {
            html = self.minify_html(&html);
        }

        fs::write(&self.output_path, html).map_err(|e| {
            ConverterError::OutputFileError(format!("Failed to write output: {}", e))
        })?;

        pb.finish_with_message("Conversion completed successfully!");
        info!("Output saved to: {:?}", self.output_path);

        Ok(())
    }

    pub fn watch(&self) -> Result<()> {
        info!("Watching for changes in {:?}", self.input_path);

        let (tx, rx) = channel();
        let config = Config::default();
        let mut watcher: RecommendedWatcher = Watcher::new(tx, config)?;
        watcher.watch(&self.input_path, RecursiveMode::NonRecursive)?;

        println!("{}", "Watching for changes (Ctrl+C to stop)...".green());

        loop {
            match rx.recv() {
                Ok(_event) => {
                    println!("File changed, converting...");
                    if let Err(e) = self.convert() {
                        error!("Error converting file: {}", e);
                    }
                }
                Err(e) => error!("Watch error: {}", e),
            }
        }
    }

    fn apply_syntax_highlighting(&self, html: &str) -> Result<String> {
        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();
        let theme = &ts.themes["base16-ocean.dark"];

        let code_block_regex =
            Regex::new(r#"<pre><code class="language-(\w+)">([\s\S]*?)</code></pre>"#).unwrap();

        let html = code_block_regex.replace_all(html, |caps: &regex::Captures| {
            let lang = &caps[1];
            let code = &caps[2];

            if let Some(syntax) = ss.find_syntax_by_token(lang) {
                match highlighted_html_for_string(code, &ss, syntax, theme) {
                    Ok(highlighted) => highlighted,
                    Err(_) => caps[0].to_string(),
                }
            } else {
                caps[0].to_string()
            }
        });

        Ok(html.to_string())
    }

    fn generate_table_of_contents(&self, html: &str) -> Result<String> {
        lazy_static! {
            static ref HEADING_RE: Regex = Regex::new(r"<h([1-6])>(.*?)</h[1-6]>").unwrap();
        }

        let mut toc =
            String::from("<div class=\"table-of-contents\">\n<h2>Table of Contents</h2>\n<ul>\n");
        let mut current_level = 1;

        for cap in HEADING_RE.captures_iter(html) {
            let level = cap[1].parse::<i32>().unwrap();
            let text = &cap[2];
            let id = text.to_lowercase().replace(' ', "-");

            while level > current_level {
                toc.push_str("<ul>\n");
                current_level += 1;
            }
            while level < current_level {
                toc.push_str("</ul>\n");
                current_level -= 1;
            }

            toc.push_str(&format!("<li><a href=\"#{id}\">{text}</a></li>\n"));
        }

        while current_level > 1 {
            toc.push_str("</ul>\n");
            current_level -= 1;
        }

        toc.push_str("</ul>\n</div>\n");

        let html = HEADING_RE.replace_all(html, |caps: &regex::Captures| {
            let level = &caps[1];
            let text = &caps[2];
            let id = text.to_lowercase().replace(' ', "-");
            format!("<h{level} id=\"{id}\">{text}</h{level}>")
        });

        Ok(format!("{}{}", toc, html))
    }

    fn add_css(&self, html: &str) -> Result<String> {
        let css = if let Some(css_path) = &self.css_path {
            fs::read_to_string(css_path)
                .map_err(|e| ConverterError::CssError(format!("Failed to read CSS file: {}", e)))?
        } else {
            include_str!("../assets/default.css").to_string()
        };

        Ok(format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css">
    <script src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/contrib/auto-render.min.js"></script>
    <style>
        {}
    </style>
    <script>
        document.addEventListener("DOMContentLoaded", function() {{
            renderMathInElement(document.body, {{
                delimiters: [
                    {{left: "$$", right: "$$", display: true}},
                    {{left: "$", right: "$", display: false}}
                ],
                throwOnError: false,
                fleqn: false,
                leqno: false,
                strict: false,
                trust: true,
                macros: {{
                    "\\mathbf": "\\boldsymbol"
                }}
            }});
        }});
    </script>
</head>
<body>
{}
</body>
</html>"#,
            css, html
        ))
    }

    fn minify_html(&self, html: &str) -> String {
        html.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs::File;
    use std::io::Write;

    fn create_temp_file(content: &str) -> PathBuf {
        let temp_path = temp_dir().join("test.md");
        let mut file = File::create(&temp_path).unwrap();
        writeln!(file, "{}", content).unwrap();
        temp_path
    }

    fn cleanup_temp_file(path: &PathBuf) {
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_converter_initialization() {
        let temp_path = create_temp_file("# Test");

        let converter =
            MarkdownConverter::new(temp_path.clone(), None, None, true, false, false).unwrap();

        assert_eq!(converter.input_path, temp_path);
        assert_eq!(converter.syntax_highlight, true);
        assert_eq!(converter.generate_toc, false);
        assert_eq!(converter.minify, false);

        cleanup_temp_file(&temp_path);
    }

    #[test]
    fn test_output_path_generation() {
        let temp_path = create_temp_file("# Test");
        let mut expected_output = temp_path.clone();
        expected_output.set_extension("html");

        let converter =
            MarkdownConverter::new(temp_path.clone(), None, None, false, false, false).unwrap();

        assert_eq!(converter.output_path, expected_output);
        cleanup_temp_file(&temp_path);
    }

    #[test]
    fn test_minification() {
        let temp_path = create_temp_file("# Test\n\nSome content");

        let converter =
            MarkdownConverter::new(temp_path.clone(), None, None, false, false, true).unwrap();

        let html = "<div>\n    <p>Test</p>\n</div>";
        let minified = converter.minify_html(html);
        assert!(!minified.contains('\n'));
        assert!(!minified.contains("    "));

        cleanup_temp_file(&temp_path);
    }

    #[test]
    fn test_css_handling() {
        let temp_path = create_temp_file("# Test");
        let converter =
            MarkdownConverter::new(temp_path.clone(), None, None, false, false, false).unwrap();

        let html = "<p>Test</p>";
        let result = converter.add_css(html).unwrap();

        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<style>"));
        assert!(result.contains("</style>"));
        assert!(result.contains(html));

        cleanup_temp_file(&temp_path);
    }
}
