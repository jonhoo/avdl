# CLI-level integration tests missing

## Symptom

All existing integration tests call the library API directly via the
`Idl` and `Idl2Schemata` builder structs. No tests exercise the CLI
binary as a subprocess, including:

- stdin/stdout piping
- `--import-dir` flag parsing
- Error output formatting and exit codes
- The broken-pipe fix in `write_output`

This means regressions in CLI argument parsing or output routing would
not be caught by `cargo test`.

## Root cause

The integration test suite was built around the library API for
convenience. The `compare-golden.sh` script exercises the CLI
end-to-end but is not part of `cargo test` and is not run in CI.

## Affected files

- `tests/integration.rs` -- missing CLI subprocess tests
- `src/main.rs` -- `write_output`, argument parsing, error formatting

## Reproduction

Run `cargo test` and observe that no test invokes the built binary
as a subprocess. The CLI behavior is only verified manually or via
`scripts/compare-golden.sh`.

## Suggested fix

Add a small set of CLI integration tests using
`std::process::Command` to invoke the built binary:

1. `idl` subcommand with file input and stdout output
2. `idl` with `--import-dir` flag
3. `idl2schemata` with directory output
4. Error case: nonexistent input file (verify non-zero exit code)
5. Error case: missing required argument (verify usage message)

Priority: low. The library-level integration tests cover the core
logic thoroughly, and `compare-golden.sh` covers the CLI path.

## Provenance

Re-filed from deleted issue `5b2199d6-remaining-test-coverage-gaps.md`
(Gap 8). All other gaps from that issue have been resolved.
