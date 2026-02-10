use std::borrow::Borrow;
use std::collections::HashSet;
use std::sync::LazyLock;

use antlr4rust::char_stream::InputData;
use antlr4rust::token::Token;
use antlr4rust::token_factory::TokenFactory;
use antlr4rust::token_stream::TokenStream;
use regex::Regex;

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
/// the way Java ANTLR does [yet](https://github.com/antlr4rust/antlr4/pull/39), but `get(index)`
/// is public and lets us access any token by index, including hidden-channel tokens.
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
        && let Some(idx) = doc_token_index
    {
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

// ==============================================================================
// Doc comment indent stripping
// ==============================================================================
//
// This is a direct port of Java's `IdlReader.stripIndents()`, which uses two
// regex-based strategies to remove doc comment decoration:
//
// 1. **Star-prefix**: If the comment starts with `*` or `**` and all subsequent
//    lines (after optional horizontal whitespace) start with the same star
//    prefix, strip the stars and one optional trailing whitespace character from
//    each line.
//
// 2. **Common whitespace indent**: If all subsequent lines share a common
//    leading whitespace prefix, strip it.
//
// Java's patterns use `\h` (horizontal whitespace) and `\R` (any line break).
// Rust's `regex` crate lacks these, so we use `[\t ]` and `\r?\n` respectively.
// Java also uses backreferences (`\k<stars>`) to match the same star count on
// subsequent lines; we avoid backreferences (unsupported in `regex`) by checking
// star count manually and building the replacement pattern from it.

/// Validation pattern for single-star prefix: the comment starts with `*` and
/// all subsequent lines start with `*` after optional horizontal whitespace.
/// Empty lines are allowed (they don't need a star).
static STAR_1_VALIDATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)^\*[^\r\n]*(?:\r?\n[\t ]*\*[^\r\n]*|\r?\n[\t ]*)*$").unwrap()
});

/// Validation pattern for double-star prefix: same as above but with `**`.
static STAR_2_VALIDATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?s)^\*\*[^\r\n]*(?:\r?\n[\t ]*\*\*[^\r\n]*|\r?\n[\t ]*)*$").unwrap()
});

/// Replacement pattern for single-star: matches start-of-string or
/// (newline + horizontal whitespace) followed by `*` and optional trailing
/// horizontal whitespace.
///
/// Java equivalent: `(?U)(?:^|(\R)\h*)\Q*\E\h?`
static STAR_1_REPLACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|(\r?\n)[\t ]*)\*[\t ]?").unwrap());

/// Replacement pattern for double-star.
///
/// Java equivalent: `(?U)(?:^|(\R)\h*)\Q**\E\h?`
static STAR_2_REPLACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|(\r?\n)[\t ]*)\*\*[\t ]?").unwrap());

/// Validation pattern for common whitespace indent: the comment has at least
/// two lines and all non-empty subsequent lines share a common leading
/// whitespace prefix. We determine the actual indent length manually.
static WS_INDENT_VALIDATE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)[^\r\n]*\r?\n[\t ]+[^\r\n]*").unwrap());

/// Strip common indentation from a doc comment body, matching Java's
/// `IdlReader.stripIndents()` behavior.
///
/// Handles two patterns:
/// 1. Star-prefixed: lines starting with `*` or `**` (common in `/** ... */` blocks)
/// 2. Whitespace-indented: strips the common leading whitespace across all lines
pub fn strip_indents(doc_comment: &str) -> String {
    // Try double-star prefix first (longer match takes priority, matching Java's
    // `\\*{1,2}` greedy quantifier behavior).
    if STAR_2_VALIDATE.is_match(doc_comment) {
        return STAR_2_REPLACE
            .replace_all(doc_comment, |caps: &regex::Captures| {
                // Preserve the newline (group 1) if present; at start-of-string
                // there is no newline to preserve.
                caps.get(1).map_or("", |m| m.as_str()).to_string()
            })
            .into_owned();
    }

    if STAR_1_VALIDATE.is_match(doc_comment) {
        return STAR_1_REPLACE
            .replace_all(doc_comment, |caps: &regex::Captures| {
                caps.get(1).map_or("", |m| m.as_str()).to_string()
            })
            .into_owned();
    }

    // Try common whitespace indent. We find the common indent prefix manually
    // (since Rust regex doesn't support backreferences) and then build a
    // replacement regex from it.
    if WS_INDENT_VALIDATE.is_match(doc_comment)
        && let Some(result) = try_strip_ws_indent(doc_comment)
    {
        return result;
    }

    doc_comment.to_string()
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

    // Build a replacement regex: after each newline, strip exactly the common
    // indent prefix. Java equivalent: `(?U)(\R)<indent>`
    let escaped_indent = regex::escape(indent);
    let re = Regex::new(&format!(r"(\r?\n){}", escaped_indent))
        .expect("escaped indent produces valid regex");
    Some(re.replace_all(doc_comment, "$1").into_owned())
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
    use pretty_assertions::assert_eq;

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

    #[test]
    fn test_strip_indents_tab_after_star() {
        // Tab after star should be stripped just like a space (issue 17c10dd1).
        // Multi-line: `*\ttext` on continuation lines.
        let input = "*\tFirst line\n *\tSecond line";
        assert_eq!(strip_indents(input), "First line\nSecond line");
    }

    #[test]
    fn test_strip_indents_single_line_tab_after_star() {
        // Single-line: `*\ttext` should strip the tab.
        assert_eq!(strip_indents("*\ttext"), "text");
    }

    #[test]
    fn test_strip_indents_single_line_tab_after_double_star() {
        // Single-line: `**\ttext` should strip the tab.
        assert_eq!(strip_indents("**\ttext"), "text");
    }

    #[test]
    fn test_strip_indents_mixed_whitespace_around_stars() {
        // Tabs and spaces around the star prefix on continuation lines.
        // The regex approach handles any horizontal whitespace uniformly.
        let input = "* First\n\t * Second\n \t* Third";
        assert_eq!(strip_indents(input), "First\nSecond\nThird");
    }
}
