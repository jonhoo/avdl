# Consolidate duplicated test helper functions

## Symptom

Several test helper functions are copy-pasted across test files:

1. **`normalize_crlf`** -- identical in `tests/integration.rs:31` and
   `tests/cli.rs:25`. Recursively replaces `\r\n` with `\n` in JSON
   `Value` trees.

2. **`render_warnings`** -- near-identical in `tests/integration.rs:95`
   and `tests/error_reporting.rs:99`. Both use
   `GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor())`
   with width 80.

3. **`render_error`** -- present in both `tests/integration.rs:82` and
   `src/reader.rs:3168` (test module). The integration.rs version uses
   `unicode_nocolor` theme while the reader.rs version uses
   `GraphicalTheme::none()`. These are intentionally different (one
   matches CLI test style, the other matches error_reporting.rs style),
   but the pattern is worth noting.

## Root cause

Rust's test organization doesn't natively support shared test utility
modules without a `tests/common/mod.rs` (or similar) pattern. Each
test file independently defines its helpers.

## Affected files

- `tests/integration.rs` -- `normalize_crlf`, `render_warnings`,
  `render_error`
- `tests/cli.rs` -- `normalize_crlf`
- `tests/error_reporting.rs` -- `render_warnings`
- `src/reader.rs` (test module) -- `render_error`

## Reproduction

Search for `fn normalize_crlf`, `fn render_warnings`, or
`fn render_error` across the codebase.

## Suggested fix

Create a `tests/common/mod.rs` module with the shared helpers:

```rust
// tests/common/mod.rs
pub fn normalize_crlf(value: serde_json::Value) -> serde_json::Value { ... }
pub fn render_warnings(warnings: &[miette::Report]) -> String { ... }
pub fn render_error(err: &miette::Report) -> String { ... }
```

Then import them in each test file:

```rust
mod common;
use common::{normalize_crlf, render_warnings};
```

The `render_error` in `src/reader.rs` tests uses a different theme
(`none` vs `unicode_nocolor`) so it may need to remain separate or
accept a theme parameter.

This is low priority -- the duplication is small and test-only.
