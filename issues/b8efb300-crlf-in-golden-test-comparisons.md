# CRLF line endings cause golden-file test failures on Windows

## Symptom

On Windows, `test_cli_idl_import_dir` (`tests/cli.rs:111`) fails
because the `"doc"` field in the JSON output contains `\r\n` while the
golden `.avpr` file has `\n`. CI exits on first failure, but the
integration tests (`tests/integration.rs`) have the same issue.

## Root cause

On Windows, Git checks out `.avdl` source files with `\r\n` line
endings. The doc comment extraction intentionally preserves whatever
line endings the input contains. The golden `.avpr` files use `\n`.
Both test suites compare `serde_json::Value` trees where string equality
is byte-exact, so `"foo\r\nbar" != "foo\nbar"`.

## Affected tests

Any test comparing parsed IDL output against golden files where the
`.avdl` input has multi-line doc comments. Golden `.avpr` files with
multi-line `"doc"` values: `import`, `nestedimport`, `baseball`,
`interop`, `reservedwords`, `namespaces`, `mr_events`, `unicode`.

In `tests/cli.rs`: `test_cli_idl_import_dir` (and potentially
`test_cli_idl_file_to_stdout`, `test_cli_idl_file_to_file`).

In `tests/integration.rs`: `test_import`, `test_nestedimport`, and all
tests exercised by `test_idl2schemata_golden_comparison`.

## Suggested fix

Add a test-only `normalize_crlf(value: Value) -> Value` helper to both
test files that recursively replaces `\r\n` with `\n` in all JSON
string values. Takes an owned `Value` to avoid cloning. Apply it to
both sides of every `assert_eq!` that compares against golden files.
It's a no-op on Linux.

The production code should **not** normalize — the tool preserves
whatever line endings the user's input contains.

```rust
fn normalize_crlf(value: Value) -> Value {
    match value {
        Value::String(s) => Value::String(s.replace("\r\n", "\n")),
        Value::Array(arr) => Value::Array(arr.into_iter().map(normalize_crlf).collect()),
        Value::Object(obj) => {
            Value::Object(obj.into_iter().map(|(k, v)| (k, normalize_crlf(v))).collect())
        }
        other => other,
    }
}
```

## Affected files

- `tests/cli.rs` — add helper, wrap ~3 asserts
- `tests/integration.rs` — add helper, wrap ~20 asserts
