# CLI does not support `--version` flag

## Symptom

Running `avdl --version` returns an error instead of printing the
version:

```
error: invalid option '--version'
```

Users commonly expect `--version` (or `-V`) to print the version of a
CLI tool. This is especially useful for bug reports and verifying that
the correct version is installed after a Homebrew upgrade.

## Root cause

The `main.rs` CLI parser (using `lexopt`) only handles `--help`/`-h`
at the top level, plus the `idl` and `idl2schemata` subcommands.
There is no case arm for `--version` or `-V`.

## Affected files

- `src/main.rs`

## Reproduction

```sh
avdl --version
# error: invalid option '--version'
```

## Suggested fix

Add a `--version` / `-V` handler to the top-level argument parser in
`main.rs`. The version string can be pulled from `env!("CARGO_PKG_VERSION")`
at compile time. The output format could follow the conventional
`avdl 0.1.4+1.12.1` style.
