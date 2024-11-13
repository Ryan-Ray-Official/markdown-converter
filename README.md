# Markdown Converter CLI

A powerful command-line tool for converting Markdown files to HTML, built in Rust.

## Features

- Interactive CLI interface
- Syntax highlighting for code blocks
- Math expressions support using KaTeX
- Table of contents generation
- Custom CSS styling
- File watching for live updates
- HTML minification
- Support for:
  - Tables
  - Nested lists
  - Blockquotes
  - Math expressions
  - Code blocks with syntax highlighting
  - Task lists
  - And more...

## Installation

```bash
git clone https://github.com/Ryan-Ray-Official/markdown-converter
cd markdown-converter
cargo build --release
```

## Usage

```bash
cargo run
```

Follow the interactive prompts to:
1. Select input Markdown file
2. Choose output HTML file location
3. Optionally provide custom CSS
4. Enable/disable features like syntax highlighting and table of contents

## Example

```bash
cargo run
# Enter input file: example.md
# Press Enter for default output location
# Select desired features
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

> Thanks to [ChatGPT](https://chatgpt.com) for generating a good template example.md that helped with tests, and for helping fix some errors with the markdown parser and for generating the css for the markdown.
