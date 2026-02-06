use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

/// A parse error with source location information for rich diagnostics.
#[derive(Debug, Diagnostic, Error)]
#[error("{message}")]
pub struct ParseDiagnostic {
    #[source_code]
    pub src: NamedSource<String>,
    #[label("{message}")]
    pub span: SourceSpan,
    pub message: String,
}

/// General error type for the IDL reader.
#[derive(Debug, Error)]
pub enum IdlError {
    #[error("parse error: {0}")]
    Parse(String),

    #[error(transparent)]
    Diagnostic(#[from] ParseDiagnostic),

    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("{0}")]
    Other(String),
}

// miette needs Diagnostic on the top-level error to render it properly.
impl Diagnostic for IdlError {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        match self {
            IdlError::Diagnostic(d) => d.source_code(),
            _ => None,
        }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        match self {
            IdlError::Diagnostic(d) => d.labels(),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, IdlError>;
