use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConverterError {
    #[error("Failed to read input file: {0}")]
    InputFileError(#[from] std::io::Error),

    #[error("Failed to parse markdown: {0}")]
    ParsingError(String),

    #[error("Failed to write output file: {0}")]
    OutputFileError(String),

    #[error("Invalid CSS file: {0}")]
    CssError(String),

    #[error("Syntax highlighting error: {0}")]
    SyntaxHighlightError(String),
}
