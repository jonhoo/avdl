# `extract_doc_comment` uses fragile index slicing for prefix/suffix

## Symptom

The `/**` prefix and `*/` suffix are stripped using hardcoded slice
indices (`text[3..text.len() - 2]`) without validating that the
stripped characters are actually those strings. A malformed token
could cause silent wrong output or a panic on non-ASCII input.

## Root cause

Direct index slicing assumes exact byte widths without assertion.

## Location

- `src/doc_comments.rs:53-54` — `let inner = &text[3..text.len() - 2];`

## Expected behavior

Use `strip_prefix("/**")` and `strip_suffix("*/")` with
`expect(...)`, or add debug assertions to validate the assumption.

## Difficulty

Easy — 5-line change.
