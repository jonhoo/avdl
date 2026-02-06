use std::borrow::Borrow;

use antlr4rust::char_stream::InputData;
use antlr4rust::token::Token;
use antlr4rust::token_factory::TokenFactory;
use antlr4rust::token_stream::TokenStream;

use crate::generated::idlparser::{Idl_DocComment, Idl_EmptyComment, Idl_WS};

/// Extract the doc comment associated with a parse tree node, given the
/// token index of the node's start token.
///
/// Scans backwards from `token_index - 1` through the token stream,
/// skipping whitespace and empty comments, looking for a `DocComment` token.
///
/// antlr4rust's `CommonTokenStream` does not expose `getHiddenTokensToLeft()`
/// the way Java ANTLR does, but `get(index)` is public and lets us access
/// any token by index, including hidden-channel tokens.
pub fn extract_doc_comment<'input, TS>(
    token_stream: &TS,
    token_index: isize,
) -> Option<String>
where
    TS: TokenStream<'input>,
{
    if token_index <= 0 {
        return None;
    }

    let mut i = token_index - 1;
    let mut doc_token_text: Option<String> = None;

    while i >= 0 {
        let tok_wrapper = token_stream.get(i);
        let token: &<TS::TF as TokenFactory<'input>>::Inner = tok_wrapper.borrow();
        let token_type = token.get_token_type();

        if token_type == Idl_DocComment {
            doc_token_text = Some(token.get_text().to_display());
            break;
        } else if token_type == Idl_WS || token_type == Idl_EmptyComment {
            // Skip whitespace and empty comments, continue scanning.
            i -= 1;
            continue;
        } else {
            // Hit a non-hidden, non-doc token -- no doc comment for this node.
            break;
        }
    }

    let text = doc_token_text?;

    // Strip the /** prefix and */ suffix.
    let inner = &text[3..text.len() - 2];
    let trimmed = inner.trim();

    if trimmed.is_empty() {
        return None;
    }

    Some(strip_indents(trimmed))
}

/// Strip common indentation from a doc comment body, matching the Java
/// `IdlReader.stripIndents()` behavior.
///
/// Handles two patterns:
/// 1. Star-prefixed: lines starting with `*` or `**` (common in `/** ... */` blocks)
/// 2. Whitespace-indented: strips the common leading whitespace across all lines
pub fn strip_indents(doc_comment: &str) -> String {
    // Try star-prefix pattern first.
    // If all lines after the first start with `*` or `**` (after optional whitespace),
    // strip that prefix.
    if let Some(result) = try_strip_star_indent(doc_comment) {
        return result;
    }

    // Try common whitespace indent.
    if let Some(result) = try_strip_ws_indent(doc_comment) {
        return result;
    }

    doc_comment.to_string()
}

/// Try to strip star-prefixed indentation.
///
/// Matches doc comments like:
/// ```text
/// * First line
/// * Second line
/// ```
/// or:
/// ```text
/// ** First line
/// ** Second line
/// ```
fn try_strip_star_indent(doc_comment: &str) -> Option<String> {
    let lines: Vec<&str> = doc_comment.lines().collect();
    if lines.len() < 2 {
        // Single-line comments don't have star indents to strip.
        return None;
    }

    // Determine star prefix length (1 or 2 stars) from the first line.
    let first_line = lines[0];
    let star_count = if first_line.starts_with("**") {
        2
    } else if first_line.starts_with('*') {
        1
    } else {
        return None;
    };

    let star_prefix = &"**"[..star_count];

    // Verify all subsequent lines (after whitespace trimming) start with the same
    // star prefix.
    for line in &lines[1..] {
        let trimmed = line.trim_start();
        if !trimmed.is_empty() && !trimmed.starts_with(star_prefix) {
            return None;
        }
    }

    // Strip the star prefix from each line.
    let mut result_lines = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            // First line: strip leading stars and optional following space.
            let after_stars = &first_line[star_count..];
            let stripped = if after_stars.starts_with(' ') {
                &after_stars[1..]
            } else {
                after_stars
            };
            result_lines.push(stripped);
        } else {
            let trimmed = line.trim_start();
            if trimmed.is_empty() {
                result_lines.push("");
            } else {
                let after_stars = &trimmed[star_count..];
                let stripped = if after_stars.starts_with(' ') {
                    &after_stars[1..]
                } else {
                    after_stars
                };
                result_lines.push(stripped);
            }
        }
    }

    Some(result_lines.join("\n"))
}

/// Try to strip common whitespace indentation from a multi-line doc comment.
fn try_strip_ws_indent(doc_comment: &str) -> Option<String> {
    let lines: Vec<&str> = doc_comment.lines().collect();
    if lines.len() < 2 {
        return None;
    }

    // Find the common whitespace indent across all lines after the first.
    // The first line's indent was already stripped by trim().
    let mut common_indent: Option<&str> = None;
    for line in &lines[1..] {
        if line.trim().is_empty() {
            continue;
        }
        let indent = &line[..line.len() - line.trim_start().len()];
        common_indent = Some(match common_indent {
            None => indent,
            Some(current) => common_prefix(current, indent),
        });
    }

    let indent = common_indent.unwrap_or("");
    if indent.is_empty() {
        return None;
    }

    let mut result_lines = Vec::new();
    result_lines.push(lines[0]);
    for line in &lines[1..] {
        if line.len() >= indent.len() {
            result_lines.push(&line[indent.len()..]);
        } else {
            result_lines.push(line);
        }
    }

    Some(result_lines.join("\n"))
}

/// Find the common prefix of two strings (character by character).
fn common_prefix<'a>(a: &'a str, b: &str) -> &'a str {
    let len = a
        .chars()
        .zip(b.chars())
        .take_while(|(ca, cb)| ca == cb)
        .count();
    &a[..a.chars().take(len).map(|c| c.len_utf8()).sum::<usize>()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_indents_single_line() {
        assert_eq!(strip_indents("A simple comment."), "A simple comment.");
    }

    #[test]
    fn test_strip_indents_star_prefix() {
        assert_eq!(
            strip_indents("* First line\n * Second line"),
            "First line\nSecond line"
        );
    }

    #[test]
    fn test_strip_indents_double_star() {
        assert_eq!(
            strip_indents("** First line\n ** Second line"),
            "First line\nSecond line"
        );
    }

    #[test]
    fn test_strip_indents_whitespace() {
        assert_eq!(
            strip_indents("First line\n    Second line\n    Third line"),
            "First line\nSecond line\nThird line"
        );
    }
}
