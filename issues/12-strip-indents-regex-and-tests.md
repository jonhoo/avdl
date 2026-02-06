# `strip_indents` should use regex and needs expanded test coverage

## Symptom

The current `strip_indents` implementation uses manual string slicing
and prefix checks. The test suite covers only 4 basic cases.

## Root cause

The Java implementation uses regex patterns for doc comment indent
stripping. The Rust port uses handwritten logic that works for common
cases but may diverge on edge cases.

## Location

- `src/doc_comments.rs:70-100` — `strip_indents` and helpers
- `src/doc_comments.rs:226-254` — tests

## Expected behavior

Consider rewriting with the `regex` crate to match Java behavior
exactly. Add test cases for:
- Mixed star prefixes and whitespace
- Tabs vs spaces
- Empty lines in the middle of comments
- Trailing whitespace
- Unicode whitespace characters
- Single-line comments with various star counts
- Multi-line comments where not all lines have star prefixes

## Difficulty

Medium.
