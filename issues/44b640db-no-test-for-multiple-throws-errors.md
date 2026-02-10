# No test for message declarations with multiple `throws` error types

## Symptom

The ANTLR grammar allows multiple error types in a `throws` clause:

    messageDeclaration:
      ... (oneway=Oneway | Throws errors+=identifier (Comma errors+=identifier)*)? Semicolon;

The only `throws` usage in the test suite is `simple.avdl`:

    void `error`() throws TestError;

No test verifies that `throws Error1, Error2` produces the correct
JSON output with multiple error types in the `"errors"` array. The
implementation in `walk_message_declaration` (reader.rs ~2294) iterates
over `ctx.errors`, but this code path has never been exercised by a
test with more than one error type.

## Root cause

The upstream Java golden test suite (`simple.avdl`) only tests a single
`throws` error type. No additional coverage was added when porting.

## Affected files

- `src/reader.rs` (the `walk_message_declaration` function, errors
  handling loop)
- `tests/integration.rs` (no test case for multiple throws)

## Reproduction

No test currently exercises this. An ad-hoc test:

```avdl
protocol P {
    error Err1 { string message; }
    error Err2 { string reason; }
    void dangerous() throws Err1, Err2;
}
```

The expected JSON output should have:

```json
"errors": ["Err1", "Err2"]
```

## Suggested fix

Add an integration test or unit test that parses a protocol with
`throws Err1, Err2` and verifies the resulting `Message.errors` list
contains both error references. Also verify the JSON serialization
produces the correct `"errors"` array.
