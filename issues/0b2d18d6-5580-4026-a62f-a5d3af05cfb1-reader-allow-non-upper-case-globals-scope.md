# Change `#![allow(non_upper_case_globals)]` to `#![expect(...)]` in `reader.rs`

## Symptom

`src/reader.rs` has `#![allow(non_upper_case_globals)]` at line 12, suppressing
the lint for the entire 3900+ line module. This is needed for ANTLR-generated
token constants (`Idl_Boolean`, `Idl_Null`, etc.) but uses `allow` instead of
`expect`.

Using `#[expect]` (stabilized in Rust 1.81) would be strictly better here: it
suppresses the warnings the same way, but the compiler will warn via
`unfulfilled_lint_expectations` if the suppression ever becomes unnecessary
(e.g., if the ANTLR constants are removed or the code is refactored). This is
the intended semantic for known, justified suppressions.

## Root cause

The `#![allow(...)]` was written before `#[expect]` was stabilized, or before
the team adopted the `allow` → `expect` convention.

## Affected files

- `src/reader.rs` (line 12)

## Reproduction

The attribute is visible at the top of the file.

## Suggested fix

Change:
```rust
#![allow(non_upper_case_globals)]
```
to:
```rust
#![expect(non_upper_case_globals, reason = "ANTLR-generated token constants use PascalCase")]
```

The `reason` parameter (also stabilized in 1.81) documents the justification
inline, replacing the need for a separate comment.
