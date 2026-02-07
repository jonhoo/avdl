# ANTLR parse errors do not cause the tool to fail

## Symptom

When the ANTLR parser encounters syntax errors in `.avdl` input, the
Rust tool prints the ANTLR error message to stderr but continues
processing via ANTLR's error recovery mechanism. It produces output
and exits with code 0, making it appear that the parse succeeded.

Java's `IdlReader` installs a custom `BaseErrorListener` (line 117)
that throws `SchemaParseException` on syntax errors, causing the tool
to fail immediately.

## Root cause

The Rust tool does not set up a custom error listener on the ANTLR
parser. The default ANTLR4 error listener (`ConsoleErrorListener`)
prints errors to stderr but does not fail or record the error. The
`parse_idl` function in `reader.rs` does not check for ANTLR parse
errors after calling `parser.idlFile()`.

Java's equivalent code (IdlReader.java lines 115-118) installs:

```java
parser.removeErrorListeners();
parser.addErrorListener(new BaseErrorListener() {
  @Override
  public void syntaxError(...) {
    throw new SchemaParseException("line " + line + ":" + col + " " + msg);
  }
});
```

This means ANY ANTLR syntax error immediately terminates parsing in
Java, while Rust silently recovers and may produce incorrect output.

## Impact on output

1. **Missing semicolons**: `record R { string name }` -- ANTLR
   recovers and produces a correct-looking record. Rust exits 0, Java
   exits 1.

2. **Schema after named types**: `record R {...} schema R?;` -- ANTLR
   recovery skips the `schema` declaration. Rust produces a
   `NamedSchemasFile` with just the record (ignoring the `schema`
   keyword). Java exits 1.

3. **Nullable array/map**: `array<string>?` -- ANTLR skips the `?`
   token. Rust produces a non-nullable array (semantically wrong
   output). Java exits 1 (or crashes in 1.12.1).

Issue `a4c8fe26` covers case 3 specifically, but the underlying cause
is the same: ANTLR error recovery is not treated as fatal.

## Affected files

- `src/reader.rs` -- `parse_idl` function, around the
  `parser.idlFile()` call

## Reproduction

```sh
# Missing semicolon -- Rust succeeds, Java fails:
cat > tmp/missing-semi.avdl << 'EOF'
@namespace("test")
protocol P {
    record R { string name }
}
EOF

cargo run -- idl tmp/missing-semi.avdl
# Prints error to stderr, but produces output and exits 0

java -jar ../avro-tools-1.12.1.jar idl tmp/missing-semi.avdl
# Exits 1 with SchemaParseException

# Schema after named types -- Rust ignores schema keyword:
cat > tmp/schema-after.avdl << 'EOF'
namespace test;
record Inner { string name; }
schema Inner?;
EOF

cargo run -- idl tmp/schema-after.avdl
# Produces NamedSchemasFile (ignores schema), exits 0

java -jar ../avro-tools-1.12.1.jar idl tmp/schema-after.avdl
# Exits 1 with parse error
```

## Suggested fix

Install a custom error listener on the ANTLR parser that collects
syntax errors. After `parser.idlFile()` completes, check if any
errors were recorded and return an `Err` if so.

The antlr4rust runtime supports custom error listeners via
`parser.remove_error_listeners()` and `parser.add_error_listener()`.
The listener should store errors (with line/column/message) in a
`Vec` or similar structure that can be checked after parsing.

Alternatively, if `antlr4rust`'s error listener API is awkward, count
ANTLR error messages by intercepting stderr output or by checking the
parser's internal error state after `idlFile()` returns.

## Priority

High. This affects all syntax validation. Users of the Rust tool may
not notice syntax errors in their `.avdl` files because the tool
exits successfully. Automation scripts that check exit codes will
not catch these errors.
