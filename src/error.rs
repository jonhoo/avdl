use miette::{Diagnostic, LabeledSpan, MietteError, MietteSpanContents, SourceCode, SourceSpan};

/// A source span paired with the file it points into.
///
/// Stored in `ParseDiagnostic`, `Warning`, and `AvroSchema::Reference` so that
/// every diagnostic — including those from imported files — can render the
/// correct source excerpt and filename.
///
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SpanWithSource {
    pub offset: usize,
    pub length: usize,
    /// Absolute path of the file that contains this span.
    pub name: &'static str,
    /// Full source text of that file.
    pub content: &'static str,
}

impl SpanWithSource {
    pub fn new(
        offset: usize,
        length: usize,
        file_name: &'static str,
        content: &'static str,
    ) -> Self {
        SpanWithSource {
            offset,
            length,
            name: file_name,
            content,
        }
    }

    pub(crate) fn source_span(&self) -> SourceSpan {
        SourceSpan::new(self.offset.into(), self.length)
    }
}

impl SourceCode for SpanWithSource {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, MietteError> {
        let inner = SourceCode::read_span(
            &self.content,
            span,
            context_lines_before,
            context_lines_after,
        )?;
        Ok(Box::new(MietteSpanContents::new_named(
            self.name.to_string(),
            inner.data(),
            *inner.span(),
            inner.line(),
            inner.column(),
            inner.line_count(),
        )))
    }
}

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
    pub span: SpanWithSource,
    pub message: String,
    /// Shorter label for the source-underline annotation. When `None`, falls
    /// back to `message`.
    pub label: Option<String>,
    /// Additional help text displayed below the error. Used to show the full
    /// expected-token list when the main message has been simplified.
    pub help: Option<String>,
    /// Additional diagnostics displayed below the primary error. Used when
    /// multiple independent errors are detected (e.g., multiple ANTLR syntax
    /// errors, multiple unresolved type references) so users can fix them all
    /// in one edit cycle.
    pub related: Vec<ParseDiagnostic>,
}

impl std::fmt::Display for ParseDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseDiagnostic {}

/// Render a `miette::Report` to a deterministic string for snapshot testing.
///
/// Uses `GraphicalTheme::none()` (no box-drawing characters) with a wide
/// width to avoid wrapping error messages (which would break tempdir path
/// replacement in tests that normalize paths).
#[cfg(test)]
pub(crate) fn render_diagnostic(report: &miette::Report) -> String {
    use miette::{GraphicalReportHandler, GraphicalTheme};
    let handler = GraphicalReportHandler::new_themed(GraphicalTheme::none()).with_width(200);
    let mut buf = String::new();
    handler
        .render_report(&mut buf, report.as_ref())
        .expect("render to String is infallible");
    buf
}

impl miette::Diagnostic for ParseDiagnostic {
    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.span)
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
            self.span.source_span(),
        ))))
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        if self.related.is_empty() {
            None
        } else {
            Some(Box::new(self.related.iter().map(|d| d as &dyn Diagnostic)))
        }
    }
}
