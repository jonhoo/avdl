# `idl2schemata` error path not end-to-end tested

## Symptom

When `idl2schemata` encounters unresolved type references, it should
report an error via the `validate_references()` check at the end of
`run_idl2schemata`. This error path is only tested indirectly through
`resolve.rs` unit tests (`test_validate_references`,
`test_validate_nested_references`). No test exercises the full
`run_idl2schemata` pipeline with invalid input to verify the error is
surfaced correctly.

## Root cause

The `idl2schemata` integration tests were written for the happy path
only. The existing negative tests in `integration.rs`
(`test_duplicate_type_definition`, `test_import_nonexistent_file`, etc.)
all target the `idl` parsing path, not the `idl2schemata` output path.

## Affected files

- `tests/integration.rs` -- missing negative test for `idl2schemata`
- `src/main.rs` -- `run_idl2schemata` error handling at lines 236-264

## Reproduction

No existing test exercises the `idl2schemata` error path. An example of
input that should trigger the error:

```avdl
@namespace("test")
protocol P {
    record R {
        MissingType field;
    }
}
```

Running `cargo run -- idl2schemata` on this input should produce an
"Undefined name: test.MissingType" error, but no test verifies this.

## Suggested fix

Add an integration test that parses an `.avdl` file with an unresolved
type reference through the `idl2schemata` pipeline and asserts that:

1. The error is reported (not silently producing `.avsc` files with bare
   string references).
2. The error message contains the unresolved type name.

This could use the `parse_idl2schemata` helper but with a registry that
contains an unresolved reference, or use the CLI as a subprocess (which
would also address Gap 8 from issue `5b2199d6`).

Priority: medium. The error path exists and works correctly (verified by
unit tests), but a regression in the integration between parsing,
registry validation, and error reporting would not be caught.
