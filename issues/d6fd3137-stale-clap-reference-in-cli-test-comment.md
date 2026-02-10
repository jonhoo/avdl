# Test comment references "clap" but CLI uses `lexopt`

## Symptom

The doc comment on `test_cli_idl2schemata_missing_input` in
`tests/cli.rs` line 275 says:

> Run `avdl idl2schemata` with no arguments and verify a non-zero exit
> code, since clap requires the input argument.

The CLI does not use `clap`. It uses `lexopt` for argument parsing
(`src/main.rs` line 14: `use lexopt::prelude::*;`). The word "clap" in
the comment is incorrect and suggests a dependency that does not exist.

## Root cause

The comment was likely written (or templated) when considering `clap` as
the CLI framework, and not updated after `lexopt` was chosen instead.

## Affected files

- `tests/cli.rs` line 275

## Reproduction

Read the comment at line 274-275 of `tests/cli.rs` and compare with the
actual imports in `src/main.rs`.

## Suggested fix

Replace "since clap requires the input argument" with "since the
`idl2schemata` subcommand requires an input argument", or simply
"since the input argument is required".
