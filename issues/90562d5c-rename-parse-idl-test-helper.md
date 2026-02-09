# Rename `parse_idl` test helper to reflect its CRLF normalization purpose

## Symptom

The `#[cfg(test)]` function `parse_idl` in `src/reader.rs` has a
generic name that suggests it is the primary parsing API, but it is
actually a test convenience wrapper around `parse_idl_named` that
performs CRLF normalization before delegating. The name is misleading:
a reader of the test code would reasonably expect `parse_idl` to be
the main entry point, when the real public API is `parse_idl_named`
(used by the compiler in production).

## Root cause

The function was introduced as a shorthand for unit tests that pass
inline IDL strings. The CRLF normalization was added later to handle
test fixture files with Windows line endings (e.g., the
`doc-comment-crlf-preservation.avdl` regression test). The function
name was never updated to signal its test-specific behavior.

## Affected files

- `src/reader.rs` -- `parse_idl` definition (lines 300-315) and
  ~50 call sites in `mod tests`

## Reproduction

Read `src/reader.rs` lines 300-315: the function is `#[cfg(test)]`,
performs CRLF-to-LF normalization, then delegates to
`parse_idl_named`. The name `parse_idl` does not communicate either
of these facts.

## Suggested fix

Rename `parse_idl` to something like `parse_idl_for_test` or
`parse_idl_normalized`, and update all ~50 call sites in the test
module. The doc comment should explain that callers who need to
preserve CRLF (e.g., to test CRLF-specific behavior) should call
`parse_idl_named` directly.

Alternatively, extract the CRLF normalization into a standalone
`normalize_line_endings` helper and have the test wrapper call it
explicitly, making the normalization step visible at each call site
or in the wrapper's name.

Priority: low. The function is already `#[cfg(test)]` so it has no
production impact. This is a code clarity / naming concern only.

## Related observation

The production code path (`parse_idl_named`, called from
`compiler.rs`) does not normalize CRLF. This means `SourceSpan` byte
offsets in error diagnostics could be off by one per line for CRLF
input files in production. The `doc_comments.rs` regexes correctly
use `\r?\n` patterns, but the ANTLR token byte offsets themselves
are not adjusted. This is a separate potential issue from the naming
concern tracked here.
