# Convert `.contains()` error assertions to insta snapshots

## Symptom

Multiple unit tests use `assert!(msg.contains("..."))` to check error
messages instead of `insta::assert_snapshot!`. The `enrich_*` tests in
`reader.rs` are particularly fragile: they test 3-4 substrings of a
single error message instead of snapshotting the whole `EnrichedError`.

CLAUDE.md error test conventions explicitly say: "Do not use
`format!(\"{err}\")` + `.contains(...)` for new error tests."

## Root cause

These tests predate the adoption of insta snapshots for error testing.

## Affected files

- `src/reader.rs` (unit tests, especially `enrich_*` tests)
- `src/compiler.rs` (unit tests)
- `src/resolve.rs` (unit tests)
- `src/model/schema.rs` (unit tests)

Note: `tests/cli.rs` has a similar pattern (lines 266, 346, 350) but
those are harder to snapshot since they involve subprocess stderr.

## Reproduction

Search for `assert!(.*contains` in `src/`.

## Suggested fix

Replace `assert!(msg.contains("..."))` chains with
`insta::assert_snapshot!` calls. For the `enrich_*` tests, snapshot
the full `EnrichedError` display. This catches regressions in the
complete error output rather than just checking for substring presence.
