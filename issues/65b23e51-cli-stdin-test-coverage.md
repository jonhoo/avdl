# Add CLI test coverage for stdin input and `-` output paths

## Symptom

The `idl` subcommand supports reading from stdin (when no input file is
given) and treating `"-"` as stdout for the output path. Neither code
path has CLI-level test coverage in `tests/cli.rs`.

The existing tests cover:
- File-to-stdout (`test_cli_idl_file_to_stdout`)
- File-to-file (`test_cli_idl_file_to_file`)
- Import dirs (`test_cli_idl_import_dir`)
- Nonexistent file error (`test_cli_idl_nonexistent_file`)

Missing coverage:
- **stdin-to-stdout**: `echo '...' | avdl idl`
- **stdin with `-` arg**: `echo '...' | avdl idl -`
- **File to `-` (explicit stdout)**: `avdl idl input.avdl -`

## Root cause

These paths were not included in the initial CLI test suite, likely
because piping stdin in `assert_cmd` requires using `.write_stdin()`
which is slightly more complex than passing file arguments.

## Affected files

- `tests/cli.rs` -- missing test cases

## Reproduction

Not a bug, but a test coverage gap. The stdin code path in
`main.rs:198-206` is untested at the CLI subprocess level.

## Suggested fix

Add tests using `assert_cmd`'s `.write_stdin()` method:

```rust
#[test]
fn test_cli_idl_stdin_to_stdout() {
    let input = r#"protocol P { record R { int x; } }"#;
    let output = avdl_cmd()
        .args(["idl"])
        .write_stdin(input)
        .output()
        .expect("run avdl idl with stdin");
    assert!(output.status.success());
    let actual: Value = serde_json::from_slice(&output.stdout)
        .expect("stdout should be valid JSON");
    assert_eq!(actual["protocol"], "P");
}

#[test]
fn test_cli_idl_explicit_dash_output() {
    let output = avdl_cmd()
        .args(["idl", &format!("{INPUT_DIR}/simple.avdl"), "-"])
        .output()
        .expect("run avdl idl with - output");
    assert!(output.status.success());
    let actual: Value = serde_json::from_slice(&output.stdout)
        .expect("stdout should be valid JSON");
    assert_eq!(actual["protocol"], "Simple");
}
```
