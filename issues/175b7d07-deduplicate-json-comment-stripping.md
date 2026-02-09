# Deduplicate JSON comment-stripping logic in `import.rs`

## Symptom

The expression `serde_json::from_reader(json_comments::CommentSettings::c_style().strip_comments(content.as_bytes()))` is repeated verbatim in two production functions (`import_protocol` at line 290 and `import_schema` at line 344). A test-only helper `parse_json_with_comments` already exists (line 1978) that wraps the same logic, but it lives inside `#[cfg(test)]` and is not available to production code.

## Root cause

The helper was written for the test module only. When the production functions were written (or when the helper was extracted), nobody promoted the helper to production scope for reuse.

## Affected files

- `src/import.rs`
  - `import_protocol` (lines 290-292)
  - `import_schema` (lines 344-346)
  - `parse_json_with_comments` test helper (lines 1978-1982)

## Reproduction

Search for `strip_comments` in `src/import.rs` — three call sites are visible, two of which are identical production code.

## Suggested fix

Extract a small non-test helper function (e.g., `parse_json_with_comments`) at module scope that takes `&str` and returns `Result<Value, serde_json::Error>`:

```rust
fn parse_json_with_comments(input: &str) -> std::result::Result<Value, serde_json::Error> {
    serde_json::from_reader(
        json_comments::CommentSettings::c_style().strip_comments(input.as_bytes()),
    )
}
```

Then call it from both `import_protocol` and `import_schema`, keeping only the `.map_err()` at each call site for context-specific error messages. The test helper can then be removed (or kept as a re-export alias if desired, though calling the production function directly in tests is fine).

This is a low-risk, purely mechanical refactor.
