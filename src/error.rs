use miette::{LabeledSpan, NamedSource, SourceSpan};

/// A parse error with source location information for rich diagnostics.
#[derive(Debug)]
pub struct ParseDiagnostic {
    pub src: NamedSource<String>,
    pub span: SourceSpan,
    pub message: String,
}

impl std::fmt::Display for ParseDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseDiagnostic {}

impl miette::Diagnostic for ParseDiagnostic {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        Some(Box::new(std::iter::once(LabeledSpan::new_with_span(
            Some(self.message.clone()),
            self.span,
        ))))
    }
}
