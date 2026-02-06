# Persistent `cargo check` warnings from `error.rs`

## Symptom

`cargo check` produces 4 "value assigned to ... is never read"
warnings for the `ParseDiagnostic` struct fields in `src/error.rs`:

```
warning: value assigned to `src` is never read
 --> src/error.rs:9:9
warning: value assigned to `span` is never read
 --> src/error.rs:11:9
warning: value assigned to `message` is never read
 --> src/error.rs:12:9
```

These appear on every build and clutter the output.

## Root cause

The `ParseDiagnostic` struct uses `#[derive(Debug, Diagnostic, Error)]`
from `miette`, with fields annotated by `#[source_code]`, `#[label]`,
and `#[help]`. The struct is constructed via `make_diagnostic()` which
initializes the fields, but the compiler's `unused_assignments` lint
fires because it doesn't understand that `miette`'s derive macros read
these fields during error formatting.

The warnings are likely caused by the struct using `pub` fields that
are set during construction but only read by the derived `Diagnostic`
impl, which the lint analysis doesn't trace through.

## Affected files

- `src/error.rs` — `ParseDiagnostic` struct definition

## Reproduction

```sh
cargo check 2>&1 | grep warning
```

## Suggested fix

Options (in preference order):

1. Add a targeted `#[allow(unused_assignments)]` on the struct or its
   fields if the derive macros are triggering the false positive.
2. Restructure the struct construction to avoid the pattern that
   triggers the lint (e.g., use a builder pattern or direct struct
   literal).
3. Add `#![allow(unused_assignments)]` at the module level in
   `error.rs` with a comment explaining why.

## Priority

Low. These are cosmetic warnings that don't affect correctness, but
they make it harder to spot real warnings during development.
