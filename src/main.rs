use markdown_converter::{converter, utils};
use anyhow::Result;
use colored::*;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect};
use log::error;
use std::path::PathBuf;

struct ConversionOptions {
    input_path: PathBuf,
    output_path: PathBuf,
    css_path: Option<PathBuf>,
    syntax_highlight: bool,
    generate_toc: bool,
    minify: bool,
    watch: bool,
}

fn main() -> Result<()> {
    env_logger::init();
    let _term = Term::stdout();
    let theme = ColorfulTheme::default();

    print_banner();

    let options = gather_options(&theme)?;

    if !options.input_path.exists() {
        error!("{}", "Input file does not exist!".red());
        std::process::exit(1);
    }

    let converter = converter::MarkdownConverter::new(
        options.input_path,
        Some(options.output_path),
        options.css_path,
        options.syntax_highlight,
        options.generate_toc,
        options.minify,
    )?;

    if options.watch {
        converter.watch()?;
    } else {
        converter.convert()?;
    }

    Ok(())
}

fn gather_options(theme: &ColorfulTheme) -> Result<ConversionOptions> {
    println!("\n{}", "Please configure your conversion options:".green());

    let input_path: String = Input::with_theme(theme)
        .with_prompt("Enter input markdown file path")
        .validate_with(|input: &String| -> Result<(), &str> {
            let path = PathBuf::from(input);
            if !utils::is_markdown_file(&path) {
                Err("File must be a markdown file (.md)")
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    let output_path: String = Input::with_theme(theme)
        .with_prompt("Enter output HTML file path (or press Enter for default)")
        .allow_empty(true)
        .interact_text()?;

    let output_path = if output_path.is_empty() {
        let mut path = PathBuf::from(&input_path);
        path.set_extension("html");
        path
    } else {
        PathBuf::from(output_path)
    };

    let css_path: String = Input::with_theme(theme)
        .with_prompt("Enter custom CSS file path (optional)")
        .allow_empty(true)
        .interact_text()?;

    let css_path = if css_path.is_empty() {
        None
    } else {
        Some(PathBuf::from(css_path))
    };

    let options = &[
        "Syntax highlighting",
        "Generate table of contents",
        "Minify HTML output",
        "Watch for changes",
    ];

    let defaults = &[true, false, false, false];

    println!("\n{}", "Select additional options (Space to select/unselect, Enter to confirm):".green());
    let selections = MultiSelect::with_theme(theme)
        .items(options)
        .defaults(defaults)
        .interact()?;

    let options = ConversionOptions {
        input_path: PathBuf::from(input_path),
        output_path,
        css_path,
        syntax_highlight: selections.contains(&0),
        generate_toc: selections.contains(&1),
        minify: selections.contains(&2),
        watch: selections.contains(&3),
    };

    println!("\n{}", "Configuration Summary:".bright_blue());
    println!("Input file: {:?}", options.input_path);
    println!("Output file: {:?}", options.output_path);
    println!("CSS file: {:?}", options.css_path);
    println!("Syntax highlighting: {}", options.syntax_highlight);
    println!("Generate TOC: {}", options.generate_toc);
    println!("Minify HTML: {}", options.minify);
    println!("Watch mode: {}", options.watch);

    let confirmed = Confirm::with_theme(theme)
        .with_prompt("Proceed with conversion?")
        .default(true)
        .interact()?;

    if !confirmed {
        println!("{}", "Conversion cancelled.".yellow());
        std::process::exit(0);
    }

    Ok(options)
}

fn print_banner() {
    println!("{}", "╔════════════════════════════════════════╗".bright_blue());
    println!("{}", "║         Markdown Converter CLI         ║".bright_blue());
    println!("{}", "║         Version 0.1.0                 ║".bright_blue());
    println!("{}", "╚════════════════════════════════════════╝".bright_blue());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gather_options() {

    }

    #[test]
    fn test_conversion_options() {
        let options = ConversionOptions {
            input_path: PathBuf::from("test.md"),
            output_path: PathBuf::from("test.html"),
            css_path: None,
            syntax_highlight: true,
            generate_toc: false,
            minify: false,
            watch: false,
        };
        assert!(!options.watch);
        assert!(options.syntax_highlight);
    }
}
