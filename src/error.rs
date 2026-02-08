// Suppress false-positive `unused_assignments` warnings from miette's derive
// macros. The `#[source_code]`, `#[label]`, and field attributes cause the
// compiler to think struct fields are written but never read, because it
// doesn't trace through the generated `Diagnostic` impl.
#![allow(unused_assignments)]

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
