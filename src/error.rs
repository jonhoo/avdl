use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
#[error("{message}")]
pub struct ParseDiagnostic {
    #[source_code]
    pub src: NamedSource<String>,
    #[label("{message}")]
    pub span: SourceSpan,
    pub message: String,
}
