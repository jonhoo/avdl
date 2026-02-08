# Replace `IdlError` enum with `miette::Result` everywhere

## Motivation

This crate is binary-only — no downstream library consumers need to
match on `IdlError` variants. The structured enum adds complexity
without benefit. `miette::Result` (i.e., `Result<T, miette::Report>`)
provides the same user-facing error rendering with less boilerplate.

## Current state

`src/error.rs` defines:
- `IdlError` enum with 4 variants: `Parse(String)`, `Diagnostic(ParseDiagnostic)`, `Io { source }`, `Other(String)`
- `Result<T>` type alias for `std::result::Result<T, IdlError>`
- Manual `Diagnostic` impl on `IdlError` that delegates to `ParseDiagnostic`

`IdlError` is used in ~60 call sites across `reader.rs`, `import.rs`,
and `main.rs`.

## Plan

### Step 1: Keep `ParseDiagnostic`, delete `IdlError`

`ParseDiagnostic` carries `miette::SourceCode` and `SourceSpan` for
rich error rendering — it must stay. It already implements
`miette::Diagnostic`, so wrapping it in `miette::Report` works
automatically.

Delete `IdlError`, its `Diagnostic` impl, and the `Result<T>` alias.

### Step 2: Switch all functions to `miette::Result<T>`

- `reader.rs`: change `use crate::error::{IdlError, ParseDiagnostic, Result}`
  to `use crate::error::ParseDiagnostic` and `use miette::Result`
- `import.rs`: change `use crate::error::{IdlError, Result}` to
  `use miette::Result`
- `main.rs`: change `use avdl::error::IdlError` to nothing (or just
  `use miette::Result`)

### Step 3: Replace `IdlError` variant constructors

| Old pattern | New pattern |
|---|---|
| `IdlError::Parse(msg)` | `miette::bail!("parse error: {msg}")` or `Err(miette::miette!(...))` |
| `IdlError::Diagnostic(d)` | `Err(d.into())` (automatic via `Diagnostic` impl) |
| `IdlError::Io { source: e }` | `Err(e).wrap_err("context")` or `.into_diagnostic().wrap_err(...)` |
| `IdlError::Other(msg)` | `miette::bail!("{msg}")` |
| `.map_err(\|e\| IdlError::Other(format!(...)))` | `.into_diagnostic().wrap_err(...)` or `.map_err(\|e\| miette::miette!(...))` |

The helper functions `make_error` and `make_type_error` in `reader.rs`
that construct `IdlError::Diagnostic(ParseDiagnostic { ... })` should
return `miette::Report` instead (just drop the `IdlError::` wrapper).

### Step 4: Simplify `src/error.rs`

After the migration, `error.rs` should contain only `ParseDiagnostic`.
Consider whether it's worth keeping as a separate module or inlining
into `reader.rs` (the only consumer of source-span diagnostics).

### Step 5: Update tests

- `tests/integration.rs`: update any `.is_err()` or error-matching
  patterns
- `tests/error_reporting.rs`: snapshot tests should still work since
  they render via `Display`/`Diagnostic`, not pattern matching

## Scope

~60 call sites across 3 files. Mechanical refactor — no behavior
change. The rendered error messages should be identical.

## Verification

- `cargo test` — all 281 tests pass
- `scripts/compare-golden.sh idl` — 18/18
- `scripts/compare-golden.sh idl2schemata` — 62/62
- Spot-check error rendering: parse a malformed `.avdl` and verify
  miette still renders source spans with carets
