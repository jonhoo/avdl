use std::borrow::Borrow;
use std::collections::HashSet;

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
/// If `consumed_indices` is provided, the index of the consumed doc comment
/// token is recorded so callers can later detect orphaned (unconsumed) doc
/// comments and generate warnings.
///
/// antlr4rust's `CommonTokenStream` does not expose `getHiddenTokensToLeft()`
/// the way Java ANTLR does, but `get(index)` is public and lets us access
/// any token by index, including hidden-channel tokens.
pub fn extract_doc_comment<'input, TS>(
    token_stream: &TS,
    token_index: isize,
    consumed_indices: Option<&mut HashSet<isize>>,
) -> Option<String>
where
    TS: TokenStream<'input>,
{
    if token_index <= 0 {
        return None;
    }

    let mut i = token_index - 1;
    let mut doc_token_text: Option<String> = None;
    let mut doc_token_index: Option<isize> = None;

    while i >= 0 {
        let tok_wrapper = token_stream.get(i);
        let token: &<TS::TF as TokenFactory<'input>>::Inner = tok_wrapper.borrow();
        let token_type = token.get_token_type();

        if token_type == Idl_DocComment {
            doc_token_text = Some(token.get_text().to_display());
            doc_token_index = Some(i);
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

    // Record the consumed token index so we can later detect orphaned doc
    // comments (those not consumed by any declaration).
    if let Some(consumed) = consumed_indices
        && let Some(idx) = doc_token_index {
            consumed.insert(idx);
        }

    // Strip the /** prefix and */ suffix.
    let inner = text
        .strip_prefix("/**")
        .and_then(|s| s.strip_suffix("*/"))
        .unwrap_or(&text);
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

    // Handle single-line comments that start with a star prefix (e.g. `* text`
    // from `/** * text */`). The multi-line star stripper requires >= 2 lines,
    // so we handle this case separately.
    if let Some(stripped) = doc_comment.strip_prefix("** ") {
        return stripped.to_string();
    }
    if let Some(stripped) = doc_comment.strip_prefix("**") {
        return stripped.to_string();
    }
    if let Some(stripped) = doc_comment.strip_prefix("* ") {
        return stripped.to_string();
    }
    if let Some(stripped) = doc_comment.strip_prefix('*') {
        return stripped.to_string();
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
            let stripped = if let Some(s) = after_stars.strip_prefix(' ') {
                s
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
                let stripped = if let Some(s) = after_stars.strip_prefix(' ') {
                    s
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

    // =========================================================================
    // Edge-case tests for `strip_indents` (issue #12)
    // =========================================================================

    #[test]
    fn test_strip_indents_empty_string() {
        // An empty doc comment body (e.g. from `/** */` after trim) should
        // pass through unchanged.
        assert_eq!(strip_indents(""), "");
    }

    #[test]
    fn test_strip_indents_single_line_star_prefix_with_space() {
        // `/** * text */` -> after stripping delimiters and trim -> `* text`
        assert_eq!(strip_indents("* text"), "text");
    }

    #[test]
    fn test_strip_indents_single_line_star_prefix_no_space() {
        // `/** *text */` -> after stripping delimiters and trim -> `*text`
        assert_eq!(strip_indents("*text"), "text");
    }

    #[test]
    fn test_strip_indents_single_line_double_star_with_space() {
        // `/** ** text */` -> after stripping delimiters and trim -> `** text`
        assert_eq!(strip_indents("** text"), "text");
    }

    #[test]
    fn test_strip_indents_single_line_double_star_no_space() {
        // `/** **text */` -> after stripping delimiters and trim -> `**text`
        assert_eq!(strip_indents("**text"), "text");
    }

    #[test]
    fn test_strip_indents_multi_line_star_prefix_with_blank_lines() {
        // Blank lines between star-prefixed lines should be preserved as
        // empty lines, not cause the star pattern to fail.
        let input = "* First line\n\n * Second line";
        assert_eq!(strip_indents(input), "First line\n\nSecond line");
    }

    #[test]
    fn test_strip_indents_multi_line_with_tabs() {
        // Tab-indented subsequent lines should have the common tab indent
        // stripped.
        let input = "First line\n\tSecond line\n\tThird line";
        assert_eq!(strip_indents(input), "First line\nSecond line\nThird line");
    }

    #[test]
    fn test_strip_indents_multi_line_mixed_indent_depth() {
        // When subsequent lines have varying indent depths, only the common
        // prefix should be stripped.
        let input = "First\n    Second\n        Third";
        assert_eq!(strip_indents(input), "First\nSecond\n    Third");
    }

    #[test]
    fn test_strip_indents_unicode() {
        // Unicode content in doc comments should be preserved correctly.
        assert_eq!(
            strip_indents("* Ünïcödé text\n * More ünïcödé"),
            "Ünïcödé text\nMore ünïcödé"
        );
    }

    #[test]
    fn test_strip_indents_single_star_only() {
        // A doc comment body that is just `*` with no following text.
        assert_eq!(strip_indents("*"), "");
    }

    #[test]
    fn test_strip_indents_double_star_only() {
        // A doc comment body that is just `**` with no following text.
        assert_eq!(strip_indents("**"), "");
    }

    #[test]
    fn test_strip_indents_multi_line_star_no_space_after_star() {
        // Multi-line where stars are not followed by a space.
        let input = "*First line\n *Second line";
        assert_eq!(strip_indents(input), "First line\nSecond line");
    }

    #[test]
    fn test_strip_indents_no_common_indent() {
        // Multi-line where subsequent lines have no common indent -- the
        // input should be returned unchanged.
        let input = "First line\nSecond line\nThird line";
        assert_eq!(strip_indents(input), "First line\nSecond line\nThird line");
    }

    #[test]
    fn test_strip_indents_whitespace_only_lines_ignored_for_indent() {
        // Blank or whitespace-only subsequent lines should not affect the
        // common indent calculation.
        let input = "First\n    Second\n\n    Third";
        assert_eq!(strip_indents(input), "First\nSecond\n\nThird");
    }

    #[test]
    fn test_strip_indents_single_line_plain_text() {
        // Plain text with no star prefix should pass through unchanged.
        assert_eq!(strip_indents("Just some text"), "Just some text");
    }

    #[test]
    fn test_strip_indents_multi_line_double_star_with_blank_lines() {
        // Double-star prefix with blank lines interspersed.
        let input = "** First\n\n ** Second\n ** Third";
        assert_eq!(strip_indents(input), "First\n\nSecond\nThird");
    }
}
