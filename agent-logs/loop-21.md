# Loop 21: Lint Audit

## Focus

Narrowly scoped to Rust and clippy lint hygiene: auditing existing
`#[allow]` suppressions, running clippy with extended lint groups,
and establishing a centralized lint policy.

## Phase 1: Discovery

Three focused agents audited:
1. All `#[allow]` / `#![allow]` in `src/` (excluding generated code)
2. `cargo clippy --all-targets` with `clippy::all` and pedantic lints
3. Project lint configuration (Cargo.toml, clippy.toml, crate-level
   attributes)

**8 issues filed**, 0 duplicates. The codebase was remarkably clean —
only one `#[allow]` in non-generated code (`non_upper_case_globals`
in `reader.rs`), and `clippy::all` only had 11 warnings total.

## Phase 2: Resolution

### Wave 1 (parent-applied batch)
- `#![allow(non_upper_case_globals)]` → `#![expect(...)]` in reader.rs
- `#[allow(deprecated)]` → `#[expect(...)]` in tests/cli.rs
- `#[must_use]` on `Idl::new()` and `Idl2Schemata::new()`
- Collapsed nested `if` blocks in tests/error_reporting.rs

### Wave 2 (two parallel agents)
- Replaced `3.14`/`2.718` test values with `3.25`/`2.75` to avoid
  false `approx_constant` warnings (per user preference, instead of
  adding lint suppressions)
- Removed unnecessary `Result`/`Option` wrapping from three
  infallible functions (`extract_impl`, `string_to_schema`,
  `append_quoting_hint`)

### Wave 3 (single agent, large refactor)
- Changed ~18 function signatures from `&Option<T>` to `Option<&T>`
  across reader.rs, json.rs, resolve.rs, and main.rs

### Wave 4 (parent-applied)
- Added `[lints]` section to Cargo.toml: `unsafe_code = "forbid"`,
  `clippy::all = "warn"`, plus four pedantic lints that the codebase
  now satisfies

## Outcome

- 5 commits landed on main (one per wave, clean cherry-picks)
- All 705 tests pass, `cargo clippy --all-targets` clean
- `issues/` directory empty
- No CHANGELOG updates (all changes are internal code quality)

## Observations

- The codebase had very few lint issues — a sign of maturity after
  20 prior iterations.
- `#[expect]` (Rust 1.81+) is strictly better than `#[allow]` for
  known, justified suppressions. Future iterations should use
  `#[expect]` for any new suppressions.
- `doc_markdown`, `missing_errors_doc`, and `missing_panics_doc`
  pedantic lints fire 230+ times. A dedicated documentation pass
  would be needed to enable them.
