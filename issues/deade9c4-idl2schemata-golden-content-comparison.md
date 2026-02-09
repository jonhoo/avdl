# `idl2schemata` golden-file tests compare metadata only, not full content

## Symptom

The `test_idl2schemata_golden_comparison` test compares `idl2schemata`
output against golden `.avpr` files, but only asserts metadata: schema
count, type kind (`record`/`enum`/`fixed`/`error`), and namespace.
It does not compare the full JSON content of each individual `.avsc`
schema (fields, defaults, doc comments, annotations, ordering).

A bug in field serialization, default value handling, or annotation
propagation within `idl2schemata` output would not be caught.

## Root cause

The golden `.avpr` files contain type definitions inline within a
protocol, not as standalone `.avsc` files. Extracting full type
definitions from `.avpr` files for comparison against standalone
`.avsc` output is non-trivial because `idl2schemata` must inline
cross-references that are bare strings in the protocol context.

The `compare_schemata` helper in `test_idl2schemata_golden_comparison`
was written as a structural metadata check rather than a full content
comparison.

## Affected files

- `tests/integration.rs` -- `test_idl2schemata_golden_comparison`

## Reproduction

The test passes even if `idl2schemata` output has incorrect field
types, missing doc comments, or wrong default values, as long as
the schema names, kinds, and namespaces match the golden `.avpr`.

The `scripts/compare-golden.sh idl2schemata` script performs full
content comparison and currently passes, but this is not part of
`cargo test`.

## Suggested fix

Generate committed golden `.avsc` files from the Java tool's
`idl2schemata` output and compare each Rust-produced schema against
the corresponding golden file via `serde_json::Value` equality.
This would catch content-level regressions in `idl2schemata` output.

Priority: medium. The `idl` golden-file tests catch most content
issues since the same serialization code is used, but `idl2schemata`
has its own inlining logic that could diverge.

## Provenance

Re-filed from deleted issue
`200f5d4d-idl2schemata-golden-file-tests-missing.md`. The original
issue requested full `.avsc` JSON comparison; the current test
provides metadata-level comparison only.
