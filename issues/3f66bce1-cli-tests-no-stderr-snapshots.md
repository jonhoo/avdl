# CLI tests don't snapshot-test stderr (warnings and errors)

## Symptom

The CLI integration tests in `tests/cli.rs` verify exit codes and
stdout JSON but never assert on stderr content. Warnings (orphaned
doc comments, lexer errors) and error diagnostics are rendered to
stderr in production, but no test confirms the user actually sees
them.

Library-level tests (`tests/integration.rs`,
`tests/error_reporting.rs`) snapshot-test warning and error
rendering through `render_warnings()`, but those exercise
`miette::Report` formatting directly — not the actual CLI stderr
path through `eprintln!("{w:?}")` in `src/main.rs`.

## Root cause

`tests/cli.rs` captures `output.stderr` in some tests but only uses
it for failure diagnostics (e.g., printing stderr when an unexpected
exit code occurs). No test snapshots or asserts on the stderr content
itself.

## Desired tests

At minimum, add two insta snapshot tests to `tests/cli.rs`:

1. **Successful invocation with warnings.** Run `avdl idl` on
   `comments.avdl` (which produces ~27 orphaned-doc-comment
   warnings) and snapshot stderr. This confirms warnings are
   actually emitted through the CLI path, and catches regressions
   in their formatting.

2. **Failing invocation with warnings.** Run `avdl idl` on an input
   that has both orphaned doc comments AND a syntax error, and
   snapshot the full stderr (warnings + error diagnostic). This
   confirms the user sees both the warnings and the error.

## Prerequisite: warnings are lost on error

Currently, warnings are emitted _after_ the `convert()`/`extract()`
call succeeds (`src/main.rs` lines 212–214 and 243–245). When
`convert()` returns `Err`, the `?` propagates the error before the
warning-emission loop runs:

```rust
let idl_output = match &input {
    Some(path) if path != "-" => builder.convert(path)?,  // <-- warnings lost on Err
    ...
};

for w in &idl_output.warnings {
    eprintln!("{w:?}");  // never reached if convert() failed
}
```

So test (2) above will require the CLI to be fixed first: either
emit warnings before propagating the error, or bundle accumulated
warnings into the error path.

## Affected files

- `tests/cli.rs` — add the two snapshot tests
- `src/main.rs` — fix warning loss on error path (for test 2)
