use miette::{LabeledSpan, NamedSource, SourceSpan};

/// A parse error with source location information for rich diagnostics.
///
/// The `message` field is used for the top-level `Display` text (the line after
/// the `x` in miette's graphical output). The optional `label` field provides a
/// shorter string for the source-underline annotation. When `label` is `None`,
/// the label falls back to `message`, preserving backwards compatibility.
///
/// Separating these two fields avoids the duplication where the same long
/// ANTLR error message appeared both as the top-level error text and as the
/// source-underline label.
#[derive(Debug)]
pub struct ParseDiagnostic {
    pub src: NamedSource<String>,
    pub span: SourceSpan,
    pub message: String,
    /// Shorter label for the source-underline annotation. When `None`, falls
    /// back to `message`.
    pub label: Option<String>,
    /// Additional help text displayed below the error. Used to show the full
    /// expected-token list when the main message has been simplified.
    pub help: Option<String>,
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

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        self.help
            .as_ref()
            .map(|h| Box::new(h) as Box<dyn std::fmt::Display + 'a>)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        let label_text = self.label.clone().unwrap_or_else(|| self.message.clone());
        Some(Box::new(std::iter::once(LabeledSpan::new_with_span(
            Some(label_text),
            self.span,
        ))))
    }
}
