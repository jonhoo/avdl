# Remaining test coverage gaps

## Symptom

The integration test suite has grown substantially but still has two
notable gaps from the original tracking issue.

## Status

Re-opened during audit of deleted issues. Original issue: `24-test-suite-coverage-gaps.md`.

## Evidence of partial fix

The following gaps from the original issue have been resolved:

- Gap 1: `import.avdl` and `nestedimport.avdl` tests added.
- Gap 2: `idl2schemata` tests for `interop.avdl` and `import.avdl` added.
- Gap 3: `putOnClassPath/` import resolution covered by `test_import`.
- Gap 4: Import cycle detection tests (`test_self_import_cycle_handled_gracefully`,
  `test_mutual_import_cycle_handled_gracefully`) added.
- Gap 5: Doc comment warning test (`test_comments_warnings_count`) added.
- Gap 6: Logical type propagation tests added.
- Gap 6b: `extra/` directory tests added.
- Gap 9b: Tools golden files (`test_tools_schema`, `test_tools_protocol`) added.
- Gap 9 partial: `idl2schemata` file count assertion (`test_idl2schemata_tools_protocol`) added.
- Workspace path: `test_workspace_path` added.

## Remaining work

### Gap 7: Second `cycle.avdl` variant -- RESOLVED

Resolved: `test_cycle_test_root` added to `tests/integration.rs`.

### Gap 8: CLI-level integration tests (Low priority, deferred)

All existing integration tests call the library API directly. No tests
exercise the CLI binary, including stdin/stdout piping, `--import-dir`
flag parsing, error output formatting, or exit codes. This means the
broken-pipe fix in `write_output` and the `idl2schemata` required-arg
change have no dedicated regression tests.

This is deferred as low priority. The library-level integration tests
cover the parsing and serialization logic thoroughly, and the
`compare-golden.sh` script exercises the CLI end-to-end.

## Affected files

- `tests/integration.rs`

## Reproduction

Gap 8: No test exercises `cargo run -- idl` or `cargo run -- idl2schemata`
as a subprocess to verify CLI behavior (exit codes, error formatting,
pipe handling).
