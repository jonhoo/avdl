# `idl2schemata` integration tests lack golden-file comparison

## Symptom

The `idl2schemata` integration tests (`test_idl2schemata_echo`,
`test_idl2schemata_simple`, `test_idl2schemata_interop`,
`test_idl2schemata_import`, `test_idl2schemata_tools_protocol`) only
perform structural assertions (type names, field counts, inlining
behavior). They do not compare the serialized `.avsc` JSON against the
golden output files produced by the Java tool.

The `scripts/compare-golden.sh idl2schemata` script does perform these
comparisons and currently passes all 62 files. However, these
comparisons are not part of `cargo test` and could silently regress if
the script is not run.

## Root cause

The `idl2schemata` tests were originally written with ad-hoc structural
assertions rather than golden-file comparisons. The `idl` subcommand
tests (`test_simple`, `test_echo`, etc.) do compare against golden files
via `load_expected()` and `assert_eq!`, but the `idl2schemata` tests
were not given the same treatment.

## Affected files

- `tests/integration.rs` -- `idl2schemata` test functions

## Reproduction

Run `scripts/compare-golden.sh idl2schemata` to see 62 passing
comparisons. Then observe that `cargo test` runs only structural
assertions for `idl2schemata` output:

```sh
cargo test test_idl2schemata -- --nocapture
# Only structural assertions, no golden-file comparisons
```

## Suggested fix

For each `idl2schemata` test case, serialize each named schema to JSON
and compare against the corresponding `.avsc` golden file in
`avro/lang/java/idl/src/test/idl/output/`. The golden output directory
does not contain per-schema `.avsc` files for most test cases (it has
`.avpr` files instead), so golden `.avsc` files could be generated via
`scripts/compare-golden.sh idl2schemata` and committed, or the test
could compare against Java tool output.

Alternatively, the most practical approach is to run the Rust
`idl2schemata` pipeline and compare each output `.avsc` file against the
one produced by `java -jar avro-tools idl2schemata`, caching the Java
outputs as committed golden files.

Priority: medium. The script covers this, but CI-only or non-script
environments would miss regressions.
