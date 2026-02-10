// ==============================================================================
// IDL Reader: Recursive Parse Tree Walker
// ==============================================================================
//
// This module is the core of the Avro IDL parser. It takes a string containing
// Avro IDL source, lexes and parses it via ANTLR, then walks the resulting
// parse tree recursively to build our domain model (Protocol, AvroSchema, etc.).
//
// The generated parser defines token constants in lower_Camel_case (e.g.
// `Idl_Boolean`). We suppress the naming warning for the whole module since
// these constants appear extensively in match arms.
#![allow(non_upper_case_globals)]
//
// The Java reference implementation uses ANTLR's listener pattern with mutable
// stacks. That approach is awkward in Rust due to lifetime constraints on trait
// objects, so instead we set `build_parse_tree = true` and walk the tree with
// plain recursive functions that return values. This is simpler and more
// idiomatic Rust.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use std::borrow::Borrow;

use antlr4rust::common_token_stream::CommonTokenStream;
use antlr4rust::error_listener::ErrorListener;
use antlr4rust::parser::Parser;
use antlr4rust::recognizer::Recognizer;
use antlr4rust::token::Token;
use antlr4rust::token_factory::TokenFactory;
use antlr4rust::token_stream::TokenStream;
use antlr4rust::tree::{ParseTree, Tree};
use antlr4rust::{InputStream, TidExt};
use serde_json::Value;
use std::collections::HashMap;

use crate::doc_comments::extract_doc_comment;
use crate::error::ParseDiagnostic;
use crate::generated::idllexer::IdlLexer;
use crate::generated::idlparser::*;
use crate::model::protocol::{Message, Protocol};
use crate::model::schema::{
    AvroSchema, Field, FieldOrder, LogicalType, PrimitiveType, validate_default,
};
use crate::resolve::is_valid_avro_name;
use miette::{Context, Result};

// ==============================================================================
// Warnings
// ==============================================================================

/// A non-fatal warning generated during IDL parsing, such as an out-of-place
/// documentation comment. Matches the Java `IdlReader` warning format.
///
/// When `source` and `span` are present, the warning can be rendered with
/// source context highlighting via miette, similar to how parse errors show
/// the offending token underlined.
pub(crate) struct Warning {
    pub(crate) message: String,
    /// The source text and file name, for rich diagnostic rendering.
    pub(crate) source: Option<miette::NamedSource<String>>,
    /// Byte range of the token that triggered the warning.
    pub(crate) span: Option<miette::SourceSpan>,
}

/// Custom `Debug` implementation that shows a compact representation instead of
/// the deeply nested default. `NamedSource` debug-prints `source: "<redacted>"`
/// and `SourceSpan` wraps offsets in `SourceOffset(...)`, making the derived
/// output ~20 lines for a single warning. This shows just the essential fields:
/// the message, the file name, and the byte span as `start..end`.
impl std::fmt::Debug for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Warning")
            .field("message", &self.message)
            .field("file", &self.source.as_ref().map(|s| s.name().to_string()))
            .field(
                "span",
                &self
                    .span
                    .map(|s| format!("{}..{}", s.offset(), s.offset() + s.len())),
            )
            .finish()
    }
}

impl Warning {
    /// Create an out-of-place doc comment warning with line and column info.
    ///
    /// The format matches Java's `IdlReader.getDocComment()`:
    ///   "Line %d, char %d: Ignoring out-of-place documentation comment.\n
    ///    Did you mean to use a multiline comment ( /* ... */ ) instead?"
    ///
    /// `token_start` and `token_stop` are the inclusive byte offsets from
    /// `Token::get_start()` / `Token::get_stop()`.
    fn out_of_place_doc_comment(
        line: isize,
        column: isize,
        src: &SourceInfo<'_>,
        token_start: isize,
        token_stop: isize,
    ) -> Self {
        let (offset, length) = if token_start >= 0 && token_stop >= token_start {
            (
                token_start as usize,
                (token_stop - token_start + 1) as usize,
            )
        } else if token_start >= 0 {
            (token_start as usize, 1)
        } else {
            (0, 0)
        };

        Warning {
            message: format!(
                "Line {}, char {}: Ignoring out-of-place documentation comment.\n\
                 Did you mean to use a multiline comment ( /* ... */ ) instead?",
                line,
                // Java uses getCharPositionInLine() + 1 (1-based); ANTLR's
                // get_column() is 0-based, so we add 1 to match.
                column + 1,
            ),
            source: Some(miette::NamedSource::new(src.name, src.source.to_string())),
            span: Some(miette::SourceSpan::new(offset.into(), length)),
        }
    }
}

impl std::fmt::Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Warning {}

/// Implements `miette::Diagnostic` so warnings with source spans render with
/// underlined source context, matching how parse errors already display.
impl miette::Diagnostic for Warning {
    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Warning)
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.source.as_ref().map(|s| s as &dyn miette::SourceCode)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        let span = self.span?;
        // Derive a short label from the message content rather than hardcoding
        // a single label for all warning types.
        let label = if self.message.contains("out-of-place documentation comment") {
            "out-of-place doc comment"
        } else if self.message.contains("token recognition error") {
            "unrecognized token"
        } else {
            "here"
        };
        Some(Box::new(std::iter::once(
            miette::LabeledSpan::new_with_span(Some(label.to_string()), span),
        )))
    }
}

// ==============================================================================
// ANTLR Error Collection
// ==============================================================================
//
// Java's IdlReader installs a custom `BaseErrorListener` that throws
// `SchemaParseException` on any syntax error, causing the tool to fail
// immediately. ANTLR's default `ConsoleErrorListener` only prints to stderr
// and lets error recovery continue, which silently produces incorrect output.
//
// We replace the default listener with `CollectingErrorListener`, which records
// each syntax error's line/column/message. After parsing, we check the
// collected errors and return an error if any were found.

/// A collected ANTLR syntax error with byte offset information for source
/// highlighting via miette.
struct SyntaxError {
    /// Byte offset of the offending token's start in the source text.
    offset: usize,
    /// Byte length of the offending token.
    length: usize,
    /// The ANTLR error message (e.g., "mismatched input '}' expecting {';', ','}").
    message: String,
    /// Shorter label for the source-underline annotation in miette output.
    /// When `None`, the full `message` is used as the label (legacy behavior).
    label: Option<String>,
    /// Additional help text (e.g., the full expected-token list when the main
    /// message has been simplified).
    help: Option<String>,
}

// ==========================================================================
// ANTLR Error Message Enrichment
// ==========================================================================
//
// Raw ANTLR error messages are technically correct but often unhelpful
// because the parser's recovery merges tokens in confusing ways. For
// example, `@beta record Foo { ... }` produces:
//
//     no viable alternative at input '@betarecord'
//
// because ANTLR treats `@beta` as the start of a `schemaProperty` rule
// and expects `(` next; when it sees `record` instead, it lumps the
// tokens together. We pattern-match on known error shapes to produce
// more actionable messages while preserving the original as context.

/// The result of enriching an ANTLR error message. Contains a rewritten
/// `message` for Display and an optional shorter `label` for the source
/// underline annotation.
struct EnrichedError {
    message: String,
    /// Shorter label for the source annotation. `None` means the message
    /// itself should be used as the label (the legacy behavior).
    label: Option<String>,
    /// Additional help text displayed below the error (e.g., the full
    /// expected-token list when the main message was simplified).
    help: Option<String>,
}

/// Rewrites known ANTLR error patterns into more user-friendly messages.
/// Returns `None` if the error doesn't match any known pattern, in which
/// case the original message is used as-is.
fn enrich_antlr_error(msg: &str) -> Option<EnrichedError> {
    // Pattern 1: "no viable alternative at input '...@<word>...'"
    //
    // This occurs when `@word` appears without `(value)`. ANTLR merges the
    // annotation name with subsequent tokens into a single error string
    // like `@betarecord`. We split the merged text at known keyword
    // boundaries to recover the annotation name.
    //
    // https://github.com/antlr4rust/antlr4/pull/40
    if let Some(input) = extract_no_viable_input(msg)
        && let Some(anno_name) = extract_annotation_name(input)
    {
        return Some(EnrichedError {
            message: format!(
                "annotation `@{anno_name}` is missing its value -- \
                     use `@{anno_name}(\"value\")` syntax"
            ),
            label: None,
            help: None,
        });
    }

    // Pattern 2: "mismatched input '<token>' expecting '('"
    //
    // This occurs when `@name` is followed by something other than `(`,
    // meaning the annotation value is missing. The error is clear about
    // `(` being expected, but doesn't explain WHY -- the user may not
    // realize annotations require parenthesized values.
    //
    // https://github.com/antlr4rust/antlr4/pull/38
    if msg.contains("expecting '('") && msg.contains("mismatched input") {
        return Some(EnrichedError {
            message: format!("{msg} (annotations require `@name(value)` syntax)"),
            label: None,
            help: None,
        });
    }

    // Pattern 3: errors with a large expected-token set (more than 5 tokens).
    //
    // ANTLR dumps the full set of expected tokens when it encounters an
    // unexpected token. For most grammar positions this set is huge (20-30
    // tokens covering every Avro keyword, type name, and IdentifierToken).
    // We detect these large sets and produce a shorter, more actionable
    // message while keeping a concise label for the source annotation.
    if let Some(enriched) = simplify_large_expecting_set(msg) {
        return Some(enriched);
    }

    None
}

// ==========================================================================
// Large Expected-Token Set Simplification
// ==========================================================================

/// Threshold: if the expected-token set contains more than this many tokens,
/// we simplify the message instead of showing the full list.
const EXPECTING_SET_TRUNCATION_THRESHOLD: usize = 5;

/// Detects ANTLR error messages with large expected-token sets and rewrites
/// them into more user-friendly messages with short labels.
///
/// The full expected-token list is preserved in the `help` field so that
/// miette renders it below the error, giving the user both a concise message
/// and the full set of valid alternatives.
///
/// Handles three ANTLR error patterns:
/// - `extraneous input '<tok>' expecting {<set>}`
/// - `mismatched input '<tok>' expecting {<set>}`
/// - `no viable alternative at input '<tok>'` (without annotation pattern)
fn simplify_large_expecting_set(msg: &str) -> Option<EnrichedError> {
    // Try to extract the expected-token set from `expecting {<set>}` or
    // `expecting '<single-token>'`.
    let expecting_tokens = extract_expecting_tokens(msg);

    // Only simplify when the token set exceeds our threshold.
    let tokens = expecting_tokens.as_deref()?;
    if count_tokens_in_set(tokens) <= EXPECTING_SET_TRUNCATION_THRESHOLD {
        return None;
    }

    let help = format_expected_help(tokens);
    let expects_string = expecting_set_includes_string_literal(tokens);

    // Determine the offending token and error shape.
    if let Some(offending) = extract_quoted_token(msg, "extraneous input ") {
        return Some(build_unexpected_token_error(offending, help, expects_string));
    }

    if let Some(offending) = extract_quoted_token(msg, "mismatched input ") {
        return Some(build_unexpected_token_error(offending, help, expects_string));
    }

    None
}

/// Builds an `EnrichedError` for an unexpected token in a position with a
/// large expected-token set.
///
/// When the offending token looks like a bare identifier (e.g., `YELLOW`) and
/// the expected set includes `StringLiteral`, we add a hint suggesting that
/// the identifier should be quoted. This covers the common mistake of writing
/// `Color primary = YELLOW;` instead of `Color primary = "YELLOW";` for enum
/// defaults.
fn build_unexpected_token_error(
    offending: &str,
    help: Option<String>,
    expects_string: bool,
) -> EnrichedError {
    if offending == "<EOF>" {
        return EnrichedError {
            message: "unexpected end of file".to_string(),
            label: Some("unexpected end of file".to_string()),
            help,
        };
    }

    // When the offending token looks like a bare identifier and a string
    // literal is expected, the user likely forgot to quote the value (e.g.,
    // an enum default like `Color primary = YELLOW` instead of `= "YELLOW"`).
    if expects_string && looks_like_bare_identifier(offending) {
        let help = append_quoting_hint(help, offending);
        return EnrichedError {
            message: format!(
                "unexpected token `{offending}` -- did you mean `\"{offending}\"`?"
            ),
            label: Some(format!(
                "unexpected `{offending}` -- did you mean `\"{offending}\"`?"
            )),
            help,
        };
    }

    EnrichedError {
        message: format!("unexpected token `{offending}`"),
        label: Some(format!("unexpected `{offending}`")),
        help,
    }
}

/// Formats the ANTLR expected-token set into a human-readable help string.
///
/// Strips internal tokens that aren't meaningful to users (DocComment,
/// the SUB character `\u001A`, and `<EOF>`) and removes surrounding quotes
/// from token names.
fn format_expected_help(tokens: &str) -> Option<String> {
    let cleaned: Vec<&str> = tokens
        .split(',')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .filter(|t| *t != "DocComment" && *t != "'\\u001A'" && *t != "<EOF>")
        .map(|t| t.trim_matches('\''))
        .collect();
    if cleaned.is_empty() {
        return None;
    }
    Some(format!("expected one of: {}", cleaned.join(", ")))
}

/// Extracts the token set string from `expecting {<set>}` in an ANTLR
/// error message. Returns the content between the braces, or `None` if the
/// pattern is not found.
fn extract_expecting_tokens(msg: &str) -> Option<&str> {
    let prefix = "expecting {";
    let start = msg.find(prefix)? + prefix.len();
    let end = start + msg[start..].find('}')?;
    Some(&msg[start..end])
}

/// Counts the number of comma-separated tokens in an ANTLR expected-token
/// set string. Handles both `{tok1, tok2, ...}` and single-token forms.
fn count_tokens_in_set(set: &str) -> usize {
    if set.trim().is_empty() {
        return 0;
    }
    set.split(',').count()
}

/// Extracts the quoted token after a given prefix in an ANTLR error message.
/// For example, given prefix `"extraneous input "` and message
/// `"extraneous input '<EOF>' expecting {…}"`, returns `Some("<EOF>")`.
fn extract_quoted_token<'a>(msg: &'a str, prefix: &str) -> Option<&'a str> {
    let start = msg.find(prefix)? + prefix.len();
    let rest = &msg[start..];
    if !rest.starts_with('\'') {
        return None;
    }
    let end = rest[1..].find('\'')?;
    Some(&rest[1..1 + end])
}

/// Returns `true` if the ANTLR expected-token set includes `StringLiteral`.
///
/// This signals that the parser was expecting a JSON value (or similar
/// string-accepting position), which helps us detect the "bare identifier
/// instead of quoted string" pattern.
fn expecting_set_includes_string_literal(tokens: &str) -> bool {
    tokens
        .split(',')
        .any(|t| t.trim().trim_matches('\'') == "StringLiteral")
}

/// Returns `true` if the token text looks like a bare identifier: starts with
/// a letter or underscore, and contains only alphanumeric characters and
/// underscores. This excludes JSON keywords (`null`, `true`, `false`) since
/// those are valid in default-value positions and have their own ANTLR tokens.
fn looks_like_bare_identifier(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    // JSON keywords are valid jsonValue alternatives; don't flag them.
    if matches!(token, "null" | "true" | "false") {
        return false;
    }
    let mut chars = token.chars();
    let first = chars.next().expect("non-empty token has a first char");
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Appends a quoting hint to an existing help message (or creates one).
///
/// Produces a suggestion like: `hint: did you mean "YELLOW"? Enum default
/// values must be quoted strings.`
fn append_quoting_hint(help: Option<String>, offending: &str) -> Option<String> {
    let hint = format!(
        "hint: did you mean \"{offending}\"? \
         Enum default values must be quoted strings"
    );
    Some(match help {
        Some(h) => format!("{h}\n{hint}"),
        None => hint,
    })
}

/// Extracts the quoted input string from a "no viable alternative at input
/// '<input>'" ANTLR error message.
fn extract_no_viable_input(msg: &str) -> Option<&str> {
    let prefix = "no viable alternative at input '";
    let start = msg.find(prefix)? + prefix.len();
    let end = msg[start..].find('\'')?;
    Some(&msg[start..start + end])
}

/// Avro IDL keywords that commonly follow a bare annotation in source. When
/// ANTLR merges the annotation name with the next token, these keywords help
/// us split the merged text to recover the actual annotation name. For
/// example, `@betarecord` splits into `beta` + `record`.
const AVRO_KEYWORDS: &[&str] = &[
    "protocol",
    "record",
    "error",
    "enum",
    "fixed",
    "import",
    "schema",
    "namespace",
    "boolean",
    "int",
    "long",
    "float",
    "double",
    "string",
    "bytes",
    "null",
    "array",
    "map",
    "union",
    "void",
    "oneway",
    "throws",
    "date",
    "time_ms",
    "timestamp_ms",
    "localtimestamp_ms",
    "uuid",
];

/// Extracts the annotation name from a merged ANTLR input string like
/// `@betarecord` or `@namespace("com.example")@versionprotocol`.
///
/// Returns the *last* `@word` that is not followed by `(`, since that is
/// the one missing its value. Earlier annotations with `(` were parsed
/// successfully.
///
/// Because ANTLR merges the annotation name with the next token (e.g.,
/// `@beta` + `record` becomes `@betarecord`), we try to split the text
/// at known keyword boundaries to recover the actual annotation name.
fn extract_annotation_name(input: &str) -> Option<&str> {
    // Walk backwards through all `@word` occurrences to find the last
    // one not followed by `(`.
    let mut search_from = input.len();
    loop {
        let at_pos = input[..search_from].rfind('@')?;
        let after_at = &input[at_pos + 1..];

        // Collect the full identifier text after `@`.
        let ident_len = after_at
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .count();

        if ident_len > 0 {
            let full_ident = &after_at[..ident_len];
            let after_ident = &after_at[ident_len..];

            if !after_ident.starts_with('(') {
                // This `@name` is missing its `(value)`. Try to split
                // off a trailing keyword to recover the real name.
                let name = split_trailing_keyword(full_ident);
                return Some(name);
            }
        }

        if at_pos == 0 {
            return None;
        }
        search_from = at_pos;
    }
}

/// Given a merged identifier like `betarecord`, tries to split off a
/// trailing Avro keyword to recover the annotation name (`beta`).
/// Returns the original string if no keyword suffix is found.
fn split_trailing_keyword(merged: &str) -> &str {
    let lower = merged.to_ascii_lowercase();
    for &kw in AVRO_KEYWORDS {
        if lower.ends_with(kw) && lower.len() > kw.len() {
            return &merged[..merged.len() - kw.len()];
        }
    }
    merged
}

/// Convert 1-based `line` and 0-based `column` (as reported by ANTLR) to a
/// byte offset into `source`. Returns 0 if the coordinates are out of range.
fn line_col_to_byte_offset(source: &str, line: isize, column: isize) -> usize {
    if line <= 0 || column < 0 {
        return 0;
    }
    let target_line = (line - 1) as usize; // convert to 0-based
    let mut offset = 0;
    for (i, src_line) in source.split('\n').enumerate() {
        if i == target_line {
            return offset + (column as usize).min(src_line.len());
        }
        offset += src_line.len() + 1; // +1 for the newline character
    }
    0
}

/// An ANTLR error listener that collects syntax errors into a shared `Vec`
/// instead of printing them to stderr. This lets us detect parse errors after
/// `parser.idlFile()` returns and fail with a proper error.
///
/// The optional `source` field holds the original input text, enabling the
/// listener to compute byte offsets from ANTLR's (line, column) when no
/// offending token is available (lexer errors).
struct CollectingErrorListener {
    errors: Rc<RefCell<Vec<SyntaxError>>>,
    /// Original source text for line/column-to-byte-offset conversion.
    /// `None` for parser errors where the offending token always provides
    /// byte offsets directly.
    source: Option<Rc<str>>,
}

impl<'a, T: Recognizer<'a>> ErrorListener<'a, T> for CollectingErrorListener {
    fn syntax_error(
        &self,
        _recognizer: &T,
        offending_symbol: Option<&<T::TF as TokenFactory<'a>>::Inner>,
        line: isize,
        column: isize,
        msg: &str,
        _error: Option<&antlr4rust::errors::ANTLRError>,
    ) {
        // Extract byte offsets from the offending token when available. These
        // give us a precise source span for miette to underline. When the token
        // is absent (e.g., lexer errors), we compute the offset from the
        // line/column parameters using the stored source text.
        let (offset, length) = offending_symbol
            .map(|tok| {
                let start = tok.get_start();
                let stop = tok.get_stop();
                if start >= 0 && stop >= start {
                    (start as usize, (stop - start + 1) as usize)
                } else if start >= 0 {
                    (start as usize, 1)
                } else {
                    (0, 0)
                }
            })
            .unwrap_or_else(|| {
                // Lexer errors have no offending token. Fall back to computing
                // the byte offset from (line, column) using the source text.
                if let Some(ref src) = self.source {
                    let offset = line_col_to_byte_offset(src, line, column);
                    (offset, 1)
                } else {
                    (0, 0)
                }
            });

        // Try to enrich the raw ANTLR message with a more user-friendly
        // explanation. Fall back to the original if no pattern matches.
        let enriched = enrich_antlr_error(msg);

        let (display_msg, label, help) = match enriched {
            Some(e) => (e.message, e.label, e.help),
            None => (msg.to_string(), None, None),
        };

        self.errors.borrow_mut().push(SyntaxError {
            offset,
            length,
            message: format!("line {line}:{column} {display_msg}"),
            label,
            help,
        });
    }
}

/// Type names that collide with Avro built-in types. Matches Java's
/// `IdlReader.INVALID_TYPE_NAMES`.
const INVALID_TYPE_NAMES: &[&str] = &[
    "boolean",
    "int",
    "long",
    "float",
    "double",
    "bytes",
    "string",
    "null",
    "date",
    "time_ms",
    "timestamp_ms",
    "localtimestamp_ms",
    "uuid",
];

// ==========================================================================
// Public API
// ==========================================================================

/// The result of parsing an IDL file -- either a protocol or a standalone schema.
#[derive(Debug)]
pub enum IdlFile {
    Protocol(Protocol),
    /// A file with an explicit `schema <type>;` declaration. Serialized as a
    /// single JSON schema object.
    Schema(AvroSchema),
    /// A file with bare named type declarations (no `schema` keyword and no
    /// `protocol`). Serialized as a JSON array of all named schemas, matching
    /// the Java `IdlFile.outputString()` behavior.
    NamedSchemas(Vec<AvroSchema>),
}

/// Import type discovered during parsing. The actual import resolution is
/// deferred to the `import` module (not yet implemented).
#[derive(Debug, Clone)]
pub struct ImportEntry {
    pub kind: ImportKind,
    pub path: String,
    /// Byte range of the import statement in the originating IDL source,
    /// enabling source-highlighted diagnostics when import resolution fails.
    pub span: Option<miette::SourceSpan>,
}

/// The kind of import statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportKind {
    Idl,
    Protocol,
    Schema,
}

/// A declaration item in source order. Captures both import statements and
/// local type definitions interleaved exactly as they appear in the IDL file.
/// This preserves the declaration order so that the caller can register types
/// and resolve imports in the correct sequence.
#[derive(Debug, Clone)]
pub enum DeclItem {
    /// An import statement to be resolved later.
    Import(ImportEntry),
    /// A locally-defined named type (record, enum, or fixed).
    ///
    /// The optional `SourceSpan` records the byte range of the declaration in
    /// the originating IDL source, enabling source-highlighted diagnostics when
    /// registration fails (e.g., duplicate type name).
    Type(AvroSchema, Option<miette::SourceSpan>),
}

/// Test-only wrapper around [`parse_idl_named`] that normalizes CRLF line
/// endings to LF before parsing. This ensures byte offsets in ANTLR tokens
/// (and therefore in `SourceSpan` error diagnostics) are consistent in tests
/// regardless of the source file's line ending convention.
///
/// Tests that need to verify CRLF-specific behavior should call
/// [`parse_idl_named`] directly.
#[cfg(test)]
pub fn parse_idl_for_test(input: &str) -> Result<(IdlFile, Vec<DeclItem>, Vec<Warning>)> {
    let normalized;
    let input = if input.contains("\r\n") {
        normalized = input.replace("\r\n", "\n");
        normalized.as_str()
    } else {
        input
    };

    parse_idl_named(input, "<input>")
}

/// Parse an Avro IDL string, attaching `source_name` to any error diagnostics
/// so that error messages identify the originating file.
pub fn parse_idl_named(
    input: &str,
    source_name: &str,
) -> Result<(IdlFile, Vec<DeclItem>, Vec<Warning>)> {
    // The ANTLR grammar's `idlFile` rule includes `('\u001a' .*?)? EOF`
    // to treat the ASCII SUB character (U+001A) as an end-of-file marker,
    // ignoring any trailing content. The antlr4rust runtime does not handle
    // this correctly, so we strip the SUB character and everything after it
    // before passing the input to the lexer.
    let input = if let Some(pos) = input.find('\u{001a}') {
        &input[..pos]
    } else {
        input
    };
    let input_stream = InputStream::new(input);
    let mut lexer = IdlLexer::new(input_stream);

    // Replace the lexer's default ConsoleErrorListener with a
    // CollectingErrorListener so that token-recognition errors (e.g.,
    // unrecognized characters) don't leak to stderr. We surface them as
    // warnings instead, matching Java's behavior of not treating lexer
    // errors as fatal. (Java also doesn't install a custom listener on
    // the lexer — it just lets ConsoleErrorListener print to stderr.)
    let source_rc: Rc<str> = Rc::from(input);
    let lexer_errors: Rc<RefCell<Vec<SyntaxError>>> = Rc::new(RefCell::new(Vec::new()));
    lexer.remove_error_listeners();
    lexer.add_error_listener(Box::new(CollectingErrorListener {
        errors: Rc::clone(&lexer_errors),
        source: Some(Rc::clone(&source_rc)),
    }));

    let token_stream = CommonTokenStream::new(lexer);
    let mut parser = IdlParser::new(token_stream);

    // Build a parse tree so we can walk it recursively. The `build_parse_trees`
    // field is on `BaseParser`, accessible through `Deref`.
    parser.build_parse_trees = true;

    // Replace the default ConsoleErrorListener with a CollectingErrorListener
    // that records syntax errors. Java's IdlReader does the same thing -- it
    // removes the default listener and installs one that throws on any error.
    // We collect instead of throwing because ANTLR's error recovery may still
    // produce a usable parse tree, but we'll fail after parsing completes.
    let syntax_errors: Rc<RefCell<Vec<SyntaxError>>> = Rc::new(RefCell::new(Vec::new()));
    parser.remove_error_listeners();
    // Parser errors always have an offending token with byte offsets, so no
    // source text is needed for line/column fallback.
    parser.add_error_listener(Box::new(CollectingErrorListener {
        errors: Rc::clone(&syntax_errors),
        source: None,
    }));

    let tree = parser
        .idlFile()
        .map_err(|e| miette::miette!("ANTLR parse error: {e:?}"))
        .wrap_err_with(|| format!("parse `{source_name}`"))?;

    // Convert any lexer errors into warnings. Lexer errors (e.g., unrecognized
    // characters) don't necessarily prevent a valid parse — the lexer skips the
    // offending character and continues. Java also treats these as non-fatal
    // (prints to stderr via the default ConsoleErrorListener).
    let lexer_warnings: Vec<Warning> = RefCell::borrow(&lexer_errors)
        .iter()
        .map(|e| Warning {
            message: e.message.clone(),
            source: Some(miette::NamedSource::new(source_name, input.to_string())),
            span: Some(miette::SourceSpan::new(e.offset.into(), e.length)),
        })
        .collect();

    // Check for ANTLR parser errors. Any syntax error means the input is
    // malformed, even if ANTLR's error recovery produced a parse tree. This
    // matches Java's behavior of throwing on the first error.
    let collected_errors = RefCell::borrow(&syntax_errors);
    if !collected_errors.is_empty() {
        let first = &collected_errors[0];
        return Err(ParseDiagnostic {
            src: miette::NamedSource::new(source_name, input.to_string()),
            span: miette::SourceSpan::new(first.offset.into(), first.length),
            message: first.message.clone(),
            label: first.label.clone(),
            help: first.help.clone(),
        }
        .into());
    }
    drop(collected_errors);

    // The parser's `input` field (on `BaseParser`, accessible through `Deref`)
    // holds the token stream. We need it for doc comment extraction (scanning
    // backwards from a token index through hidden-channel tokens).
    let token_stream = &parser.input;

    let src = SourceInfo {
        source: input,
        name: source_name,
        consumed_doc_indices: RefCell::new(HashSet::new()),
    };

    let mut namespace: Option<String> = None;
    let mut decl_items = Vec::new();

    let idl_file = walk_idl_file(&tree, token_stream, &src, &mut namespace, &mut decl_items)
        .wrap_err_with(|| format!("parse `{source_name}`"))?;

    // ==============================================================================
    // Orphaned Doc Comment Detection
    // ==============================================================================
    //
    // After the tree walk, scan the entire token stream for DocComment tokens
    // that were not consumed by any declaration. These are "out-of-place" doc
    // comments that should be regular multiline comments instead.
    //
    // This matches Java's `IdlReader.getDocComment()` behavior, which generates
    // a warning for each DocComment token in the gap between the previous call's
    // position and the current call's position that isn't the actual doc comment
    // for the current node.
    let warnings = collect_orphaned_doc_comment_warnings(
        token_stream,
        &src.consumed_doc_indices.borrow(),
        &src,
    );

    let mut all_warnings = lexer_warnings;
    all_warnings.extend(warnings);

    Ok((idl_file, decl_items, all_warnings))
}

// ==========================================================================
// Token Stream Type Alias
// ==========================================================================

/// Concrete token stream type produced by our lexer. Every walk function
/// threads this through so it can extract doc comments from hidden tokens.
type TS<'input> = CommonTokenStream<'input, IdlLexer<'input, InputStream<&'input str>>>;

// ==========================================================================
// Source Location Diagnostic Helpers
// ==========================================================================

/// Carries the original source text and a display name through the tree walk
/// so that error messages can include source location context via miette.
///
/// Also tracks which doc comment token indices have been consumed by
/// declarations, so that orphaned doc comments can be detected after the walk.
struct SourceInfo<'a> {
    source: &'a str,
    name: &'a str,
    /// Token indices of doc comments consumed by `extract_doc_from_context`.
    /// After the full tree walk, any `DocComment` token NOT in this set is
    /// orphaned and should generate a warning.
    consumed_doc_indices: RefCell<HashSet<isize>>,
}

/// Construct a `miette::Report` wrapping a `ParseDiagnostic` with source
/// location extracted from an ANTLR parse tree context's start token.
///
/// The start token gives us a byte offset into the original source text. We
/// use the token's `get_start()` and `get_stop()` to compute a byte-level
/// `SourceSpan` that miette can render as an underlined region in the error
/// output.
fn make_diagnostic<'input>(
    src: &SourceInfo<'_>,
    ctx: &impl antlr4rust::parser_rule_context::ParserRuleContext<'input>,
    message: impl Into<String>,
) -> miette::Report {
    let start_token = ctx.start();
    let stop_token = ctx.stop();
    let offset = start_token.get_start();

    // Use the stop token's end byte to span the entire context (e.g. the full
    // `@name(value)` annotation rather than just the leading `@`). Fall back
    // to the start token's own stop byte if the stop token has no valid
    // position.
    let stop = {
        let candidate = stop_token.get_stop();
        if candidate >= offset {
            candidate
        } else {
            start_token.get_stop()
        }
    };

    // Compute a span covering at least one character. ANTLR byte offsets are
    // inclusive on both ends, so length = stop - start + 1.
    let (offset, length) = if offset >= 0 && stop >= offset {
        (offset as usize, (stop - offset + 1) as usize)
    } else if offset >= 0 {
        (offset as usize, 1)
    } else {
        // No valid position available; point at the start of the file.
        (0, 0)
    };

    let message = message.into();
    ParseDiagnostic {
        src: miette::NamedSource::new(src.name, src.source.to_string()),
        span: miette::SourceSpan::new(offset.into(), length),
        message,
        label: None,
        help: None,
    }
    .into()
}

/// Like `make_diagnostic` but takes a raw `Token` reference instead of a
/// context node. Useful when the error relates to a specific token field
/// (e.g. `ctx.size`, `ctx.typeName`) rather than the whole context.
fn make_diagnostic_from_token(
    src: &SourceInfo<'_>,
    token: &impl Token,
    message: impl Into<String>,
) -> miette::Report {
    let offset = token.get_start();
    let stop = token.get_stop();

    let (offset, length) = if offset >= 0 && stop >= offset {
        (offset as usize, (stop - offset + 1) as usize)
    } else if offset >= 0 {
        (offset as usize, 1)
    } else {
        (0, 0)
    };

    let message = message.into();
    ParseDiagnostic {
        src: miette::NamedSource::new(src.name, src.source.to_string()),
        span: miette::SourceSpan::new(offset.into(), length),
        message,
        label: None,
        help: None,
    }
    .into()
}

/// Extract a `SourceSpan` from a parse tree context's start token.
///
/// Returns `None` if the token has no valid position (e.g., synthetic tokens).
/// Used to attach spans to `DeclItem::Type` and `DeclItem::Import` entries so
/// that downstream errors (duplicate type name, import failure) can produce
/// source-highlighted diagnostics.
fn span_from_context<'input>(
    ctx: &impl antlr4rust::parser_rule_context::ParserRuleContext<'input>,
) -> Option<miette::SourceSpan> {
    let start_token = ctx.start();
    let offset = start_token.get_start();
    let stop = start_token.get_stop();

    if offset >= 0 && stop >= offset {
        Some(miette::SourceSpan::new(
            (offset as usize).into(),
            (stop - offset + 1) as usize,
        ))
    } else if offset >= 0 {
        Some(miette::SourceSpan::new((offset as usize).into(), 1))
    } else {
        None
    }
}

// ==========================================================================
// Schema Properties Helper
// ==========================================================================

/// Accumulated `@name(value)` annotations from the parse tree.
///
/// Schema properties like `@namespace`, `@aliases`, and `@order` are special:
/// they are consumed by the walker and not passed through as custom properties.
/// All other annotations end up in the `properties` map.
struct SchemaProperties {
    namespace: Option<String>,
    aliases: Vec<String>,
    order: Option<FieldOrder>,
    properties: HashMap<String, Value>,
}

impl SchemaProperties {
    fn new() -> Self {
        SchemaProperties {
            namespace: None,
            aliases: Vec::new(),
            order: None,
            properties: HashMap::new(),
        }
    }
}

// ==========================================================================
// Context-Sensitive Property Handling
// ==========================================================================
//
// Java's `SchemaProperties` class controls which annotation names are
// intercepted as special (`@namespace`, `@aliases`, `@order`) vs treated
// as custom properties, using boolean flags that vary per parse context.
// When a flag is false, that annotation name falls through to the custom
// properties map instead of being intercepted. This struct mirrors those
// flags.

/// Controls which annotations are intercepted as special fields vs custom
/// properties. Matches the Java `SchemaProperties(contextNamespace,
/// withNamespace, withAliases, withOrder)` constructor flags. Also carries a
/// set of reserved property names that must be rejected if used as custom
/// annotations, matching Java's `JsonProperties.addProp()` validation.
#[derive(Clone, Copy)]
struct PropertyContext {
    with_namespace: bool,
    with_aliases: bool,
    with_order: bool,
    /// Reserved property names for this context. Annotations matching any of
    /// these names produce an error ("Can't set reserved property: {name}").
    /// The sets are taken from avro-tools 1.12.1 (Schema.SCHEMA_RESERVED,
    /// Schema.ENUM_RESERVED, Schema.FIELD_RESERVED, Protocol.PROTOCOL_RESERVED,
    /// Protocol.MESSAGE_RESERVED).
    reserved: &'static [&'static str],
}

// ==========================================================================
// Reserved Property Name Sets
// ==========================================================================
//
// Java's `JsonProperties.addProp()` rejects annotations whose names collide
// with structural JSON keys. The reserved sets are defined per context type
// in `Schema.java` and `Protocol.java`. These must match avro-tools 1.12.1
// behavior exactly -- not the git HEAD source, which may differ.

/// Reserved property names for Schema objects (record, fixed, array, map, etc.).
/// From `Schema.SCHEMA_RESERVED` in avro-tools 1.12.1.
const SCHEMA_RESERVED: &[&str] = &[
    "doc",
    "fields",
    "items",
    "name",
    "namespace",
    "size",
    "symbols",
    "values",
    "type",
    "aliases",
];

/// Reserved property names for Enum schemas. All of `SCHEMA_RESERVED` plus
/// `default`. From `Schema.ENUM_RESERVED` in avro-tools 1.12.1.
const ENUM_RESERVED: &[&str] = &[
    "doc",
    "fields",
    "items",
    "name",
    "namespace",
    "size",
    "symbols",
    "values",
    "type",
    "aliases",
    "default",
];

/// Reserved property names for Field objects. From `Schema.FIELD_RESERVED`
/// (defined alongside `Schema.Field`) in avro-tools 1.12.1.
const FIELD_RESERVED: &[&str] = &["default", "doc", "name", "order", "type", "aliases"];

/// Reserved property names for Protocol objects. From
/// `Protocol.PROTOCOL_RESERVED` in avro-tools 1.12.1.
///
/// Note: the git submodule source may include `version` here, but
/// avro-tools 1.12.1 does NOT -- `@version` is accepted on protocols
/// (used in `simple.avdl` and `nestedimport.avdl`).
const PROTOCOL_RESERVED: &[&str] = &[
    "namespace",
    "protocol",
    "doc",
    "messages",
    "types",
    "errors",
];

/// Reserved property names for Message objects. From
/// `Protocol.MESSAGE_RESERVED` in avro-tools 1.12.1.
const MESSAGE_RESERVED: &[&str] = &["doc", "response", "request", "errors", "one-way"];

/// Context for protocol declarations: namespace is intercepted, but aliases
/// and order are treated as custom properties.
const PROTOCOL_PROPS: PropertyContext = PropertyContext {
    with_namespace: true,
    with_aliases: false,
    with_order: false,
    reserved: PROTOCOL_RESERVED,
};

/// Context for record/fixed declarations: namespace and aliases are
/// intercepted, order is a custom property.
const NAMED_TYPE_PROPS: PropertyContext = PropertyContext {
    with_namespace: true,
    with_aliases: true,
    with_order: false,
    reserved: SCHEMA_RESERVED,
};

/// Context for enum declarations: same interception as record/fixed, but
/// with the extended enum reserved set (includes `default`).
const ENUM_PROPS: PropertyContext = PropertyContext {
    with_namespace: true,
    with_aliases: true,
    with_order: false,
    reserved: ENUM_RESERVED,
};

/// Context for variable declarations (field names): aliases and order are
/// intercepted, namespace is a custom property.
const VARIABLE_PROPS: PropertyContext = PropertyContext {
    with_namespace: false,
    with_aliases: true,
    with_order: true,
    reserved: FIELD_RESERVED,
};

/// Context for fullType: nothing is intercepted (all annotations become
/// custom properties). Uses the schema reserved set since type annotations
/// flow into schema objects.
const BARE_PROPS: PropertyContext = PropertyContext {
    with_namespace: false,
    with_aliases: false,
    with_order: false,
    reserved: SCHEMA_RESERVED,
};

/// Context for message declarations: nothing is intercepted.
const MESSAGE_PROPS: PropertyContext = PropertyContext {
    with_namespace: false,
    with_aliases: false,
    with_order: false,
    reserved: MESSAGE_RESERVED,
};

/// Walk a list of `SchemaPropertyContext` nodes and accumulate them into a
/// `SchemaProperties` struct. Which annotations are intercepted as special
/// fields (`namespace`, `aliases`, `order`) depends on the `pctx` flags,
/// matching Java's context-sensitive `SchemaProperties` behavior.
fn walk_schema_properties<'input>(
    props: &[Rc<SchemaPropertyContextAll<'input>>],
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    pctx: PropertyContext,
) -> Result<SchemaProperties> {
    let mut result = SchemaProperties::new();

    for prop in props {
        let name_ctx = prop
            .identifier()
            .ok_or_else(|| make_diagnostic(src, &**prop, "missing property name"))?;
        let name = identifier_text(&name_ctx);

        let value_ctx = prop
            .jsonValue()
            .ok_or_else(|| make_diagnostic(src, &**prop, "missing property value"))?;
        let value = walk_json_value(&value_ctx, token_stream, src)
            .wrap_err_with(|| format!("parse value for schema property `{name}`"))?;

        // Intercept well-known annotations only when the context flags allow it.
        // When a flag is false, that name falls through to the custom properties
        // path (and may be rejected as reserved there).
        if pctx.with_namespace && name == "namespace" {
            if let Value::String(s) = &value {
                // Last-write-wins for duplicate @namespace, matching Java's
                // behavior (LinkedHashMap.put overwrites silently) and our
                // own handling of duplicate @aliases.
                result.namespace = Some(s.clone());
            } else {
                return Err(make_diagnostic(
                    src,
                    &**prop,
                    "@namespace must contain a string value",
                ));
            }
        } else if pctx.with_aliases && name == "aliases" {
            if let Value::Array(arr) = &value {
                let mut aliases = Vec::new();
                for elem in arr {
                    if let Value::String(s) = elem {
                        // Validate each alias name segment (aliases can be
                        // fully-qualified like "com.example.OldName").
                        for segment in s.split('.') {
                            if !is_valid_avro_name(segment) {
                                return Err(make_diagnostic(
                                    src,
                                    &**prop,
                                    format!("invalid alias name: {s}"),
                                ));
                            }
                        }
                        aliases.push(s.clone());
                    } else {
                        return Err(make_diagnostic(
                            src,
                            &**prop,
                            "@aliases must contain an array of strings",
                        ));
                    }
                }
                result.aliases = aliases;
            } else {
                return Err(make_diagnostic(
                    src,
                    &**prop,
                    "@aliases must contain an array of strings",
                ));
            }
        } else if pctx.with_order && name == "order" {
            if let Value::String(s) = &value {
                match s.to_uppercase().as_str() {
                    "ASCENDING" => result.order = Some(FieldOrder::Ascending),
                    "DESCENDING" => result.order = Some(FieldOrder::Descending),
                    "IGNORE" => result.order = Some(FieldOrder::Ignore),
                    _ => {
                        return Err(make_diagnostic(
                            src,
                            &**prop,
                            format!("@order must be ASCENDING, DESCENDING, or IGNORE, got: {s}"),
                        ));
                    }
                }
            } else {
                return Err(make_diagnostic(
                    src,
                    &**prop,
                    "@order must contain a string value",
                ));
            }
        } else {
            // Reject reserved property names that would collide with
            // structural JSON keys. This matches Java's
            // `JsonProperties.addProp()` which throws
            // "Can't set reserved property: {name}" for each context's
            // reserved set.
            if pctx.reserved.contains(&name.as_str()) {
                return Err(make_diagnostic(
                    src,
                    &**prop,
                    format!("Can't set reserved property: {name}"),
                ));
            }
            result.properties.insert(name, value);
        }
    }

    Ok(result)
}

// ==========================================================================
// Tree Walking Functions
// ==========================================================================

/// Top-level dispatch: protocol mode vs. schema mode.
///
/// Instead of registering types in a SchemaRegistry during parsing, this
/// function collects all imports and local type definitions into `decl_items`
/// in source order. The caller processes these items sequentially to build a
/// correctly ordered registry.
fn walk_idl_file<'input>(
    ctx: &IdlFileContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &mut Option<String>,
    decl_items: &mut Vec<DeclItem>,
) -> Result<IdlFile> {
    // Protocol mode: the IDL contains `protocol Name { ... }`.
    if let Some(protocol_ctx) = ctx.protocolDeclaration() {
        let protocol = walk_protocol(&protocol_ctx, token_stream, src, namespace, decl_items)?;
        return Ok(IdlFile::Protocol(protocol));
    }

    // Schema mode: optional `namespace`, optional `schema` declaration, plus
    // named type declarations.
    if let Some(ns_ctx) = ctx.namespaceDeclaration()
        && let Some(id_ctx) = ns_ctx.identifier()
    {
        let id = identifier_text(&id_ctx);
        // In schema mode, `namespace foo.bar;` sets the enclosing namespace
        // directly. Unlike protocol/record identifiers (where dots in the
        // name imply a namespace prefix), here the entire identifier IS the
        // namespace value.
        *namespace = Some(id);
    }

    // Walk the body children in source order, interleaving imports and named
    // schema declarations. The grammar rule is:
    //   (imports+=importStatement | namedSchemas+=namedSchemaDeclaration)*
    // We iterate all children to preserve the original declaration order.
    let mut local_schemas = Vec::new();
    for child in ctx.get_children() {
        if let Ok(import_ctx) = child
            .clone()
            .downcast_rc::<ImportStatementContextAll<'input>>()
        {
            collect_single_import(&import_ctx, decl_items);
        } else if let Ok(ns_ctx) = child.downcast_rc::<NamedSchemaDeclarationContextAll<'input>>() {
            let span = span_from_context(&*ns_ctx);
            let schema = walk_named_schema_no_register(&ns_ctx, token_stream, src, namespace)?;
            local_schemas.push(schema.clone());
            decl_items.push(DeclItem::Type(schema, span));
        }
    }

    // The main schema declaration uses `schema <fullType>;`.
    if let Some(main_ctx) = ctx.mainSchemaDeclaration()
        && let Some(ft_ctx) = main_ctx.fullType()
    {
        let schema = walk_full_type(&ft_ctx, token_stream, src, namespace)?;
        return Ok(IdlFile::Schema(schema));
    }

    // Return all locally-declared named schemas (possibly empty). Files with
    // only `namespace` and `import` statements yield an empty vec here; their
    // imported types are resolved later by `parse_and_resolve`. We intentionally
    // do NOT reject the empty case at parse time — that policy belongs to the
    // CLI layer (`Idl::convert_impl` rejects it for the `idl` subcommand, while
    // `Idl2Schemata::extract_impl` allows it, matching Java's behavior).
    Ok(IdlFile::NamedSchemas(local_schemas))
}

/// Walk a protocol declaration and return a complete `Protocol`.
///
/// Instead of registering types immediately, this function iterates the
/// protocol body's children in source order, appending `DeclItem::Import`
/// and `DeclItem::Type` entries to `decl_items`. Messages are collected
/// directly into the protocol since they don't affect type ordering.
fn walk_protocol<'input>(
    ctx: &ProtocolDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &mut Option<String>,
    decl_items: &mut Vec<DeclItem>,
) -> Result<Protocol> {
    // Extract doc comment by scanning hidden tokens before the context's start token.
    let doc = extract_doc_from_context(ctx, token_stream, src);

    // Process `@namespace(...)` and other schema properties on the protocol.
    let props =
        walk_schema_properties(&ctx.schemaProperty_all(), token_stream, src, PROTOCOL_PROPS)?;

    // Get the protocol name from the identifier.
    let name_ctx = ctx
        .identifier()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing protocol name"))?;
    let raw_identifier = identifier_text(&name_ctx);

    // Determine namespace: explicit `@namespace` overrides, otherwise if the
    // identifier contains dots, the part before the last dot is the namespace.
    *namespace = compute_namespace(&raw_identifier, &props.namespace);
    let protocol_name = extract_name(&raw_identifier);

    if INVALID_TYPE_NAMES.contains(&protocol_name.as_str()) {
        return Err(make_diagnostic(
            src,
            &*name_ctx,
            format!("Illegal name: {protocol_name}"),
        ));
    }

    // Build the protocol properties (custom annotations that aren't namespace/aliases/order).
    let protocol_properties = props.properties;

    // Walk the protocol body.
    let body = ctx
        .protocolDeclarationBody()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing protocol body"))?;

    // Walk the protocol body children in source order. The ANTLR grammar
    // interleaves imports, named schema declarations, and message declarations:
    //   protocolDeclarationBody: '{' (import | namedSchema | message)* '}'
    // We iterate all children and dispatch based on type, preserving the
    // original declaration order for imports and types.
    let mut messages = HashMap::new();
    for child in body.get_children() {
        if let Ok(import_ctx) = child
            .clone()
            .downcast_rc::<ImportStatementContextAll<'input>>()
        {
            collect_single_import(&import_ctx, decl_items);
        } else if let Ok(ns_ctx) = child
            .clone()
            .downcast_rc::<NamedSchemaDeclarationContextAll<'input>>()
        {
            let span = span_from_context(&*ns_ctx);
            let schema = walk_named_schema_no_register(&ns_ctx, token_stream, src, namespace)?;
            decl_items.push(DeclItem::Type(schema, span));
        } else if let Ok(msg_ctx) = child.downcast_rc::<MessageDeclarationContextAll<'input>>() {
            let (msg_name, message) = walk_message(&msg_ctx, token_stream, src, namespace)?;
            messages.insert(msg_name, message);
        }
    }

    // The types list in the Protocol is initially empty; the caller will
    // populate it from the registry after processing all DeclItems in order.
    Ok(Protocol {
        name: protocol_name,
        namespace: namespace.clone(),
        doc,
        properties: protocol_properties,
        types: Vec::new(),
        messages,
    })
}

/// Dispatch to record, enum, or fixed based on the named schema declaration.
///
/// This function parses the named schema but does NOT register it in a
/// SchemaRegistry. The caller is responsible for registration, which allows
/// imports and local types to be registered in source order.
fn walk_named_schema_no_register<'input>(
    ctx: &NamedSchemaDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &mut Option<String>,
) -> Result<AvroSchema> {
    if let Some(fixed_ctx) = ctx.fixedDeclaration() {
        walk_fixed(&fixed_ctx, token_stream, src, namespace)
    } else if let Some(enum_ctx) = ctx.enumDeclaration() {
        walk_enum(&enum_ctx, token_stream, src, namespace)
    } else if let Some(record_ctx) = ctx.recordDeclaration() {
        walk_record(&record_ctx, token_stream, src, namespace)
    } else {
        Err(make_diagnostic(
            src,
            ctx,
            "unknown named schema declaration",
        ))
    }
}

// ==========================================================================
// Record
// ==========================================================================

// NOTE: The ANTLR grammar's `recordBody` rule only permits `fieldDeclaration`
// children — it does not include `namedSchemaDeclaration`. Therefore
// `walk_record` does not need access to the schema registry. If the grammar
// is ever extended to allow nested named schema declarations inside records,
// a `registry: &mut SchemaRegistry` parameter would need to be added back.
fn walk_record<'input>(
    ctx: &RecordDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &mut Option<String>,
) -> Result<AvroSchema> {
    let doc = extract_doc_from_context(ctx, token_stream, src);
    let props = walk_schema_properties(
        &ctx.schemaProperty_all(),
        token_stream,
        src,
        NAMED_TYPE_PROPS,
    )?;

    let name_ctx = ctx
        .identifier()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing record name"))?;
    let raw_identifier = identifier_text(&name_ctx);

    // Determine if this is a record or an error type.
    let is_error = ctx
        .recordType
        .as_ref()
        .is_some_and(|tok| tok.get_token_type() == Idl_Error);

    // Compute namespace: `@namespace` on the record overrides; otherwise
    // the identifier may contain dots, or we fall back to the enclosing namespace.
    let record_namespace =
        compute_namespace(&raw_identifier, &props.namespace).or_else(|| namespace.clone());
    let record_name = extract_name(&raw_identifier);

    if INVALID_TYPE_NAMES.contains(&record_name.as_str()) {
        return Err(make_diagnostic(
            src,
            &*name_ctx,
            format!("Illegal name: {record_name}"),
        ));
    }

    // Save and set the current namespace for field type resolution inside the
    // record body, then restore it afterwards.
    let saved_namespace = namespace.clone();
    if record_namespace.is_some() {
        *namespace = record_namespace.clone();
    }

    // Walk the record body to get fields.
    let body = ctx
        .recordBody()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing record body"))?;

    let mut fields = Vec::new();
    let mut seen_field_names: HashSet<String> = HashSet::new();
    for field_ctx in body.fieldDeclaration_all() {
        let mut field_fields = walk_field_declaration(&field_ctx, token_stream, src, namespace)?;
        // Check for duplicates. We zip with the variable declaration contexts
        // so that the diagnostic highlights the duplicate field *name*, not the
        // type keyword that starts the field declaration.
        let var_ctxs = field_ctx.variableDeclaration_all();
        for (field, var_ctx) in field_fields.iter().zip(var_ctxs.iter()) {
            if !seen_field_names.insert(field.name.clone()) {
                *namespace = saved_namespace;
                let name_ctx = var_ctx.identifier();
                let diag = if let Some(name_ctx) = name_ctx {
                    make_diagnostic(
                        src,
                        &*name_ctx,
                        format!(
                            "duplicate field '{}' in record '{}'",
                            field.name, record_name
                        ),
                    )
                } else {
                    make_diagnostic(
                        src,
                        &*field_ctx,
                        format!(
                            "duplicate field '{}' in record '{}'",
                            field.name, record_name
                        ),
                    )
                };
                return Err(diag);
            }
        }
        fields.append(&mut field_fields);
    }

    // Restore namespace.
    *namespace = saved_namespace;

    Ok(AvroSchema::Record {
        name: record_name,
        namespace: record_namespace,
        doc,
        fields,
        is_error,
        aliases: props.aliases,
        properties: props.properties,
    })
}

// ==========================================================================
// Field Declaration
// ==========================================================================

/// Walk a field declaration, which has one fullType and one or more variable
/// declarations sharing that type.
fn walk_field_declaration<'input>(
    ctx: &FieldDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<Vec<Field>> {
    // The doc comment on the field declaration acts as a default for variables
    // that don't have their own doc comment.
    let default_doc = extract_doc_from_context(ctx, token_stream, src);

    // Walk the field type.
    let full_type_ctx = ctx
        .fullType()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing field type"))?;
    let field_type = walk_full_type(&full_type_ctx, token_stream, src, namespace)?;

    // Walk each variable declaration.
    let mut fields = Vec::new();
    for var_ctx in ctx.variableDeclaration_all() {
        let field = walk_variable(
            &var_ctx,
            &field_type,
            &default_doc,
            token_stream,
            src,
            namespace,
        )?;
        fields.push(field);
    }

    Ok(fields)
}

/// Walk a single variable declaration and create a `Field`.
fn walk_variable<'input>(
    ctx: &VariableDeclarationContextAll<'input>,
    field_type: &AvroSchema,
    default_doc: &Option<String>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    _namespace: &Option<String>,
) -> Result<Field> {
    // Variable-specific doc comment overrides the field-level default.
    let var_doc = extract_doc_from_context(ctx, token_stream, src);
    let doc = var_doc.or_else(|| default_doc.clone());

    let name_ctx = ctx
        .identifier()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing variable name"))?;
    let field_name = identifier_text(&name_ctx);

    // Walk the variable-level schema properties (e.g. @order, @aliases on a
    // specific variable rather than on the field type).
    let props =
        walk_schema_properties(&ctx.schemaProperty_all(), token_stream, src, VARIABLE_PROPS)?;

    // Parse the default value if present.
    let default_value = if let Some(json_ctx) = ctx.jsonValue() {
        Some(
            walk_json_value(&json_ctx, token_stream, src)
                .wrap_err_with(|| format!("parse default value for field `{field_name}`"))?,
        )
    } else {
        None
    };

    // Apply fixOptionalSchema: if the type is a nullable union (from `type?`)
    // and the default is non-null, reorder to put the non-null type first.
    let final_type = fix_optional_schema(field_type.clone(), &default_value);

    // Java's fixDefaultValue coerces IntNode to LongNode when the field type is
    // `long`. No equivalent is needed here: serde_json::Number uses a single
    // internal representation (u64/i64/f64), so `to_value(42_i32)` and
    // `to_value(42_i64)` produce the same Value::Number. The coercion is
    // implicit and lossless.

    // Validate that the default value's JSON type matches the field's Avro type.
    // This catches mismatches like `int count = "hello"` at compile time, matching
    // Java's `Schema.Field` constructor behavior with `validate=true`.
    if let Some(ref default_val) = default_value
        && let Some(reason) = validate_default(default_val, &final_type)
    {
        return Err(make_diagnostic(
            src,
            ctx,
            format!("Invalid default for field `{field_name}`: {reason}"),
        ));
    }

    Ok(Field {
        name: field_name,
        schema: final_type,
        doc,
        default: default_value,
        order: props.order,
        aliases: props.aliases,
        properties: props.properties,
    })
}

// ==========================================================================
// Enum
// ==========================================================================

fn walk_enum<'input>(
    ctx: &EnumDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    enclosing_namespace: &Option<String>,
) -> Result<AvroSchema> {
    let doc = extract_doc_from_context(ctx, token_stream, src);
    let props = walk_schema_properties(&ctx.schemaProperty_all(), token_stream, src, ENUM_PROPS)?;

    let name_ctx = ctx
        .identifier()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing enum name"))?;
    let raw_identifier = identifier_text(&name_ctx);

    // If compute_namespace returns None (no explicit @namespace and no dots
    // in the identifier), fall back to the enclosing namespace.
    let enum_namespace = compute_namespace(&raw_identifier, &props.namespace)
        .or_else(|| enclosing_namespace.clone());
    let enum_name = extract_name(&raw_identifier);

    if INVALID_TYPE_NAMES.contains(&enum_name.as_str()) {
        return Err(make_diagnostic(
            src,
            &*name_ctx,
            format!("Illegal name: {enum_name}"),
        ));
    }

    // Collect enum symbols, rejecting duplicates.
    let mut symbols = Vec::new();
    let mut seen_symbols: HashSet<String> = HashSet::new();
    for sym_ctx in ctx.enumSymbol_all() {
        if let Some(sym_name_ctx) = sym_ctx.identifier() {
            let sym_name = identifier_text(&sym_name_ctx);
            if !seen_symbols.insert(sym_name.clone()) {
                return Err(make_diagnostic(
                    src,
                    &*sym_ctx,
                    format!("duplicate enum symbol: {sym_name}"),
                ));
            }
            symbols.push(sym_name);
        }
    }

    // Get the default symbol if present (via `= symbolName;` after the closing brace).
    // Validate that it exists in the symbol list (Java's `EnumSchema` constructor
    // rejects unknown defaults with `SchemaParseException`).
    let default_symbol = if let Some(default_ctx) = ctx.enumDefault() {
        if let Some(id_ctx) = default_ctx.identifier() {
            let sym = identifier_text(&id_ctx);
            if !symbols.contains(&sym) {
                return Err(make_diagnostic(
                    src,
                    &*id_ctx,
                    format!(
                        "The Enum Default: {} is not in the enum symbol set: {:?}",
                        sym, symbols
                    ),
                ));
            }
            Some(sym)
        } else {
            None
        }
    } else {
        None
    };

    Ok(AvroSchema::Enum {
        name: enum_name,
        namespace: enum_namespace,
        doc,
        symbols,
        default: default_symbol,
        aliases: props.aliases,
        properties: props.properties,
    })
}

// ==========================================================================
// Fixed
// ==========================================================================

fn walk_fixed<'input>(
    ctx: &FixedDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    enclosing_namespace: &Option<String>,
) -> Result<AvroSchema> {
    let doc = extract_doc_from_context(ctx, token_stream, src);
    let props = walk_schema_properties(
        &ctx.schemaProperty_all(),
        token_stream,
        src,
        NAMED_TYPE_PROPS,
    )?;

    let name_ctx = ctx
        .identifier()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing fixed name"))?;
    let raw_identifier = identifier_text(&name_ctx);

    // Fall back to enclosing namespace if no explicit namespace is given.
    let fixed_namespace = compute_namespace(&raw_identifier, &props.namespace)
        .or_else(|| enclosing_namespace.clone());
    let fixed_name = extract_name(&raw_identifier);

    if INVALID_TYPE_NAMES.contains(&fixed_name.as_str()) {
        return Err(make_diagnostic(
            src,
            &*name_ctx,
            format!("Illegal name: {fixed_name}"),
        ));
    }

    // Parse the size from the IntegerLiteral token.
    let size_tok = ctx
        .size
        .as_ref()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing fixed size"))?;
    let size = parse_integer_as_u32(size_tok.get_text()).map_err(|e| {
        make_diagnostic_from_token(
            src,
            &**size_tok,
            format!("invalid fixed size for `{fixed_name}`: {e}"),
        )
    })?;

    Ok(AvroSchema::Fixed {
        name: fixed_name,
        namespace: fixed_namespace,
        doc,
        size,
        aliases: props.aliases,
        properties: props.properties,
    })
}

// ==========================================================================
// Type Walking
// ==========================================================================

/// Walk a `fullType` node: collect schema properties, walk the inner
/// `plainType`, then apply any custom properties to the resulting schema.
fn walk_full_type<'input>(
    ctx: &FullTypeContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    let props = walk_schema_properties(&ctx.schemaProperty_all(), token_stream, src, BARE_PROPS)?;

    let plain_ctx = ctx
        .plainType()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing plain type in fullType"))?;

    let schema = walk_plain_type(&plain_ctx, token_stream, src, namespace)?;

    // Type references may not be annotated. When the resolved type is a bare
    // reference to a previously-defined named type, any accumulated schema
    // properties (from annotations like `@foo("bar")`) are semantically invalid
    // -- the annotation is ambiguous (does it apply to the field or the type?).
    // The Java implementation checks this in exitNullableType (IdlReader.java
    // lines 776-777) and throws "Type references may not be annotated".
    if !props.properties.is_empty() && is_type_reference(&schema) {
        return Err(make_diagnostic(
            src,
            ctx,
            "Type references may not be annotated",
        ));
    }

    // Apply custom properties to the schema. For nullable unions we apply
    // properties to the non-null branch (matching the Java behavior).
    let schema = if !props.properties.is_empty() {
        apply_properties(schema, props.properties)
    } else {
        schema
    };

    Ok(schema)
}

/// Dispatch to array, map, union, or nullable type.
fn walk_plain_type<'input>(
    ctx: &PlainTypeContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    if let Some(array_ctx) = ctx.arrayType() {
        return walk_array_type(&array_ctx, token_stream, src, namespace);
    }
    if let Some(map_ctx) = ctx.mapType() {
        return walk_map_type(&map_ctx, token_stream, src, namespace);
    }
    if let Some(union_ctx) = ctx.unionType() {
        return walk_union_type(&union_ctx, token_stream, src, namespace);
    }
    if let Some(nullable_ctx) = ctx.nullableType() {
        return walk_nullable_type(&nullable_ctx, token_stream, src, namespace);
    }
    Err(make_diagnostic(src, ctx, "unrecognized plain type"))
}

/// Walk a nullable type: either a primitive type or a named reference,
/// optionally followed by `?` to make it nullable.
fn walk_nullable_type<'input>(
    ctx: &NullableTypeContextAll<'input>,
    _token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    let base_type = if let Some(prim_ctx) = ctx.primitiveType() {
        walk_primitive_type(&prim_ctx, src)?
    } else if let Some(ref_ctx) = ctx.identifier() {
        // Named type reference. Split the identifier into name and namespace
        // so the Reference carries them separately, enabling correct namespace
        // shortening during JSON serialization.
        let type_name = identifier_text(&ref_ctx);
        let ref_span = span_from_context(&*ref_ctx);
        if let Some((ns, name)) = type_name.rsplit_once('.') {
            AvroSchema::Reference {
                name: name.to_string(),
                namespace: Some(ns.to_string()),
                properties: HashMap::new(),
                span: ref_span,
            }
        } else {
            AvroSchema::Reference {
                name: type_name.to_string(),
                namespace: namespace.clone(),
                properties: HashMap::new(),
                span: ref_span,
            }
        }
    } else {
        return Err(make_diagnostic(src, ctx, "nullable type has no inner type"));
    };

    // If the `?` token is present, wrap in a nullable union `[null, T]`.
    // Reject `null?` because it would produce the invalid union `[null, null]`
    // (Avro requires each type in a union to be unique). Java also rejects this.
    if ctx.optional.is_some() {
        if matches!(base_type, AvroSchema::Null) {
            return Err(make_diagnostic(
                src,
                ctx,
                "`null` type cannot be made nullable",
            ));
        }
        Ok(AvroSchema::Union {
            types: vec![AvroSchema::Null, base_type],
            is_nullable_type: true,
        })
    } else {
        Ok(base_type)
    }
}

/// Walk a primitive type keyword and return the corresponding `AvroSchema`.
fn walk_primitive_type<'input>(
    ctx: &PrimitiveTypeContextAll<'input>,
    src: &SourceInfo<'_>,
) -> Result<AvroSchema> {
    let type_tok = ctx
        .typeName
        .as_ref()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing primitive type name"))?;
    let token_type = type_tok.get_token_type();

    let schema = match token_type {
        Idl_Boolean => AvroSchema::Boolean,
        Idl_Int => AvroSchema::Int,
        Idl_Long => AvroSchema::Long,
        Idl_Float => AvroSchema::Float,
        Idl_Double => AvroSchema::Double,
        Idl_Bytes => AvroSchema::Bytes,
        Idl_String => AvroSchema::String,
        Idl_Null => AvroSchema::Null,
        Idl_Date => AvroSchema::Logical {
            logical_type: LogicalType::Date,
            properties: HashMap::new(),
        },
        Idl_Time => AvroSchema::Logical {
            logical_type: LogicalType::TimeMillis,
            properties: HashMap::new(),
        },
        Idl_Timestamp => AvroSchema::Logical {
            logical_type: LogicalType::TimestampMillis,
            properties: HashMap::new(),
        },
        Idl_LocalTimestamp => AvroSchema::Logical {
            logical_type: LogicalType::LocalTimestampMillis,
            properties: HashMap::new(),
        },
        Idl_UUID => AvroSchema::Logical {
            logical_type: LogicalType::Uuid,
            properties: HashMap::new(),
        },
        Idl_Decimal => {
            // decimal(precision [, scale])
            let precision_tok = ctx
                .precision
                .as_ref()
                .ok_or_else(|| make_diagnostic(src, ctx, "decimal type missing precision"))?;
            let precision = parse_integer_as_u32(precision_tok.get_text()).map_err(|e| {
                make_diagnostic_from_token(
                    src,
                    &**precision_tok,
                    format!("invalid decimal precision: {e}"),
                )
            })?;

            // The Avro spec requires precision to be a positive integer.
            if precision == 0 {
                return Err(make_diagnostic_from_token(
                    src,
                    &**precision_tok,
                    "invalid decimal precision: 0 (must be positive)".to_string(),
                ));
            }

            let scale = if let Some(scale_tok) = ctx.scale.as_ref() {
                parse_integer_as_u32(scale_tok.get_text()).map_err(|e| {
                    make_diagnostic_from_token(
                        src,
                        &**scale_tok,
                        format!("invalid decimal scale: {e}"),
                    )
                })?
            } else {
                0
            };

            // The Avro spec requires scale to not exceed precision.
            if scale > precision {
                return Err(make_diagnostic_from_token(
                    src,
                    &**ctx
                        .scale
                        .as_ref()
                        .expect("scale token present when scale > 0"),
                    format!(
                        "invalid decimal scale: {scale} \
                         (greater than precision: {precision})"
                    ),
                ));
            }

            AvroSchema::Logical {
                logical_type: LogicalType::Decimal { precision, scale },
                properties: HashMap::new(),
            }
        }
        _ => {
            return Err(make_diagnostic_from_token(
                src,
                type_tok.as_ref(),
                format!("unexpected primitive type token: {token_type}"),
            ));
        }
    };

    Ok(schema)
}

/// Walk `array<fullType>`.
fn walk_array_type<'input>(
    ctx: &ArrayTypeContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    let element_ctx = ctx
        .fullType()
        .ok_or_else(|| make_diagnostic(src, ctx, "array type missing element type"))?;
    let items = walk_full_type(&element_ctx, token_stream, src, namespace)?;
    Ok(AvroSchema::Array {
        items: Box::new(items),
        properties: HashMap::new(),
    })
}

/// Walk `map<fullType>`.
fn walk_map_type<'input>(
    ctx: &MapTypeContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    let value_ctx = ctx
        .fullType()
        .ok_or_else(|| make_diagnostic(src, ctx, "map type missing value type"))?;
    let values = walk_full_type(&value_ctx, token_stream, src, namespace)?;
    Ok(AvroSchema::Map {
        values: Box::new(values),
        properties: HashMap::new(),
    })
}

/// Walk `union { fullType, fullType, ... }`.
fn walk_union_type<'input>(
    ctx: &UnionTypeContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    let mut types = Vec::new();
    for ft_ctx in ctx.fullType_all() {
        types.push(walk_full_type(&ft_ctx, token_stream, src, namespace)?);
    }

    // Reject nested unions (Avro spec: "Unions may not immediately contain
    // other unions").
    let ft_ctxs = ctx.fullType_all();
    for (i, t) in types.iter().enumerate() {
        if matches!(t, AvroSchema::Union { .. }) {
            return Err(make_diagnostic(
                src,
                &*ft_ctxs[i],
                "Unions may not immediately contain other unions \
                 (per the Avro specification, §schemas). Note: Java avro-tools \
                 incorrectly accepts this syntax, producing an empty union.",
            ));
        }
    }

    // Reject duplicate types (Avro spec: "Unions may not contain more than
    // one schema with the same type, except for the named types record, enum
    // and fixed"). For anonymous types the key is the type name; for named
    // types the key is the fully qualified name.
    let mut seen_keys: HashSet<String> = HashSet::new();
    for (i, t) in types.iter().enumerate() {
        let key = t.union_type_key();
        if !seen_keys.insert(key.clone()) {
            return Err(make_diagnostic(
                src,
                &*ft_ctxs[i],
                format!("Duplicate in union: {key}"),
            ));
        }
    }

    Ok(AvroSchema::Union {
        types,
        is_nullable_type: false,
    })
}

// ==========================================================================
// Message Declaration
// ==========================================================================

fn walk_message<'input>(
    ctx: &MessageDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<(String, Message)> {
    let doc = extract_doc_from_context(ctx, token_stream, src);
    let props =
        walk_schema_properties(&ctx.schemaProperty_all(), token_stream, src, MESSAGE_PROPS)?;

    // Walk the result type. `void` maps to Null.
    let result_ctx = ctx
        .resultType()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing message return type"))?;
    let response = walk_result_type(&result_ctx, token_stream, src, namespace)?;

    // When the return type is a named type reference, any message-level
    // annotations are ambiguous (do they apply to the message or to the
    // type?). Java's `exitNullableType` rejects this combination with
    // "Type references may not be annotated". We mirror that check here.
    if !props.properties.is_empty() && is_type_reference(&response) {
        return Err(make_diagnostic(
            src,
            ctx,
            "Type references may not be annotated",
        ));
    }

    // The message name is stored in the `name` field of the context ext.
    let name_ctx = ctx
        .name
        .as_ref()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing message name"))?;
    let message_name = identifier_text(name_ctx);

    // Walk formal parameters.
    let mut request_fields = Vec::new();
    let mut seen_param_names: HashSet<String> = HashSet::new();
    for param_ctx in ctx.formalParameter_all() {
        let param_doc = extract_doc_from_context(&*param_ctx, token_stream, src);

        let ft_ctx = param_ctx
            .fullType()
            .ok_or_else(|| make_diagnostic(src, &*param_ctx, "missing parameter type"))?;
        let param_type = walk_full_type(&ft_ctx, token_stream, src, namespace)?;

        let var_ctx = param_ctx
            .variableDeclaration()
            .ok_or_else(|| make_diagnostic(src, &*param_ctx, "missing parameter variable"))?;
        let field = walk_variable(
            &var_ctx,
            &param_type,
            &param_doc,
            token_stream,
            src,
            namespace,
        )?;
        if !seen_param_names.insert(field.name.clone()) {
            return Err(make_diagnostic(
                src,
                &*param_ctx,
                format!(
                    "duplicate parameter '{}' in message '{}'",
                    field.name, message_name
                ),
            ));
        }
        request_fields.push(field);
    }

    // Check for oneway.
    let one_way = ctx.oneway.is_some();

    // One-way messages must return void (AvroSchema::Null). The Avro specification
    // requires one-way messages to have a null response and no errors. The Java
    // implementation checks this in exitMessageDeclaration (IdlReader.java line 715).
    if one_way && response != AvroSchema::Null {
        return Err(make_diagnostic(
            src,
            ctx,
            format!("One-way message '{}' must return void", message_name),
        ));
    }

    // Check for throws clause. The `errors` field on the context ext struct
    // contains only the error type identifiers (not the message name).
    let errors = if !ctx.errors.is_empty() {
        let mut error_schemas = Vec::new();
        for error_id_ctx in &ctx.errors {
            let error_name = identifier_text(error_id_ctx);
            let error_span = span_from_context(&**error_id_ctx);
            if let Some((ns, name)) = error_name.rsplit_once('.') {
                error_schemas.push(AvroSchema::Reference {
                    name: name.to_string(),
                    namespace: Some(ns.to_string()),
                    properties: HashMap::new(),
                    span: error_span,
                });
            } else {
                error_schemas.push(AvroSchema::Reference {
                    name: error_name.to_string(),
                    namespace: namespace.clone(),
                    properties: HashMap::new(),
                    span: error_span,
                });
            }
        }
        Some(error_schemas)
    } else if one_way {
        // One-way messages have no error declarations.
        None
    } else {
        // Non-throwing messages omit the errors key entirely in the JSON
        // output. The Java Avro tools only emit `"errors"` when the message
        // explicitly declares `throws`.
        None
    };

    Ok((
        message_name,
        Message {
            doc,
            properties: props.properties,
            request: request_fields,
            response,
            errors,
            one_way,
        },
    ))
}

/// Walk a `resultType`: either `void` (produces Null) or a `plainType`.
fn walk_result_type<'input>(
    ctx: &ResultTypeContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    // If there's a Void token, return Null.
    if ctx.Void().is_some() {
        return Ok(AvroSchema::Null);
    }
    // Otherwise walk the plainType child.
    if let Some(plain_ctx) = ctx.plainType() {
        return walk_plain_type(&plain_ctx, token_stream, src, namespace);
    }
    // Fallback: void.
    Ok(AvroSchema::Null)
}

// ==========================================================================
// JSON Value Walking
// ==========================================================================

fn walk_json_value<'input>(
    ctx: &JsonValueContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
) -> Result<Value> {
    if let Some(obj_ctx) = ctx.jsonObject() {
        return walk_json_object(&obj_ctx, token_stream, src);
    }
    if let Some(arr_ctx) = ctx.jsonArray() {
        return walk_json_array(&arr_ctx, token_stream, src);
    }
    if let Some(lit_ctx) = ctx.jsonLiteral() {
        return walk_json_literal(&lit_ctx, src);
    }
    Err(make_diagnostic(src, ctx, "empty JSON value"))
}

fn walk_json_literal<'input>(
    ctx: &JsonLiteralContextAll<'input>,
    src: &SourceInfo<'_>,
) -> Result<Value> {
    let tok = ctx
        .literal
        .as_ref()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing JSON literal token"))?;
    let token_type = tok.get_token_type();
    let text = tok.get_text();

    match token_type {
        Idl_Null => Ok(Value::Null),
        Idl_BTrue => Ok(Value::Bool(true)),
        Idl_BFalse => Ok(Value::Bool(false)),
        Idl_StringLiteral => {
            let unescaped = get_string_from_literal(text);
            Ok(Value::String(unescaped))
        }
        Idl_IntegerLiteral => parse_integer_literal(text).map_err(|e| {
            make_diagnostic_from_token(src, tok.as_ref(), format!("invalid integer literal: {e}"))
        }),
        Idl_FloatingPointLiteral => parse_floating_point_literal(text).map_err(|e| {
            make_diagnostic_from_token(
                src,
                tok.as_ref(),
                format!("invalid floating-point literal: {e}"),
            )
        }),
        _ => Err(make_diagnostic_from_token(
            src,
            tok.as_ref(),
            format!("unexpected JSON literal token type: {token_type}"),
        )),
    }
}

fn walk_json_object<'input>(
    ctx: &JsonObjectContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
) -> Result<Value> {
    let mut map = serde_json::Map::new();
    for pair_ctx in ctx.jsonPair_all() {
        let key_tok = pair_ctx
            .name
            .as_ref()
            .ok_or_else(|| make_diagnostic(src, &*pair_ctx, "missing JSON object key"))?;
        let key = get_string_from_literal(key_tok.get_text());

        let value_ctx = pair_ctx
            .jsonValue()
            .ok_or_else(|| make_diagnostic(src, &*pair_ctx, "missing JSON object value"))?;
        let value = walk_json_value(&value_ctx, token_stream, src)?;

        map.insert(key, value);
    }
    Ok(Value::Object(map))
}

fn walk_json_array<'input>(
    ctx: &JsonArrayContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
) -> Result<Value> {
    let mut elements = Vec::new();
    for val_ctx in ctx.jsonValue_all() {
        elements.push(walk_json_value(&val_ctx, token_stream, src)?);
    }
    Ok(Value::Array(elements))
}

// ==========================================================================
// Helper Functions
// ==========================================================================

/// Extract the text from an `IdentifierContext`, removing backtick escapes.
fn identifier_text<'input>(ctx: &IdentifierContextAll<'input>) -> String {
    // The generated parser stores the matched token in `ctx.word`.
    // We use `get_text()` on the context itself as a reliable fallback.
    let text = ctx.get_text();
    text.replace('`', "")
}

/// Strip surrounding quotes from a string literal and unescape Java-style
/// escape sequences.
fn get_string_from_literal(raw: &str) -> String {
    // Strip surrounding quotes (either `"..."` or `'...'`).
    if raw.len() < 2 {
        return raw.to_string();
    }
    let inner = &raw[1..raw.len() - 1];
    unescape_java(inner)
}

/// Unescape Java-style string escape sequences.
fn unescape_java(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('b') => result.push('\u{0008}'),
                Some('f') => result.push('\u{000C}'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                Some('u') => {
                    // Unicode escape: \u+XXXX (one or more 'u' characters
                    // followed by exactly four hex digits). The extra 'u'
                    // characters are a Java-ism that some IDL files use.
                    while chars.peek() == Some(&'u') {
                        chars.next();
                    }
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(code_point) = u32::from_str_radix(&hex, 16) {
                        // Surrogate pair handling: Java's `StringEscapeUtils.unescapeJava`
                        // combines a high surrogate (\uD800-\uDBFF) followed immediately
                        // by a low surrogate (\uDC00-\uDFFF) into a single supplementary
                        // code point. We replicate that here.
                        if (0xD800..=0xDBFF).contains(&code_point) {
                            // High surrogate — peek ahead for a \uXXXX low surrogate.
                            let combined = try_parse_low_surrogate(&mut chars).map(|low| {
                                (code_point - 0xD800) * 0x400 + (low - 0xDC00) + 0x10000
                            });
                            if let Some(ch) = combined.and_then(char::from_u32) {
                                result.push(ch);
                            } else {
                                // Not followed by a valid low surrogate; emit the
                                // raw high-surrogate escape as-is.
                                result.push_str("\\u");
                                result.push_str(&hex);
                            }
                        } else if let Some(ch) = char::from_u32(code_point) {
                            result.push(ch);
                        } else {
                            // Invalid code point; emit the raw escape.
                            result.push_str("\\u");
                            result.push_str(&hex);
                        }
                    } else {
                        result.push_str("\\u");
                        result.push_str(&hex);
                    }
                }
                Some(c2) if ('0'..='7').contains(&c2) => {
                    // Octal escape: 1-3 octal digits. The grammar allows:
                    //   OctDigit OctDigit?          (1-2 digits, any octal)
                    //   [0-3] OctDigit OctDigit     (3 digits, first must be 0-3)
                    // This means a 3-digit sequence is only valid if the first
                    // digit is 0-3 (keeping the value <= \377 = 255).
                    let mut octal = String::new();
                    octal.push(c2);
                    if let Some(&next) = chars.peek()
                        && ('0'..='7').contains(&next)
                    {
                        octal.push(next);
                        chars.next();
                        // Only consume a third digit if the first was 0-3.
                        if c2 <= '3'
                            && let Some(&next2) = chars.peek()
                            && ('0'..='7').contains(&next2)
                        {
                            octal.push(next2);
                            chars.next();
                        }
                    }
                    if let Ok(val) = u32::from_str_radix(&octal, 8) {
                        if let Some(ch) = char::from_u32(val) {
                            result.push(ch);
                        } else {
                            result.push('\\');
                            result.push_str(&octal);
                        }
                    } else {
                        result.push('\\');
                        result.push_str(&octal);
                    }
                }
                Some(other) => {
                    // Unknown escape; keep as-is.
                    result.push('\\');
                    result.push(other);
                }
                None => {
                    // Trailing backslash.
                    result.push('\\');
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Try to consume a `\uXXXX` low-surrogate escape from the iterator.
///
/// Called after we've already parsed a high surrogate. If the next characters
/// are `\u` (with optional extra `u`s) followed by four hex digits in the
/// low-surrogate range (U+DC00..U+DFFF), consumes them and returns the code
/// point. Otherwise leaves the iterator untouched and returns `None`.
fn try_parse_low_surrogate(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<u32> {
    // We need to speculatively consume characters and back out if the
    // sequence is not a valid low surrogate. Collect consumed chars so we
    // can push them back (via a small buffer) on failure.
    //
    // Peekable only lets us peek one character ahead, so we clone the
    // iterator to look ahead without consuming from the original.
    let saved = chars.clone();

    // Expect '\\'.
    if chars.next() != Some('\\') {
        *chars = saved;
        return None;
    }
    // Expect 'u' (one or more).
    if chars.next() != Some('u') {
        *chars = saved;
        return None;
    }
    // Skip additional 'u' characters (the Java multi-u idiom).
    while chars.peek() == Some(&'u') {
        chars.next();
    }
    // Read exactly four hex digits.
    let hex: String = chars.by_ref().take(4).collect();
    if hex.len() != 4 {
        *chars = saved;
        return None;
    }
    if let Ok(low) = u32::from_str_radix(&hex, 16)
        && (0xDC00..=0xDFFF).contains(&low)
    {
        return Some(low);
    }
    // Not a low surrogate — restore the iterator to where we started so
    // the caller can process these characters normally.
    *chars = saved;
    None
}

/// Parse an integer literal (from a JSON or schema context).
///
/// Handles: decimal, hex (`0x`/`0X`), octal (`0` prefix), `L`/`l` suffix,
/// and underscore separators. Returns `Value::Number` (i32 if it fits, i64
/// otherwise).
fn parse_integer_literal(text: &str) -> Result<Value> {
    let mut number = text.replace('_', "");

    // Check for long suffix.
    let coerce_to_long = if number.ends_with('l') || number.ends_with('L') {
        number.pop();
        true
    } else {
        false
    };

    // Parse the number. Java's `Long.decode` handles "0x", "0X", "#", and
    // octal (leading "0"). We replicate that logic.
    let long_value: i64 = if number.starts_with("0x") || number.starts_with("0X") {
        let hex = &number[2..];
        i64::from_str_radix(hex, 16)
            .map_err(|e| miette::miette!("invalid hex integer literal '{text}': {e}"))?
    } else if number.starts_with('-') && (number.starts_with("-0x") || number.starts_with("-0X")) {
        let hex = &number[3..];
        let abs = i64::from_str_radix(hex, 16)
            .map_err(|e| miette::miette!("invalid hex integer literal '{text}': {e}"))?;
        -abs
    } else if number.starts_with('0') && number.len() > 1 && !number.contains('.') {
        // Octal.
        i64::from_str_radix(&number, 8)
            .map_err(|e| miette::miette!("invalid octal integer literal '{text}': {e}"))?
    } else if number.starts_with("-0") && number.len() > 2 && !number.contains('.') {
        let oct = &number[1..];
        let abs = i64::from_str_radix(oct, 8)
            .map_err(|e| miette::miette!("invalid octal integer literal '{text}': {e}"))?;
        -abs
    } else {
        number
            .parse::<i64>()
            .map_err(|e| miette::miette!("invalid integer literal '{text}': {e}"))?
    };

    let int_value = long_value as i32;
    if coerce_to_long || int_value as i64 != long_value {
        // Doesn't fit in i32 or explicitly long -- use i64.
        Ok(serde_json::to_value(long_value)
            .map_err(|e| miette::miette!("JSON number error: {e}"))?)
    } else {
        Ok(serde_json::to_value(int_value)
            .map_err(|e| miette::miette!("JSON number error: {e}"))?)
    }
}

/// Parse a floating point literal. NaN and Infinity become `Value::String`
/// because they are not valid JSON numbers.
///
/// The ANTLR grammar's `FloatingPointLiteral` rule allows Java-style type
/// suffixes (`f`, `F`, `d`, `D`) and hexadecimal floating-point literals
/// (`0x1.0p10`). Rust's `f64::from_str` handles neither, so we strip suffixes
/// and parse hex floats manually before falling through to the standard path.
fn parse_floating_point_literal(text: &str) -> Result<Value> {
    let val: f64 = parse_float_text(text)?;

    if val.is_nan() {
        Ok(Value::String("NaN".to_string()))
    } else if val.is_infinite() {
        if val.is_sign_positive() {
            Ok(Value::String("Infinity".to_string()))
        } else {
            Ok(Value::String("-Infinity".to_string()))
        }
    } else {
        Ok(serde_json::Number::from_f64(val)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(text.to_string())))
    }
}

/// Inner parsing logic for floating-point literal text. Handles:
/// - Optional leading sign (`+`/`-`)
/// - NaN and Infinity literals
/// - Java-style type suffixes (`f`/`F`/`d`/`D`), stripped before parsing
/// - Hex floating-point literals (`0x1.8p10` = 1.5 * 2^10 = 1536.0)
/// - Standard decimal floats and scientific notation
fn parse_float_text(text: &str) -> Result<f64> {
    // NaN and Infinity are handled directly — they never carry suffixes
    // in the grammar.
    if text == "NaN" || text == "+NaN" {
        return Ok(f64::NAN);
    }
    if text == "-NaN" {
        return Ok(-f64::NAN);
    }
    if text == "Infinity" || text == "+Infinity" {
        return Ok(f64::INFINITY);
    }
    if text == "-Infinity" {
        return Ok(f64::NEG_INFINITY);
    }

    // Strip trailing Java type suffix (f/F/d/D). These are permitted by
    // the ANTLR grammar but have no semantic effect — all values are
    // treated as f64, matching Java's `Double.parseDouble` behavior.
    let number = if text.ends_with(['f', 'F', 'd', 'D']) {
        &text[..text.len() - 1]
    } else {
        text
    };

    // Detect hex floating-point literals by looking for 0x/0X prefix
    // (after an optional sign). The format from the grammar is:
    //   [+-]? '0' [xX] <hex-mantissa> [pP] [+-]? <decimal-exponent>
    let (sign, unsigned) = match number.strip_prefix('-') {
        Some(rest) => (-1.0_f64, rest),
        None => (1.0_f64, number.strip_prefix('+').unwrap_or(number)),
    };

    if unsigned.starts_with("0x") || unsigned.starts_with("0X") {
        let hex_body = &unsigned[2..];
        let mantissa = parse_hex_float_mantissa_and_exponent(hex_body, text)?;
        return Ok(sign * mantissa);
    }

    // Standard decimal float — Rust's f64::from_str handles this directly.
    number
        .parse::<f64>()
        .map_err(|e| miette::miette!("invalid floating point literal '{text}': {e}"))
}

/// Parse the body of a hex floating-point literal (everything after the `0x`
/// prefix). The format is `<hex-mantissa> p <signed-decimal-exponent>`, where
/// the hex mantissa can contain a `.` separating integer and fractional hex
/// digits. The value is `mantissa * 2^exponent`.
///
/// Examples:
/// - `1.0p10`  -> 1.0 * 2^10  = 1024.0
/// - `1.8p1`   -> 1.5 * 2^1   = 3.0
/// - `Ap3`     -> 10.0 * 2^3  = 80.0
/// - `.8p1`    -> 0.5 * 2^1   = 1.0
/// - `1.p0`    -> 1.0 * 2^0   = 1.0
fn parse_hex_float_mantissa_and_exponent(hex_body: &str, original: &str) -> Result<f64> {
    // Split on the binary exponent marker (p/P). The grammar guarantees
    // exactly one is present.
    let (mantissa_str, exp_str) = hex_body.split_once(['p', 'P']).ok_or_else(|| {
        miette::miette!("invalid hex float literal '{original}': missing 'p'/'P' exponent")
    })?;

    // Parse the binary exponent (decimal integer, possibly signed).
    let exponent: i32 = exp_str
        .parse()
        .map_err(|e| miette::miette!("invalid hex float exponent in '{original}': {e}"))?;

    // Parse the hex mantissa, which may contain a '.' decimal point.
    let mantissa = if let Some((int_part, frac_part)) = mantissa_str.split_once('.') {
        // Integer part: each hex digit contributes its value in the
        // corresponding hex place.
        let int_val = if int_part.is_empty() {
            0.0
        } else {
            u64::from_str_radix(int_part, 16)
                .map_err(|e| miette::miette!("invalid hex float mantissa in '{original}': {e}"))?
                as f64
        };

        // Fractional part: each hex digit after the point represents
        // 1/16, 1/256, etc. of its value.
        let mut frac_val = 0.0_f64;
        let mut place = 1.0_f64 / 16.0;
        for ch in frac_part.chars() {
            let digit = ch.to_digit(16).ok_or_else(|| {
                miette::miette!("invalid hex digit '{ch}' in float literal '{original}'")
            })? as f64;
            frac_val += digit * place;
            place /= 16.0;
        }

        int_val + frac_val
    } else {
        // No decimal point — the mantissa is a plain hex integer.
        u64::from_str_radix(mantissa_str, 16)
            .map_err(|e| miette::miette!("invalid hex float mantissa in '{original}': {e}"))?
            as f64
    };

    Ok(mantissa * 2.0_f64.powi(exponent))
}

/// Parse an integer literal text into a u32 (for fixed size, decimal precision/scale).
fn parse_integer_as_u32(text: &str) -> Result<u32> {
    let number = text.replace('_', "");
    let value: u32 = if number.starts_with("0x") || number.starts_with("0X") {
        u32::from_str_radix(&number[2..], 16)
            .map_err(|e| miette::miette!("invalid integer '{text}': {e}"))?
    } else if number.starts_with('0') && number.len() > 1 {
        u32::from_str_radix(&number, 8)
            .map_err(|e| miette::miette!("invalid integer '{text}': {e}"))?
    } else {
        number
            .parse()
            .map_err(|e| miette::miette!("invalid integer '{text}': {e}"))?
    };
    Ok(value)
}

/// Given an identifier (which may contain dots like `com.example.MyType`),
/// extract just the name part (after the last dot).
fn extract_name(identifier: &str) -> String {
    match identifier.rfind('.') {
        Some(pos) => identifier[pos + 1..].to_string(),
        None => identifier.to_string(),
    }
}

/// Compute the effective namespace for a named type.
///
/// Priority:
/// 1. Explicit `@namespace("...")` annotation (passed as `explicit_namespace`).
/// 2. Dots in the identifier (the part before the last dot).
/// 3. The enclosing namespace (inherited from context -- not passed here,
///    the caller should fall back to the enclosing namespace if this returns None).
fn compute_namespace(identifier: &str, explicit_namespace: &Option<String>) -> Option<String> {
    // Java priority: dots in the identifier always take precedence over
    // an explicit `@namespace` annotation. Only when the identifier has
    // no dots do we fall back to `@namespace`.
    if let Some(pos) = identifier.rfind('.') {
        let ns = &identifier[..pos];
        return Some(ns.to_string());
    }

    explicit_namespace.clone()
}

/// Check whether a schema is a type reference (a bare name referring to a
/// previously-defined named type) or a nullable union wrapping a type reference.
/// Used to reject annotations on type references, matching the Java behavior.
fn is_type_reference(schema: &AvroSchema) -> bool {
    match schema {
        AvroSchema::Reference { .. } => true,
        // A nullable type reference (`MD5?`) wraps the reference in a union.
        AvroSchema::Union {
            types,
            is_nullable_type: true,
        } => types
            .iter()
            .any(|t| matches!(t, AvroSchema::Reference { .. })),
        _ => false,
    }
}

/// When `type?` creates a union `[null, T]` and the field's default is non-null,
/// reorder the union to `[T, null]` so that the default value matches the first
/// branch. This matches the Java `fixOptionalSchema` behavior.
fn fix_optional_schema(schema: AvroSchema, default_value: &Option<Value>) -> AvroSchema {
    match &schema {
        AvroSchema::Union {
            types,
            is_nullable_type: true,
        } if types.len() == 2 => {
            let non_null_default = match default_value {
                Some(Value::Null) | None => false,
                Some(_) => true,
            };

            if non_null_default {
                // Reorder: put the non-null type first, null second.
                let null_schema = types[0].clone();
                let non_null_schema = types[1].clone();
                AvroSchema::Union {
                    types: vec![non_null_schema, null_schema],
                    is_nullable_type: true,
                }
            } else {
                schema
            }
        }
        _ => schema,
    }
}

/// Apply custom schema properties to a schema. For nullable unions, apply them
/// to the non-null branch (matching the Java behavior where properties go on
/// `type.getTypes().get(1)` for optional types).
fn apply_properties(schema: AvroSchema, properties: HashMap<String, Value>) -> AvroSchema {
    match schema {
        AvroSchema::Union {
            types,
            is_nullable_type: true,
        } if types.len() == 2 => {
            // Apply properties to the non-null branch. We find it by type
            // rather than hardcoding index 1, because nullable unions may be
            // reordered to `[T, null]` when the field has a non-null default.
            let mut new_types = types;
            let non_null_idx = if matches!(new_types[0], AvroSchema::Null) {
                1
            } else {
                0
            };
            new_types[non_null_idx] =
                apply_properties_to_schema(new_types[non_null_idx].clone(), properties);
            AvroSchema::Union {
                types: new_types,
                is_nullable_type: true,
            }
        }
        other => apply_properties_to_schema(other, properties),
    }
}

/// Apply properties directly to a single schema node.
fn apply_properties_to_schema(
    schema: AvroSchema,
    properties: HashMap<String, Value>,
) -> AvroSchema {
    match schema {
        AvroSchema::Record {
            name,
            namespace,
            doc,
            fields,
            is_error,
            aliases,
            properties: mut existing,
        } => {
            existing.extend(properties);
            AvroSchema::Record {
                name,
                namespace,
                doc,
                fields,
                is_error,
                aliases,
                properties: existing,
            }
        }
        AvroSchema::Enum {
            name,
            namespace,
            doc,
            symbols,
            default,
            aliases,
            properties: mut existing,
        } => {
            existing.extend(properties);
            AvroSchema::Enum {
                name,
                namespace,
                doc,
                symbols,
                default,
                aliases,
                properties: existing,
            }
        }
        AvroSchema::Fixed {
            name,
            namespace,
            doc,
            size,
            aliases,
            properties: mut existing,
        } => {
            existing.extend(properties);
            AvroSchema::Fixed {
                name,
                namespace,
                doc,
                size,
                aliases,
                properties: existing,
            }
        }
        AvroSchema::Array {
            items,
            properties: mut existing,
        } => {
            existing.extend(properties);
            AvroSchema::Array {
                items,
                properties: existing,
            }
        }
        AvroSchema::Map {
            values,
            properties: mut existing,
        } => {
            existing.extend(properties);
            AvroSchema::Map {
                values,
                properties: existing,
            }
        }
        AvroSchema::Logical {
            logical_type,
            properties: mut existing,
        } => {
            existing.extend(properties);
            AvroSchema::Logical {
                logical_type,
                properties: existing,
            }
        }
        AvroSchema::AnnotatedPrimitive {
            kind,
            properties: mut existing,
        } => {
            existing.extend(properties);
            // A newly-added `logicalType` property may make this a recognized
            // logical type — promote if so.
            try_promote_logical_type(AvroSchema::AnnotatedPrimitive {
                kind,
                properties: existing,
            })
        }
        // Wrap bare primitives in AnnotatedPrimitive when properties are
        // present, then attempt logical type promotion in case the
        // properties include a recognized `logicalType` annotation.
        AvroSchema::Null => try_promote_logical_type(AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Null,
            properties,
        }),
        AvroSchema::Boolean => try_promote_logical_type(AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Boolean,
            properties,
        }),
        AvroSchema::Int => try_promote_logical_type(AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Int,
            properties,
        }),
        AvroSchema::Long => try_promote_logical_type(AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Long,
            properties,
        }),
        AvroSchema::Float => try_promote_logical_type(AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Float,
            properties,
        }),
        AvroSchema::Double => try_promote_logical_type(AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Double,
            properties,
        }),
        AvroSchema::Bytes => try_promote_logical_type(AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Bytes,
            properties,
        }),
        AvroSchema::String => try_promote_logical_type(AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::String,
            properties,
        }),
        AvroSchema::Reference {
            name,
            namespace,
            properties: mut existing,
            span,
        } => {
            existing.extend(properties);
            AvroSchema::Reference {
                name,
                namespace,
                properties: existing,
                span,
            }
        }
        // Union and other types that don't carry top-level properties.
        // TODO: warn when `properties` is non-empty here — annotations on
        // non-nullable unions are silently dropped (Java also rejects them).
        other => other,
    }
}

/// If the schema is an `AnnotatedPrimitive` whose properties contain a
/// `logicalType` key matching a recognized Avro logical type with a compatible
/// base primitive, promote it to `AvroSchema::Logical`. This mirrors Java's
/// `LogicalTypes.fromSchemaIgnoreInvalid()` call in `SchemaProperties.copyProperties()`.
///
/// Known logical types and their required base types:
/// - `date` -> int
/// - `time-millis` -> int
/// - `timestamp-millis` -> long
/// - `local-timestamp-millis` -> long
/// - `uuid` -> string
/// - `decimal` -> bytes (also requires `precision`; `scale` defaults to 0)
fn try_promote_logical_type(schema: AvroSchema) -> AvroSchema {
    let AvroSchema::AnnotatedPrimitive {
        kind,
        mut properties,
    } = schema
    else {
        return schema;
    };

    let Some(Value::String(logical_name)) = properties.get("logicalType").cloned() else {
        return AvroSchema::AnnotatedPrimitive { kind, properties };
    };

    match (logical_name.as_str(), &kind) {
        ("date", PrimitiveType::Int) => {
            properties.remove("logicalType");
            AvroSchema::Logical {
                logical_type: LogicalType::Date,
                properties,
            }
        }
        ("time-millis", PrimitiveType::Int) => {
            properties.remove("logicalType");
            AvroSchema::Logical {
                logical_type: LogicalType::TimeMillis,
                properties,
            }
        }
        ("timestamp-millis", PrimitiveType::Long) => {
            properties.remove("logicalType");
            AvroSchema::Logical {
                logical_type: LogicalType::TimestampMillis,
                properties,
            }
        }
        ("local-timestamp-millis", PrimitiveType::Long) => {
            properties.remove("logicalType");
            AvroSchema::Logical {
                logical_type: LogicalType::LocalTimestampMillis,
                properties,
            }
        }
        ("uuid", PrimitiveType::String) => {
            properties.remove("logicalType");
            AvroSchema::Logical {
                logical_type: LogicalType::Uuid,
                properties,
            }
        }
        ("decimal", PrimitiveType::Bytes) => {
            // `decimal` requires a `precision` property. If missing or not a
            // valid integer, the logical type is invalid and we leave the
            // schema as-is (matching Java's "ignore invalid" behavior).
            //
            // Java uses signed 32-bit `int` for precision/scale, so values
            // exceeding `i32::MAX` (2,147,483,647) are treated as invalid
            // even though they fit in `u32`. We filter accordingly.
            let Some(precision) = properties
                .get("precision")
                .and_then(json_value_as_u32)
                .filter(|&v| v <= i32::MAX as u32)
            else {
                return AvroSchema::AnnotatedPrimitive { kind, properties };
            };
            let scale = properties
                .get("scale")
                .and_then(json_value_as_u32)
                .filter(|&v| v <= i32::MAX as u32)
                .unwrap_or(0);

            properties.remove("logicalType");
            properties.remove("precision");
            properties.remove("scale");

            AvroSchema::Logical {
                logical_type: LogicalType::Decimal { precision, scale },
                properties,
            }
        }
        // Unrecognized logical type or mismatched base type: leave as-is.
        _ => AvroSchema::AnnotatedPrimitive { kind, properties },
    }
}

/// Try to interpret a `serde_json::Value` as a `u32`. Accepts both
/// integer and whole-number float representations, since JSON annotations
/// may arrive as either form.
fn json_value_as_u32(v: &Value) -> Option<u32> {
    match v {
        Value::Number(n) => {
            if let Some(i) = n.as_u64() {
                u32::try_from(i).ok()
            } else if let Some(f) = n.as_f64() {
                // Accept whole-number floats like 6.0.
                if f >= 0.0 && f <= u32::MAX as f64 && f.fract() == 0.0 {
                    Some(f as u32)
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Extract the doc comment for a parse tree context by looking at its start
/// token index. Uses the `extract_doc_comment` function from `doc_comments`
/// which scans backwards through hidden tokens.
///
/// Records the consumed doc comment's token index in `src.consumed_doc_indices`
/// so that orphaned doc comments can be detected after the full tree walk.
fn extract_doc_from_context<'input, T>(
    ctx: &T,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
) -> Option<String>
where
    T: antlr4rust::parser_rule_context::ParserRuleContext<'input>,
{
    let start = ctx.start();
    let token_index = start.get_token_index();
    extract_doc_comment(
        token_stream,
        token_index,
        Some(&mut src.consumed_doc_indices.borrow_mut()),
    )
}

/// Scan the entire token stream for `DocComment` tokens that were not consumed
/// by any declaration during the tree walk. Each orphaned doc comment generates
/// a warning matching Java's format.
///
/// This implements the same logic as Java's `IdlReader.getDocComment()`, which
/// checks for doc comment tokens between the previous call's position and the
/// current call's position. Our approach is equivalent: after the full walk, any
/// `DocComment` token not in the consumed set is orphaned.
fn collect_orphaned_doc_comment_warnings<'input, S>(
    token_stream: &S,
    consumed_indices: &HashSet<isize>,
    src: &SourceInfo<'_>,
) -> Vec<Warning>
where
    S: TokenStream<'input>,
{
    let mut warnings = Vec::new();
    let token_count = token_stream.size();

    for i in 0..token_count {
        let tok_wrapper = token_stream.get(i);
        let token: &<S::TF as TokenFactory<'input>>::Inner = tok_wrapper.borrow();
        let token_type = token.get_token_type();

        if token_type == Idl_DocComment && !consumed_indices.contains(&i) {
            warnings.push(Warning::out_of_place_doc_comment(
                token.get_line(),
                token.get_column(),
                src,
                token.get_start(),
                token.get_stop(),
            ));
        }
    }

    warnings
}

/// Parse a single import statement and append it as a `DeclItem::Import` to
/// the declaration items list.
fn collect_single_import<'input>(
    import_ctx: &ImportStatementContextAll<'input>,
    decl_items: &mut Vec<DeclItem>,
) {
    let kind_tok = import_ctx.importType.as_ref();
    let location_tok = import_ctx.location.as_ref();

    if let (Some(kind), Some(loc)) = (kind_tok, location_tok) {
        let import_kind = match kind.get_token_type() {
            Idl_IDL => ImportKind::Idl,
            Idl_Protocol => ImportKind::Protocol,
            Idl_Schema => ImportKind::Schema,
            _ => return,
        };

        decl_items.push(DeclItem::Import(ImportEntry {
            kind: import_kind,
            path: get_string_from_literal(loc.get_text()),
            span: span_from_context(import_ctx),
        }));
    }
}

// ==========================================================================
// Tests
// ==========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use miette::{GraphicalReportHandler, GraphicalTheme};
    use pretty_assertions::assert_eq;

    /// Render a `miette::Report` error to a deterministic string for snapshot
    /// testing. Uses `GraphicalTheme::none()` (no box-drawing characters) to
    /// match the `error_reporting.rs` test style.
    fn render_error(err: &miette::Report) -> String {
        let handler =
            GraphicalReportHandler::new_themed(GraphicalTheme::none()).with_width(80);
        let mut buf = String::new();
        handler
            .render_report(&mut buf, err.as_ref())
            .expect("render to String is infallible");
        buf
    }

    // ------------------------------------------------------------------
    // Octal escapes (issue #5)
    // ------------------------------------------------------------------

    #[test]
    fn octal_single_digit() {
        // \7 is octal 7 = BEL (U+0007).
        assert_eq!(unescape_java(r"\7"), "\u{0007}");
    }

    #[test]
    fn octal_two_digits() {
        // \77 is octal 77 = 63 = '?'.
        assert_eq!(unescape_java(r"\77"), "?");
    }

    #[test]
    fn octal_three_digits_newline() {
        // \012 is octal 012 = 10 = '\n'.
        assert_eq!(unescape_java(r"\012"), "\n");
    }

    #[test]
    fn octal_three_digits_uppercase_a() {
        // \101 is octal 101 = 65 = 'A'.
        assert_eq!(unescape_java(r"\101"), "A");
    }

    #[test]
    fn octal_three_digits_max() {
        // \377 is octal 377 = 255 = U+00FF (latin small letter y with diaeresis).
        assert_eq!(unescape_java(r"\377"), "\u{00FF}");
    }

    #[test]
    fn octal_high_first_digit_limits_to_two() {
        // \477 -- first digit is 4 (> 3), so only two digits are consumed:
        // \47 = octal 47 = 39 = '\'' and '7' is literal.
        assert_eq!(unescape_java(r"\477"), "'7");
    }

    #[test]
    fn octal_zero() {
        // \0 is the null character.
        assert_eq!(unescape_java(r"\0"), "\0");
    }

    // ------------------------------------------------------------------
    // Unicode escapes (multi-u support)
    // ------------------------------------------------------------------

    #[test]
    fn unicode_single_u() {
        assert_eq!(unescape_java(r"\u0041"), "A");
    }

    #[test]
    fn unicode_multi_u() {
        // \uu0041 and \uuu0041 should both produce 'A'.
        assert_eq!(unescape_java(r"\uu0041"), "A");
        assert_eq!(unescape_java(r"\uuu0041"), "A");
    }

    // ------------------------------------------------------------------
    // Unicode surrogate pairs (issue #4d730252)
    // ------------------------------------------------------------------

    #[test]
    fn surrogate_pair_grinning_face() {
        // \uD83D\uDE00 is the surrogate pair encoding of U+1F600
        // (GRINNING FACE). The high surrogate 0xD83D and low surrogate
        // 0xDE00 must be combined into a single code point.
        assert_eq!(unescape_java(r"\uD83D\uDE00"), "\u{1F600}");
    }

    #[test]
    fn surrogate_pair_with_multi_u() {
        // The low surrogate can also use the multi-u Java idiom.
        assert_eq!(unescape_java(r"\uD83D\uuDE00"), "\u{1F600}");
    }

    #[test]
    fn lone_high_surrogate_at_end() {
        // A high surrogate at the end of the string (no following \u)
        // cannot form a pair. Emit the raw escape unchanged.
        assert_eq!(unescape_java(r"\uD83D"), "\\uD83D");
    }

    #[test]
    fn high_surrogate_followed_by_non_surrogate_escape() {
        // \uD83D followed by \u0041 — the second escape is not a low
        // surrogate, so both are decoded independently. The high
        // surrogate falls through to the raw-escape path, then \u0041
        // produces 'A'.
        assert_eq!(unescape_java(r"\uD83D\u0041"), "\\uD83DA");
    }

    #[test]
    fn high_surrogate_followed_by_literal_text() {
        // A high surrogate followed by plain text, not a \u escape.
        assert_eq!(unescape_java(r"\uD83Dhello"), "\\uD83Dhello");
    }

    #[test]
    fn surrogate_pair_musical_symbol_g_clef() {
        // U+1D11E (MUSICAL SYMBOL G CLEF) = \uD834\uDD1E.
        assert_eq!(unescape_java(r"\uD834\uDD1E"), "\u{1D11E}");
    }

    // ------------------------------------------------------------------
    // Slash escape removal (issue #16)
    // ------------------------------------------------------------------

    #[test]
    fn slash_is_not_unescaped() {
        // \/ is not a valid escape in the grammar. The backslash should be
        // preserved as-is, producing the two-character sequence "\/".
        assert_eq!(unescape_java(r"\/"), "\\/");
    }

    // ------------------------------------------------------------------
    // Standard escapes (regression)
    // ------------------------------------------------------------------

    #[test]
    fn standard_escapes() {
        assert_eq!(unescape_java(r"\n"), "\n");
        assert_eq!(unescape_java(r"\r"), "\r");
        assert_eq!(unescape_java(r"\t"), "\t");
        assert_eq!(unescape_java(r"\b"), "\u{0008}");
        assert_eq!(unescape_java(r"\f"), "\u{000C}");
        assert_eq!(unescape_java(r"\\"), "\\");
        assert_eq!(unescape_java(r#"\""#), "\"");
        assert_eq!(unescape_java(r"\'"), "'");
    }

    #[test]
    fn mixed_escapes() {
        assert_eq!(unescape_java(r"hello\012world"), "hello\nworld");
        assert_eq!(unescape_java(r"\101\102\103"), "ABC");
    }

    // ------------------------------------------------------------------
    // One-way messages must return void (issue #877f0e96)
    // ------------------------------------------------------------------

    #[test]
    fn oneway_nonvoid_return_is_rejected() {
        let idl = r#"
            @namespace("test")
            protocol OneWayTest {
                record Msg { string text; }
                Msg send(Msg m) oneway;
            }
        "#;
        let err = parse_idl_for_test(idl).unwrap_err();
        insta::assert_snapshot!(render_error(&err));
    }

    #[test]
    fn oneway_void_return_is_accepted() {
        let idl = r#"
            @namespace("test")
            protocol OneWayTest {
                record Msg { string text; }
                void send(Msg m) oneway;
            }
        "#;
        let result = parse_idl_for_test(idl);
        assert!(result.is_ok(), "one-way void message should be accepted");
    }

    // ------------------------------------------------------------------
    // Annotations on type references must be rejected (issue #caeb40b1)
    // ------------------------------------------------------------------

    #[test]
    fn annotation_on_type_reference_is_rejected() {
        let idl = r#"
            @namespace("test")
            protocol P {
                fixed MD5(16);
                record R {
                    @foo("bar") MD5 hash = "0000000000000000";
                }
            }
        "#;
        let err = parse_idl_for_test(idl).unwrap_err();
        insta::assert_snapshot!(render_error(&err));
    }

    #[test]
    fn annotation_on_message_with_named_return_type_is_rejected() {
        // Annotations on a message whose return type is a named type reference
        // are rejected, matching Java's exitNullableType behavior. The grammar
        // places the annotations on the messageDeclaration, but Java considers
        // them ambiguous when the return type is a named reference.
        let idl = r#"
            @namespace("test")
            protocol P {
                record Foo { string name; }
                @prop("x") Foo getFoo(string id);
            }
        "#;
        let err = parse_idl_for_test(idl).unwrap_err();
        let msg = format!("{:?}", err);
        assert!(
            msg.contains("Type references may not be annotated"),
            "expected 'Type references may not be annotated' error, got: {msg}"
        );
    }

    #[test]
    fn annotation_on_message_with_void_return_is_accepted() {
        // Annotations on a void-returning message are fine -- the annotation
        // applies to the message, and there is no named type reference.
        let idl = r#"
            @namespace("test")
            protocol P {
                record Foo { string name; }
                @prop("x") void doThing(Foo input);
            }
        "#;
        let (idl_file, _, _) = parse_idl_for_test(idl).unwrap();
        let protocol = match idl_file {
            IdlFile::Protocol(p) => p,
            _ => panic!("expected protocol"),
        };
        let msg = protocol.messages.get("doThing").expect("doThing message");
        assert_eq!(msg.properties.get("prop"), Some(&serde_json::json!("x")));
    }

    #[test]
    fn annotation_on_primitive_type_is_accepted() {
        // Annotations on primitive types are fine -- only type references
        // (bare names referring to previously-defined types) are rejected.
        let idl = r#"
            @namespace("test")
            protocol P {
                record R {
                    @foo("bar") string name;
                }
            }
        "#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_ok(),
            "annotation on primitive type should be accepted"
        );
    }

    // ------------------------------------------------------------------
    // ANTLR parse errors must be fatal (issue #1b49abf1)
    // ------------------------------------------------------------------

    #[test]
    fn missing_semicolon_is_rejected() {
        // A missing semicolon after a field declaration is a syntax error.
        // ANTLR can recover and produce output, but we must detect the error
        // and fail. Java exits 1 with SchemaParseException.
        let idl = r#"
            @namespace("test")
            protocol P {
                record R { string name }
            }
        "#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_err(),
            "expected error for missing semicolon in record field"
        );
    }

    #[test]
    fn valid_protocol_still_accepted() {
        // Sanity check: valid IDL with correct syntax must still parse.
        let idl = r#"
            @namespace("test")
            protocol P {
                record R { string name; }
            }
        "#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_ok(),
            "valid protocol should be accepted, got: {:?}",
            result.err()
        );
    }

    // ------------------------------------------------------------------
    // SUB character (U+001A) as EOF marker (issue #c44fd7cc)
    // ------------------------------------------------------------------

    #[test]
    fn sub_character_treated_as_eof() {
        // The ANTLR grammar treats \u001a (ASCII SUB) as an EOF marker.
        // Content after SUB should be ignored, and the parse should succeed.
        let idl = "protocol P { record R { int x; } }\u{001a}trailing garbage";
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_ok(),
            "SUB character should act as EOF marker, got: {:?}",
            result.err()
        );
        let (idl_file, _, _) = result.unwrap();
        assert!(
            matches!(idl_file, IdlFile::Protocol(ref p) if p.name == "P"),
            "expected Protocol named 'P', got: {:?}",
            idl_file
        );
    }

    #[test]
    fn sub_character_at_end_without_trailing_content() {
        // A SUB character at the very end (no trailing content) should also
        // parse successfully.
        let idl = "protocol P { record R { int x; } }\u{001a}";
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_ok(),
            "SUB at end of input should be accepted, got: {:?}",
            result.err()
        );
    }

    // ------------------------------------------------------------------
    // Floating-point literal parsing (issue #d34a4c3b)
    // ------------------------------------------------------------------

    #[test]
    fn float_decimal_no_suffix() {
        let val = parse_float_text("3.14").expect("plain decimal");
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn float_suffix_f_lowercase() {
        // Java-style `f` suffix is stripped before parsing.
        let val = parse_float_text("3.14f").expect("f suffix");
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn float_suffix_f_uppercase() {
        let val = parse_float_text("3.14F").expect("F suffix");
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn float_suffix_d_lowercase() {
        let val = parse_float_text("3.14d").expect("d suffix");
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn float_suffix_d_uppercase() {
        let val = parse_float_text("3.14D").expect("D suffix");
        assert!((val - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn float_scientific_with_suffix() {
        // 1e5f = 100000.0 with float suffix stripped.
        let val = parse_float_text("1e5f").expect("scientific with f suffix");
        assert!((val - 1e5).abs() < f64::EPSILON);
    }

    #[test]
    fn float_scientific_negative_exponent_with_suffix() {
        let val = parse_float_text("1.5e-3D").expect("scientific neg exp with D suffix");
        assert!((val - 1.5e-3).abs() < f64::EPSILON);
    }

    #[test]
    fn float_negative_value_with_suffix() {
        let val = parse_float_text("-2.5f").expect("negative with f suffix");
        assert!((val - (-2.5)).abs() < f64::EPSILON);
    }

    #[test]
    fn float_positive_sign_with_suffix() {
        let val = parse_float_text("+2.5d").expect("positive sign with d suffix");
        assert!((val - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn float_nan() {
        assert!(parse_float_text("NaN").expect("NaN").is_nan());
    }

    #[test]
    fn float_infinity() {
        assert_eq!(
            parse_float_text("Infinity").expect("Infinity"),
            f64::INFINITY
        );
        assert_eq!(
            parse_float_text("-Infinity").expect("-Infinity"),
            f64::NEG_INFINITY
        );
    }

    // ------------------------------------------------------------------
    // Hex floating-point literals (issue #d34a4c3b)
    // ------------------------------------------------------------------

    #[test]
    fn hex_float_basic() {
        // 0x1.0p10 = 1.0 * 2^10 = 1024.0
        let val = parse_float_text("0x1.0p10").expect("hex float 0x1.0p10");
        assert!((val - 1024.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hex_float_no_fraction() {
        // 0xAp3 = 10 * 2^3 = 80.0
        let val = parse_float_text("0xAp3").expect("hex float 0xAp3");
        assert!((val - 80.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hex_float_fraction_only() {
        // 0x.8p1 = 0.5 * 2^1 = 1.0
        let val = parse_float_text("0x.8p1").expect("hex float 0x.8p1");
        assert!((val - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hex_float_trailing_dot() {
        // 0x1.p0 = 1.0 * 2^0 = 1.0
        let val = parse_float_text("0x1.p0").expect("hex float 0x1.p0");
        assert!((val - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hex_float_negative_exponent() {
        // 0x1.0p-3 = 1.0 * 2^-3 = 0.125
        let val = parse_float_text("0x1.0p-3").expect("hex float 0x1.0p-3");
        assert!((val - 0.125).abs() < f64::EPSILON);
    }

    #[test]
    fn hex_float_uppercase_x_and_p() {
        // 0X1.0P10 = 1024.0
        let val = parse_float_text("0X1.0P10").expect("hex float 0X1.0P10");
        assert!((val - 1024.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hex_float_with_suffix() {
        // 0x1.0p10f — hex float with Java float suffix stripped.
        let val = parse_float_text("0x1.0p10f").expect("hex float with f suffix");
        assert!((val - 1024.0).abs() < f64::EPSILON);
    }

    #[test]
    fn hex_float_negative_sign() {
        // -0x1.0p10 = -1024.0
        let val = parse_float_text("-0x1.0p10").expect("negative hex float");
        assert!((val - (-1024.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn hex_float_mixed_hex_digits() {
        // 0x1.8p1 = (1 + 8/16) * 2^1 = 1.5 * 2 = 3.0
        let val = parse_float_text("0x1.8p1").expect("hex float 0x1.8p1");
        assert!((val - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn float_suffix_only_no_dot() {
        // The grammar allows `Digit+ [fFdD]` (no decimal point, no exponent,
        // just a suffix to distinguish from IntegerLiteral).
        let val = parse_float_text("42f").expect("integer-like float with f suffix");
        assert!((val - 42.0).abs() < f64::EPSILON);
    }

    // ------------------------------------------------------------------
    // Reserved property name validation (issue #ee3a2bca)
    // ------------------------------------------------------------------

    #[test]
    fn doc_annotation_on_protocol_is_rejected() {
        // `@doc("...")` is a reserved property name on protocols. Java throws
        // "Can't set reserved property: doc". Doc comments should use `/** ... */`.
        let idl = r#"
            @namespace("test")
            @doc("Protocol doc via annotation")
            protocol P {
                record R { string name; }
            }
        "#;
        let err = parse_idl_for_test(idl).unwrap_err();
        insta::assert_snapshot!(render_error(&err));
    }

    #[test]
    fn doc_annotation_on_record_is_rejected() {
        // `@doc` is reserved on schemas (records, enums, fixed).
        let idl = r#"
            @namespace("test")
            protocol P {
                @doc("Record doc") record R { string name; }
            }
        "#;
        let err = parse_idl_for_test(idl).unwrap_err();
        insta::assert_snapshot!(render_error(&err));
    }

    #[test]
    fn type_annotation_on_field_type_is_rejected() {
        // `@type` is reserved on schemas. When used as a type annotation
        // (via fullType's BARE_PROPS), it should be rejected.
        let idl = r#"
            @namespace("test")
            protocol P {
                record R { @type("custom") string name; }
            }
        "#;
        let err = parse_idl_for_test(idl).unwrap_err();
        insta::assert_snapshot!(render_error(&err));
    }

    #[test]
    fn doc_annotation_on_field_variable_is_rejected() {
        // `@doc` is reserved on fields (FIELD_RESERVED).
        let idl = r#"
            @namespace("test")
            protocol P {
                record R { string @doc("field doc") name; }
            }
        "#;
        let err = parse_idl_for_test(idl).unwrap_err();
        insta::assert_snapshot!(render_error(&err));
    }

    #[test]
    fn default_annotation_on_enum_is_rejected() {
        // `@default` is reserved on enums (ENUM_RESERVED extends SCHEMA_RESERVED
        // with `default`).
        let idl = r#"
            @namespace("test")
            protocol P {
                @default("A") enum E { A, B, C }
            }
        "#;
        let err = parse_idl_for_test(idl).unwrap_err();
        insta::assert_snapshot!(render_error(&err));
    }

    #[test]
    fn default_annotation_on_record_is_not_reserved() {
        // `default` is NOT in SCHEMA_RESERVED (only in ENUM_RESERVED and
        // FIELD_RESERVED). On a record, it should be accepted as a custom property.
        let idl = r#"
            @namespace("test")
            protocol P {
                @default("x") record R { string name; }
            }
        "#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_ok(),
            "default annotation on record should be accepted, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn version_annotation_on_protocol_is_accepted() {
        // `@version` is NOT reserved in avro-tools 1.12.1 (even though the git
        // source may list it). The golden test file `simple.avdl` uses it.
        let idl = r#"
            @namespace("test")
            @version("1.0.5")
            protocol P {
                record R { string name; }
            }
        "#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_ok(),
            "version annotation on protocol should be accepted, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn doc_annotation_on_message_is_rejected() {
        // `@doc` is reserved on messages (MESSAGE_RESERVED).
        let idl = r#"
            @namespace("test")
            protocol P {
                @doc("message doc") void ping();
            }
        "#;
        let err = parse_idl_for_test(idl).unwrap_err();
        insta::assert_snapshot!(render_error(&err));
    }

    #[test]
    fn response_annotation_on_message_is_rejected() {
        // `@response` is reserved on messages.
        let idl = r#"
            @namespace("test")
            protocol P {
                @response("custom") void ping();
            }
        "#;
        let err = parse_idl_for_test(idl).unwrap_err();
        insta::assert_snapshot!(render_error(&err));
    }

    #[test]
    fn custom_annotation_on_protocol_is_accepted() {
        // Non-reserved names should still be accepted as custom properties.
        let idl = r#"
            @namespace("test")
            @myCustomProp("hello")
            protocol P {
                record R { string name; }
            }
        "#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_ok(),
            "custom annotation on protocol should be accepted, got: {:?}",
            result.err()
        );
    }

    // ------------------------------------------------------------------
    // Logical type promotion from @logicalType annotation (issue #ae25a66f)
    // ------------------------------------------------------------------

    /// Helper: parse an IDL protocol with a single record, return its first field's schema.
    fn parse_first_field_schema(idl: &str) -> AvroSchema {
        let (_idl_file, decl_items, _warnings) =
            parse_idl_for_test(idl).expect("IDL should parse successfully");
        // Find the record among declaration items.
        for item in &decl_items {
            if let DeclItem::Type(AvroSchema::Record { fields, .. }, _) = item {
                return fields[0].schema.clone();
            }
        }
        panic!("no record found in declaration items");
    }

    #[test]
    fn logicaltype_annotation_date_promoted() {
        // `@logicalType("date") int` should be promoted to `Logical { Date }`,
        // not left as `AnnotatedPrimitive { Int, {"logicalType": "date"} }`.
        let idl = r#"
            @namespace("test")
            protocol P {
                record R { @logicalType("date") int myDate; }
            }
        "#;
        let schema = parse_first_field_schema(idl);
        assert!(
            matches!(
                schema,
                AvroSchema::Logical {
                    logical_type: LogicalType::Date,
                    ..
                }
            ),
            "expected Logical(Date), got: {schema:?}"
        );
    }

    #[test]
    fn logicaltype_annotation_time_millis_promoted() {
        let idl = r#"
            @namespace("test")
            protocol P {
                record R { @logicalType("time-millis") int myTime; }
            }
        "#;
        let schema = parse_first_field_schema(idl);
        assert!(
            matches!(
                schema,
                AvroSchema::Logical {
                    logical_type: LogicalType::TimeMillis,
                    ..
                }
            ),
            "expected Logical(TimeMillis), got: {schema:?}"
        );
    }

    #[test]
    fn logicaltype_annotation_timestamp_millis_promoted() {
        let idl = r#"
            @namespace("test")
            protocol P {
                record R { @logicalType("timestamp-millis") long myTs; }
            }
        "#;
        let schema = parse_first_field_schema(idl);
        assert!(
            matches!(
                schema,
                AvroSchema::Logical {
                    logical_type: LogicalType::TimestampMillis,
                    ..
                }
            ),
            "expected Logical(TimestampMillis), got: {schema:?}"
        );
    }

    #[test]
    fn logicaltype_annotation_local_timestamp_millis_promoted() {
        let idl = r#"
            @namespace("test")
            protocol P {
                record R { @logicalType("local-timestamp-millis") long myLts; }
            }
        "#;
        let schema = parse_first_field_schema(idl);
        assert!(
            matches!(
                schema,
                AvroSchema::Logical {
                    logical_type: LogicalType::LocalTimestampMillis,
                    ..
                }
            ),
            "expected Logical(LocalTimestampMillis), got: {schema:?}"
        );
    }

    #[test]
    fn logicaltype_annotation_uuid_promoted() {
        let idl = r#"
            @namespace("test")
            protocol P {
                record R { @logicalType("uuid") string myUuid; }
            }
        "#;
        let schema = parse_first_field_schema(idl);
        assert!(
            matches!(
                schema,
                AvroSchema::Logical {
                    logical_type: LogicalType::Uuid,
                    ..
                }
            ),
            "expected Logical(Uuid), got: {schema:?}"
        );
    }

    #[test]
    fn logicaltype_annotation_decimal_promoted() {
        // `decimal` requires `precision` and optionally `scale`. When both
        // are provided via annotations, the schema should be promoted.
        let idl = r#"
            @namespace("test")
            protocol P {
                record R {
                    @logicalType("decimal") @precision(10) @scale(2) bytes myDec;
                }
            }
        "#;
        let schema = parse_first_field_schema(idl);
        match &schema {
            AvroSchema::Logical {
                logical_type: LogicalType::Decimal { precision, scale },
                ..
            } => {
                assert_eq!(*precision, 10, "expected precision 10");
                assert_eq!(*scale, 2, "expected scale 2");
            }
            other => panic!("expected Logical(Decimal), got: {other:?}"),
        }
    }

    #[test]
    fn logicaltype_annotation_decimal_default_scale() {
        // When `@scale` is omitted, decimal should default to scale 0.
        let idl = r#"
            @namespace("test")
            protocol P {
                record R {
                    @logicalType("decimal") @precision(5) bytes myDec;
                }
            }
        "#;
        let schema = parse_first_field_schema(idl);
        match &schema {
            AvroSchema::Logical {
                logical_type: LogicalType::Decimal { precision, scale },
                ..
            } => {
                assert_eq!(*precision, 5, "expected precision 5");
                assert_eq!(*scale, 0, "expected scale 0 (default)");
            }
            other => panic!("expected Logical(Decimal) with default scale, got: {other:?}"),
        }
    }

    #[test]
    fn logicaltype_annotation_decimal_missing_precision_not_promoted() {
        // Without `@precision`, `decimal` is invalid and should remain as
        // an AnnotatedPrimitive (matching Java's "ignore invalid" behavior).
        let idl = r#"
            @namespace("test")
            protocol P {
                record R {
                    @logicalType("decimal") bytes myDec;
                }
            }
        "#;
        let schema = parse_first_field_schema(idl);
        assert!(
            matches!(schema, AvroSchema::AnnotatedPrimitive { .. }),
            "expected AnnotatedPrimitive (invalid decimal without precision), got: {schema:?}"
        );
    }

    #[test]
    fn logicaltype_annotation_wrong_base_type_not_promoted() {
        // `@logicalType("date")` on `long` (instead of `int`) should not
        // be promoted, since `date` requires `int` as the base type.
        let idl = r#"
            @namespace("test")
            protocol P {
                record R { @logicalType("date") long wrongBase; }
            }
        "#;
        let schema = parse_first_field_schema(idl);
        assert!(
            matches!(schema, AvroSchema::AnnotatedPrimitive { .. }),
            "expected AnnotatedPrimitive (date on wrong base type), got: {schema:?}"
        );
    }

    #[test]
    fn logicaltype_annotation_unknown_type_not_promoted() {
        // An unrecognized `logicalType` value should remain as AnnotatedPrimitive.
        let idl = r#"
            @namespace("test")
            protocol P {
                record R { @logicalType("custom-type") int myField; }
            }
        "#;
        let schema = parse_first_field_schema(idl);
        assert!(
            matches!(schema, AvroSchema::AnnotatedPrimitive { .. }),
            "expected AnnotatedPrimitive (unknown logicalType), got: {schema:?}"
        );
    }

    #[test]
    fn logicaltype_annotation_preserves_extra_properties() {
        // Extra custom properties alongside `@logicalType` should be preserved
        // on the promoted Logical schema.
        let idl = r#"
            @namespace("test")
            protocol P {
                record R { @logicalType("date") @custom("extra") int myDate; }
            }
        "#;
        let schema = parse_first_field_schema(idl);
        match &schema {
            AvroSchema::Logical {
                logical_type: LogicalType::Date,
                properties,
            } => {
                assert_eq!(
                    properties.get("custom"),
                    Some(&Value::String("extra".to_string())),
                    "custom property should be preserved after promotion"
                );
                assert!(
                    !properties.contains_key("logicalType"),
                    "logicalType key should be removed from properties after promotion"
                );
            }
            other => panic!("expected Logical(Date) with extra properties, got: {other:?}"),
        }
    }

    // ------------------------------------------------------------------
    // Duplicate types in union (issue #1c65fa55)
    // ------------------------------------------------------------------

    #[test]
    fn duplicate_null_in_union_is_rejected() {
        let idl = r#"
            protocol Test {
                record Foo {
                    union { null, string, null } field1;
                }
            }
        "#;
        let err = parse_idl_for_test(idl).unwrap_err();
        insta::assert_snapshot!(render_error(&err));
    }

    #[test]
    fn duplicate_string_in_union_is_rejected() {
        let idl = r#"
            protocol Test {
                record Foo {
                    union { string, int, string } field1;
                }
            }
        "#;
        let err = parse_idl_for_test(idl).unwrap_err();
        insta::assert_snapshot!(render_error(&err));
    }

    #[test]
    fn duplicate_named_type_in_union_is_rejected() {
        let idl = r#"
            protocol Test {
                record Bar { string name; }
                record Foo {
                    union { null, Bar, Bar } field1;
                }
            }
        "#;
        let err = parse_idl_for_test(idl).unwrap_err();
        insta::assert_snapshot!(render_error(&err));
    }

    #[test]
    fn valid_union_no_duplicates_is_accepted() {
        let idl = r#"
            protocol Test {
                record Foo {
                    union { null, string, int, long } field1;
                }
            }
        "#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_ok(),
            "union without duplicates should be accepted, got: {:?}",
            result.err()
        );
    }

    #[test]
    fn different_named_types_in_union_is_accepted() {
        let idl = r#"
            protocol Test {
                record Bar { string name; }
                record Baz { int value; }
                record Foo {
                    union { null, Bar, Baz } field1;
                }
            }
        "#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_ok(),
            "union with different named types should be accepted, got: {:?}",
            result.err()
        );
    }

    // ------------------------------------------------------------------
    // Enum default symbol validation (issue #1498f786)
    // ------------------------------------------------------------------

    #[test]
    fn enum_default_not_in_symbols_is_rejected() {
        let idl = r#"
            protocol P {
                enum E { A, B, C } = NONEXISTENT;
            }
        "#;
        let err = parse_idl_for_test(idl).unwrap_err();
        insta::assert_snapshot!(render_error(&err));
    }

    #[test]
    fn enum_default_in_symbols_is_accepted() {
        let idl = r#"
            protocol P {
                enum E { A, B, C } = B;
            }
        "#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_ok(),
            "valid enum default should be accepted, got: {:?}",
            result.err()
        );
    }

    // ------------------------------------------------------------------
    // Protocol name validation (issue #c5e9c318)
    // ------------------------------------------------------------------

    #[test]
    fn protocol_name_null_is_rejected() {
        let idl = "protocol `null` { }";
        let err = parse_idl_for_test(idl).unwrap_err();
        insta::assert_snapshot!(render_error(&err));
    }

    #[test]
    fn protocol_name_int_is_rejected() {
        let idl = "protocol `int` { }";
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_err(),
            "reserved protocol name 'int' should be rejected"
        );
    }

    // ------------------------------------------------------------------
    // Duplicate @namespace — last-write-wins (issue #7dc5ec17)
    // ------------------------------------------------------------------

    #[test]
    fn duplicate_namespace_uses_last_value() {
        let idl = r#"
            @namespace("test.edge")
            protocol P {
                @namespace("ns1")
                @namespace("ns2")
                record DualNs { string name; }
            }
        "#;
        let (_idl_file, decl_items, _warnings) =
            parse_idl_for_test(idl).expect("duplicate @namespace should be accepted");
        let record = decl_items
            .iter()
            .find_map(|item| {
                if let DeclItem::Type(schema @ AvroSchema::Record { .. }, _) = item {
                    Some(schema)
                } else {
                    None
                }
            })
            .expect("should contain a record");
        match record {
            AvroSchema::Record { namespace, .. } => {
                assert_eq!(
                    namespace.as_deref(),
                    Some("ns2"),
                    "last @namespace should win"
                );
            }
            _ => unreachable!(),
        }
    }

    // ------------------------------------------------------------------
    // Alias name validation (issue #24f3d986)
    // ------------------------------------------------------------------

    #[test]
    fn alias_with_leading_digit_is_rejected() {
        let idl = r#"
            protocol P {
                @aliases(["123bad"])
                record Foo { string name; }
            }
        "#;
        let err = parse_idl_for_test(idl).unwrap_err();
        insta::assert_snapshot!(render_error(&err));
    }

    #[test]
    fn alias_with_dash_is_rejected() {
        let idl = r#"
            protocol P {
                @aliases(["my-alias"])
                record Foo { string name; }
            }
        "#;
        let result = parse_idl_for_test(idl);
        assert!(result.is_err(), "dashed alias name should be rejected");
    }

    #[test]
    fn valid_qualified_alias_is_accepted() {
        let idl = r#"
            protocol P {
                @aliases(["org.example.OldFoo"])
                record Foo { string name; }
            }
        "#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_ok(),
            "qualified alias should be accepted, got: {:?}",
            result.err()
        );
    }

    // ------------------------------------------------------------------
    // Decimal precision overflow (issue #b638adba)
    // ------------------------------------------------------------------

    #[test]
    fn decimal_precision_overflow_is_not_promoted() {
        // 3000000000 exceeds i32::MAX. Java does not promote this to a
        // logical type — the schema should remain an AnnotatedPrimitive.
        let idl = r#"
            protocol P {
                record R {
                    @logicalType("decimal") @precision(3000000000) @scale(0) bytes field1;
                }
            }
        "#;
        let schema = parse_first_field_schema(idl);
        assert!(
            matches!(schema, AvroSchema::AnnotatedPrimitive { .. }),
            "decimal with precision > i32::MAX should remain AnnotatedPrimitive, \
             got: {schema:?}"
        );
    }

    // ------------------------------------------------------------------
    // Default value type validation (issue #01ee3f73)
    // ------------------------------------------------------------------

    #[test]
    fn default_int_string_is_rejected() {
        let idl = r#"protocol P { record R { int count = "hello"; } }"#;
        let err = parse_idl_for_test(idl).unwrap_err();
        insta::assert_snapshot!(render_error(&err));
    }

    #[test]
    fn default_boolean_int_is_rejected() {
        let idl = r#"protocol P { record R { boolean flag = 42; } }"#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_err(),
            "boolean with int default should be rejected"
        );
    }

    #[test]
    fn default_string_array_is_rejected() {
        let idl = r#"protocol P { record R { string name = [1, 2, 3]; } }"#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_err(),
            "string with array default should be rejected"
        );
    }

    #[test]
    fn default_int_null_is_rejected() {
        let idl = r#"protocol P { record R { int count = null; } }"#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_err(),
            "non-nullable int with null default should be rejected"
        );
    }

    #[test]
    fn default_int_float_is_rejected() {
        let idl = r#"protocol P { record R { int count = 3.14; } }"#;
        let result = parse_idl_for_test(idl);
        assert!(result.is_err(), "int with float default should be rejected");
    }

    #[test]
    fn default_int_object_is_rejected() {
        let idl = r#"protocol P { record R { int count = {"key": "value"}; } }"#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_err(),
            "int with object default should be rejected"
        );
    }

    #[test]
    fn default_int_overflow_is_rejected() {
        let idl = r#"protocol P { record R { int count = 9999999999; } }"#;
        let err = parse_idl_for_test(idl).expect_err("int with out-of-range default should be rejected");
        let rendered = render_error(&err);
        assert!(
            rendered.contains("out of range"),
            "error should mention 'out of range', got: {rendered}"
        );
    }

    #[test]
    fn default_int_negative_overflow_is_rejected() {
        let idl = r#"protocol P { record R { int count = -2147483649; } }"#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_err(),
            "int with below-range default should be rejected"
        );
    }

    #[test]
    fn default_int_max_boundary_is_accepted() {
        let idl = r#"protocol P { record R { int count = 2147483647; } }"#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "int with i32::MAX default should be accepted"
        );
    }

    #[test]
    fn default_int_min_boundary_is_accepted() {
        let idl = r#"protocol P { record R { int count = -2147483648; } }"#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "int with i32::MIN default should be accepted"
        );
    }

    #[test]
    fn default_long_accepts_value_above_i32_max() {
        let idl = r#"protocol P { record R { long count = 9999999999; } }"#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "long with value above i32::MAX should be accepted"
        );
    }

    #[test]
    fn default_bytes_int_is_rejected() {
        let idl = r#"protocol P { record R { bytes data = 42; } }"#;
        let result = parse_idl_for_test(idl);
        assert!(result.is_err(), "bytes with int default should be rejected");
    }

    #[test]
    fn default_string_int_is_rejected() {
        let idl = r#"protocol P { record R { string name = 42; } }"#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_err(),
            "string with int default should be rejected"
        );
    }

    // Valid defaults that should still be accepted:

    #[test]
    fn default_int_valid() {
        let idl = r#"protocol P { record R { int count = 42; } }"#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "int with int default should be accepted"
        );
    }

    #[test]
    fn default_string_valid() {
        let idl = r#"protocol P { record R { string name = "hello"; } }"#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "string with string default should be accepted"
        );
    }

    #[test]
    fn default_boolean_valid() {
        let idl = r#"protocol P { record R { boolean flag = true; } }"#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "boolean with boolean default should be accepted"
        );
    }

    #[test]
    fn default_double_valid() {
        let idl = r#"protocol P { record R { double value = 3.14; } }"#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "double with float default should be accepted"
        );
    }

    #[test]
    fn default_double_nan_valid() {
        let idl = r#"protocol P { record R { double value = NaN; } }"#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "double with NaN default should be accepted"
        );
    }

    #[test]
    fn default_float_infinity_valid() {
        let idl = r#"protocol P { record R { float value = -Infinity; } }"#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "float with -Infinity default should be accepted"
        );
    }

    #[test]
    fn default_nullable_null_valid() {
        let idl = r#"protocol P { record R { string? name = null; } }"#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "nullable with null default should be accepted"
        );
    }

    #[test]
    fn default_nullable_non_null_valid() {
        let idl = r#"protocol P { record R { string? name = "hello"; } }"#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "nullable with non-null default should be accepted"
        );
    }

    #[test]
    fn default_array_empty_valid() {
        let idl = r#"protocol P { record R { array<int> nums = []; } }"#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "array with empty array default should be accepted"
        );
    }

    #[test]
    fn default_map_empty_valid() {
        let idl = r#"protocol P { record R { map<string> m = {}; } }"#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "map with empty object default should be accepted"
        );
    }

    #[test]
    fn default_enum_string_valid() {
        let idl = r#"
            protocol P {
                enum Color { RED, GREEN, BLUE }
                record R { Color c = "RED"; }
            }
        "#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "enum with string default should be accepted"
        );
    }

    #[test]
    fn default_record_object_valid() {
        let idl = r#"
            protocol P {
                record Inner { string name; }
                record Outer { Inner inner = {"name": "test"}; }
            }
        "#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "record with object default should be accepted"
        );
    }

    #[test]
    fn default_union_null_first_valid() {
        let idl = r#"
            protocol P {
                record R { union { null, string } field = null; }
            }
        "#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "union with null first and null default should be accepted"
        );
    }

    #[test]
    fn default_union_non_first_branch_valid() {
        // Java validates union defaults against any branch, not just the first.
        let idl = r#"
            protocol P {
                record R {
                    union { null, string } x = "hello";
                    union { null, int } y = 42;
                }
            }
        "#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "union default matching non-first branch should be accepted"
        );
    }

    #[test]
    fn default_logical_date_int_valid() {
        let idl = r#"protocol P { record R { date d = 0; } }"#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "date with int default should be accepted"
        );
    }

    #[test]
    fn default_annotated_long_int_valid() {
        let idl = r#"protocol P { record R { @foo.bar("baz") long l = 0; } }"#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "annotated long with int default should be accepted"
        );
    }

    #[test]
    fn default_message_param_validated() {
        // Message parameters also go through walk_variable, so validation applies.
        let idl = r#"protocol P { int add(int arg1, int arg2 = "bad"); }"#;
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_err(),
            "message param with invalid default should be rejected"
        );
    }

    #[test]
    fn default_forward_reference_skips_validation() {
        // Forward references cannot be validated because the type is not yet resolved.
        // This should not error even though the default might not match.
        let idl = r#"
            protocol P {
                record R { SomeEnum e = "VALUE"; }
                enum SomeEnum { VALUE }
            }
        "#;
        assert!(
            parse_idl_for_test(idl).is_ok(),
            "forward reference with default should skip validation"
        );
    }

    // ------------------------------------------------------------------
    // ANTLR error message enrichment
    // ------------------------------------------------------------------

    #[test]
    fn enrich_no_viable_alternative_with_annotation() {
        // ANTLR merges `@beta` and `record` into `@betarecord`.
        let msg = "no viable alternative at input '@betarecord'";
        let enriched = enrich_antlr_error(msg).expect("should match").message;
        assert!(
            enriched.contains("@beta"),
            "should extract annotation name: {enriched}"
        );
        assert!(
            enriched.contains("@beta(\"value\")"),
            "should suggest correct syntax: {enriched}"
        );
    }

    #[test]
    fn enrich_no_viable_alternative_with_preceding_valid_annotation() {
        // When a valid `@namespace(...)` precedes a bare `@version`, ANTLR
        // merges everything: `@namespace("com.example")@versionprotocol`.
        let msg = "no viable alternative at input '@namespace(\"com.example\")@versionprotocol'";
        let enriched = enrich_antlr_error(msg).expect("should match").message;
        assert!(
            enriched.contains("@version"),
            "should identify the bare annotation: {enriched}"
        );
        // Should NOT include the trailing keyword in the annotation name.
        assert!(
            !enriched.contains("@versionprotocol\""),
            "should strip trailing keyword: {enriched}"
        );
    }

    #[test]
    fn enrich_mismatched_input_expecting_lparen() {
        let msg = "mismatched input 'string' expecting '('";
        let enriched = enrich_antlr_error(msg).expect("should match").message;
        assert!(
            enriched.contains("@name(value)"),
            "should explain annotation syntax: {enriched}"
        );
        // Should preserve the original message for context.
        assert!(
            enriched.contains("mismatched input"),
            "should include original message: {enriched}"
        );
    }

    #[test]
    fn enrich_returns_none_for_small_expecting_set() {
        // Errors with a small expected-token set should pass through unchanged.
        let msg = "mismatched input '}' expecting {';', ','}";
        assert!(
            enrich_antlr_error(msg).is_none(),
            "should not enrich errors with small token sets"
        );
    }

    #[test]
    fn enrich_large_extraneous_eof() {
        // When <EOF> is the extraneous token and the set is large, we should
        // produce a concise "unexpected end of file" message.
        let msg = "extraneous input '<EOF>' expecting {DocComment, 'protocol', \
                   'namespace', 'import', 'idl', 'schema', 'enum', 'fixed', \
                   'error', 'record', 'array', 'map'}";
        let enriched = enrich_antlr_error(msg).expect("should match large set");
        assert_eq!(enriched.message, "unexpected end of file");
        assert_eq!(
            enriched.label.as_deref(),
            Some("unexpected end of file"),
        );
    }

    #[test]
    fn enrich_large_extraneous_token() {
        // When a specific token is extraneous and the set is large, we should
        // produce "unexpected token `<tok>`".
        let msg = "extraneous input '123' expecting {DocComment, 'protocol', \
                   'namespace', 'import', 'idl', 'schema', 'enum', 'fixed', \
                   'error', 'record', 'array', 'map'}";
        let enriched = enrich_antlr_error(msg).expect("should match large set");
        assert_eq!(enriched.message, "unexpected token `123`");
        assert_eq!(enriched.label.as_deref(), Some("unexpected `123`"));
    }

    #[test]
    fn enrich_large_mismatched_token() {
        // Mismatched input with a large expected-token set.
        let msg = "mismatched input 'protocl' expecting {<EOF>, '\\u001A', \
                   DocComment, 'protocol', 'namespace', 'import', 'schema', \
                   'enum', 'fixed', 'error', 'record', '@'}";
        let enriched = enrich_antlr_error(msg).expect("should match large set");
        assert_eq!(enriched.message, "unexpected token `protocl`");
        assert_eq!(enriched.label.as_deref(), Some("unexpected `protocl`"));
    }

    #[test]
    fn enrich_large_mismatched_eof() {
        let msg = "mismatched input '<EOF>' expecting {'protocol', 'namespace', \
                   'import', 'idl', 'schema', 'enum', 'fixed', 'error', \
                   'record', 'array', 'map', 'union'}";
        let enriched = enrich_antlr_error(msg).expect("should match large set");
        assert_eq!(enriched.message, "unexpected end of file");
    }

    #[test]
    fn extract_annotation_name_simple() {
        assert_eq!(extract_annotation_name("@betarecord"), Some("beta"),);
    }

    #[test]
    fn extract_annotation_name_with_enum_keyword() {
        assert_eq!(extract_annotation_name("@unstableenum"), Some("unstable"),);
    }

    #[test]
    fn extract_annotation_name_no_keyword_suffix() {
        // When no known keyword is found at the end, the full ident is
        // returned. This is the best we can do without source access.
        assert_eq!(extract_annotation_name("@foobar"), Some("foobar"),);
    }

    #[test]
    fn extract_annotation_name_skips_valid_annotations() {
        // `@namespace(...)` has a `(` so it's valid; the second `@version`
        // without `(` is the problematic one.
        assert_eq!(
            extract_annotation_name("@namespace(\"x\")@versionprotocol"),
            Some("version"),
        );
    }

    #[test]
    fn split_trailing_keyword_strips_record() {
        assert_eq!(split_trailing_keyword("betarecord"), "beta");
    }

    #[test]
    fn split_trailing_keyword_strips_protocol() {
        assert_eq!(split_trailing_keyword("versionprotocol"), "version");
    }

    #[test]
    fn split_trailing_keyword_strips_string() {
        assert_eq!(split_trailing_keyword("deprecatedstring"), "deprecated");
    }

    #[test]
    fn split_trailing_keyword_no_match() {
        assert_eq!(split_trailing_keyword("foobar"), "foobar");
    }

    #[test]
    fn split_trailing_keyword_exact_keyword() {
        // If the entire merged text IS a keyword, don't strip it
        // (that would leave an empty string).
        assert_eq!(split_trailing_keyword("record"), "record");
    }

    // ------------------------------------------------------------------
    // Bare identifier quoting hint
    // ------------------------------------------------------------------

    #[test]
    fn looks_like_bare_identifier_uppercase() {
        assert!(looks_like_bare_identifier("YELLOW"));
    }

    #[test]
    fn looks_like_bare_identifier_mixed_case() {
        assert!(looks_like_bare_identifier("myValue"));
    }

    #[test]
    fn looks_like_bare_identifier_with_underscores() {
        assert!(looks_like_bare_identifier("MY_VALUE_2"));
    }

    #[test]
    fn looks_like_bare_identifier_rejects_null() {
        // JSON keywords should not trigger the quoting hint.
        assert!(!looks_like_bare_identifier("null"));
    }

    #[test]
    fn looks_like_bare_identifier_rejects_true() {
        assert!(!looks_like_bare_identifier("true"));
    }

    #[test]
    fn looks_like_bare_identifier_rejects_false() {
        assert!(!looks_like_bare_identifier("false"));
    }

    #[test]
    fn looks_like_bare_identifier_rejects_number() {
        assert!(!looks_like_bare_identifier("123"));
    }

    #[test]
    fn looks_like_bare_identifier_rejects_special_chars() {
        assert!(!looks_like_bare_identifier("<EOF>"));
    }

    #[test]
    fn looks_like_bare_identifier_rejects_empty() {
        assert!(!looks_like_bare_identifier(""));
    }

    #[test]
    fn expecting_set_includes_string_literal_present() {
        let tokens = "'null', 'true', 'false', '{', '[', StringLiteral, \
                      IntegerLiteral, FloatingPointLiteral";
        assert!(expecting_set_includes_string_literal(tokens));
    }

    #[test]
    fn expecting_set_includes_string_literal_absent() {
        let tokens = "'null', 'true', 'false', '{', '['";
        assert!(!expecting_set_includes_string_literal(tokens));
    }

    #[test]
    fn enrich_bare_identifier_in_json_value_position() {
        // Simulates the ANTLR error for `Color primary = YELLOW;`.
        let msg = "mismatched input 'YELLOW' expecting {'null', 'true', 'false', \
                   '{', '[', StringLiteral, IntegerLiteral, FloatingPointLiteral}";
        let enriched = enrich_antlr_error(msg).expect("should match");
        assert!(
            enriched.message.contains("\"YELLOW\""),
            "message should suggest quoting: {}",
            enriched.message,
        );
        assert!(
            enriched.label.as_deref().expect("should have label").contains("\"YELLOW\""),
            "label should suggest quoting: {:?}",
            enriched.label,
        );
        let help = enriched.help.as_deref().expect("should have help");
        assert!(
            help.contains("did you mean \"YELLOW\""),
            "help should suggest quoting: {help}",
        );
        assert!(
            help.contains("quoted strings"),
            "help should mention quoted strings: {help}",
        );
    }

    #[test]
    fn enrich_non_identifier_in_json_value_position_no_quoting_hint() {
        // A numeric token in a jsonValue position should NOT trigger the
        // quoting hint, since numbers are valid JSON values.
        let msg = "mismatched input ';' expecting {'null', 'true', 'false', \
                   '{', '[', StringLiteral, IntegerLiteral, FloatingPointLiteral}";
        let enriched = enrich_antlr_error(msg).expect("should match");
        // `;` is not a bare identifier, so no quoting hint should appear.
        let help = enriched.help.as_deref().unwrap_or("");
        assert!(
            !help.contains("did you mean"),
            "should not suggest quoting for non-identifiers: {help}",
        );
    }

    // ------------------------------------------------------------------
    // Integration: enriched error messages from parse_idl_for_test
    // ------------------------------------------------------------------

    #[test]
    fn parse_error_bare_enum_default_suggests_quoting() {
        let idl = r#"protocol Test {
            enum Color { RED, GREEN, BLUE }
            record Palette { Color primary = YELLOW; }
        }"#;
        let err = parse_idl_for_test(idl).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("YELLOW"),
            "error should mention the bare identifier: {msg}"
        );
        assert!(
            msg.contains("\"YELLOW\""),
            "error should suggest quoting: {msg}"
        );
    }

    #[test]
    fn parse_error_bare_annotation_before_protocol() {
        let idl = "@beta\nprotocol Test { record Foo { string name; } }";
        let err = parse_idl_for_test(idl).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("@beta"),
            "error should mention the annotation: {msg}"
        );
        assert!(
            msg.contains("@beta(\"value\")"),
            "error should suggest correct syntax: {msg}"
        );
    }

    #[test]
    fn nullable_null_rejected() {
        // `null?` would produce the invalid union `[null, null]`.
        // Java also rejects this input.
        let idl = "protocol Test { record Foo { null? value; } }";
        let result = parse_idl_for_test(idl);
        assert!(result.is_err(), "null? should be rejected");
    }

    #[test]
    fn nullable_null_in_array_rejected() {
        // `array<null?>` has the same problem in the element type.
        let idl = "protocol Test { record Foo { array<null?> values; } }";
        let result = parse_idl_for_test(idl);
        assert!(result.is_err(), "array<null?> should be rejected");
    }

    #[test]
    fn plain_null_type_accepted() {
        // Bare `null` (without `?`) is a valid field type.
        let idl = "protocol Test { record Foo { null value = null; } }";
        let result = parse_idl_for_test(idl);
        assert!(result.is_ok(), "plain null should be accepted: {result:?}");
    }

    #[test]
    fn decimal_zero_precision_rejected() {
        let idl = "protocol Test { record Foo { decimal(0) value; } }";
        let result = parse_idl_for_test(idl);
        assert!(result.is_err(), "decimal(0) should be rejected");
    }

    #[test]
    fn decimal_scale_exceeds_precision_rejected() {
        let idl = "protocol Test { record Foo { decimal(5, 10) value; } }";
        let result = parse_idl_for_test(idl);
        assert!(result.is_err(), "decimal(5, 10) should be rejected");
    }

    #[test]
    fn decimal_valid_precision_and_scale_accepted() {
        let idl = "protocol Test { record Foo { decimal(10, 2) value; } }";
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_ok(),
            "decimal(10, 2) should be accepted: {result:?}"
        );
    }

    #[test]
    fn decimal_scale_equals_precision_accepted() {
        // Edge case: scale == precision is valid per the Avro spec.
        let idl = "protocol Test { record Foo { decimal(5, 5) value; } }";
        let result = parse_idl_for_test(idl);
        assert!(
            result.is_ok(),
            "decimal(5, 5) should be accepted: {result:?}"
        );
    }

    /// Render a list of warnings to a deterministic string for snapshot tests.
    fn render_warnings(warnings: &[Warning]) -> String {
        use std::fmt::Write;
        let handler =
            miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode_nocolor())
                .with_width(80);
        let mut buf = String::new();
        for (i, w) in warnings.iter().enumerate() {
            if i > 0 {
                writeln!(buf).expect("write to String is infallible");
            }
            handler
                .render_report(&mut buf, w as &dyn miette::Diagnostic)
                .expect("render to String is infallible");
        }
        buf
    }

    #[test]
    fn lexer_error_produces_warning() {
        // A control character that the ANTLR lexer can't tokenize should
        // produce a warning (matching Java's behavior of printing to stderr),
        // not silently succeed or fatally fail.
        let idl = "protocol Test { record Foo { string\x01 name; } }";
        let (_, _, warnings) = parse_idl_for_test(idl).expect("lexer errors should not be fatal");
        assert_eq!(warnings.len(), 1);
        insta::assert_snapshot!(render_warnings(&warnings));
    }

    #[test]
    fn parse_error_annotation_missing_parens_on_field() {
        let idl = "protocol Test { record Foo { @deprecated string name; } }";
        let err = parse_idl_for_test(idl).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("@name(value)"),
            "error should explain annotation syntax: {msg}"
        );
    }
}
