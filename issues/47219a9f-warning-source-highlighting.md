# Warnings lack source highlighting

## Symptom

Warnings (e.g., out-of-place doc comments) are rendered as plain
text with line/column numbers:

    out-of-place doc comment at line 6, column 17 ...

They don't show the source context with underlined tokens like parse
errors do after the rich-error-diagnostics work.

## Root cause

The `Warning` struct uses line/column numbers extracted from ANTLR
tokens, not byte offsets. The `Display` impl formats these as plain
text. Converting to `ParseDiagnostic`-style rendering would require
changing `Warning` to store byte offsets (or a `SourceSpan`) and
implementing `miette::Diagnostic`.

## Affected files

- `src/reader.rs` — `Warning` struct and its construction sites
- `tests/error_reporting.rs` — warning snapshot tests

## Suggested fix

Either:

1. Add `span: Option<miette::SourceSpan>` to `Warning` and have the
   warning rendering path use `ParseDiagnostic` (or a similar
   `miette::Diagnostic` impl) when a span is available.

2. Or compute byte offsets from ANTLR tokens at warning creation
   time using the same `get_start()`/`get_stop()` approach used for
   errors.

Low priority since warnings are informational, not blocking.
