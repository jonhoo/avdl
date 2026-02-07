# Unresolved type references should be hard errors, not warnings

## Symptom

When a bare type name references a type in a different namespace (e.g.,
`Inner` used from the `org.outer` context when `Inner` is in `org.inner`),
the Rust tool emits a warning to stderr but still produces JSON output with
exit code 0. The resulting JSON contains a bare string reference like
`"Container"` that is semantically incorrect -- in the protocol's namespace
context, the bare name resolves to a non-existent type.

Java rejects these inputs with a hard error:
`org.apache.avro.AvroTypeException: Undefined schema: org.outer.Container`

## Root cause

In `main.rs`, `validate_references()` is called after JSON output is
already written. Unresolved references are reported as warnings on stderr
but the exit code remains 0. The output JSON is already written and
contains incorrect bare name strings.

Java's `ParseContext.find()` returns an unresolved schema placeholder
during parsing, and `ParseContext.resolveAllSchemas()` at the end raises
an exception if any schemas remain unresolved.

## Affected files

- `src/main.rs` (lines 130-138, 202-211): warning emission after output
- `src/resolve.rs`: `validate_references()` returns a list but callers
  treat it as non-fatal

## Reproduction

```avdl
@namespace("org.outer")
protocol NsRefTest {
  @namespace("org.inner")
  record Inner {
    string name;
  }

  record Outer {
    Inner nested;
  }
}
```

```sh
# Rust: produces output + warning, exit code 0
cargo run -- idl tmp/record-ns-inherit.avdl
# stderr: warning: unresolved type references: org.outer.Inner

# Java: hard error, exit code 1
java -jar ../avro-tools-1.12.1.jar idl tmp/record-ns-inherit.avdl
# Exception: Undefined schema: org.outer.Inner
```

## Suggested fix

Make unresolved type references a hard error (non-zero exit code). Two
approaches:

1. **Validate before writing output**: move `validate_references()` before
   JSON serialization and return an error if any references are unresolved.
   This matches Java's fail-fast behavior.

2. **Error after output**: keep the current structure but change the exit
   code to non-zero when there are unresolved references. Less disruptive
   but still emits potentially-incorrect JSON.

Option 1 is preferred because it prevents invalid output from being written.
The JSON output with unresolved bare name strings is semantically incorrect
and could cause downstream parsing failures.
