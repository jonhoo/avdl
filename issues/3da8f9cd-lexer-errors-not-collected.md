# Lexer errors printed to stderr but not treated as failures

## Symptom

When the ANTLR lexer encounters an unrecognizable character (e.g., a
control character like 0x01), it prints a "token recognition error" to
stderr but the tool still produces output and exits with status 0.
The error message appears on stderr interleaved with the JSON output
on stdout:

```
line 3:14 token recognition error at: '\x01'
{
  "messages": {},
  ...
}
```

This matches the Java avro-tools behavior, which also prints the lexer
error to stderr while still producing output. However, it's
inconsistent with the parser error handling, where all errors are
treated as fatal.

## Root cause

In `parse_idl_named`, the `CollectingErrorListener` is installed only
on the **parser** (lines 489-492), not on the **lexer**. The lexer
retains its default `ConsoleErrorListener`, which prints errors to
stderr without recording them.

Java's `IdlReader` has the same gap: the custom error listener is
installed on the parser only (lines 208-209 of `IdlReader.java`), so
lexer errors also print to stderr without causing failure.

## Affected files

- `src/reader.rs` -- `parse_idl_named`, around lines 474-492

## Reproduction

```sh
# Create a file with an embedded control character:
printf 'protocol Test {\n    record Foo {\n        string\x01 name;\n    }\n}\n' > tmp/control-char.avdl

# Rust: prints lexer error to stderr but succeeds:
cargo run -- idl tmp/control-char.avdl
# stderr: line 3:14 token recognition error at: ''
# stdout: valid JSON output
# exit code: 0

# Java: same behavior:
java -jar avro-tools-1.12.1.jar idl tmp/control-char.avdl
# stderr: line 3:14 token recognition error at: ''
# stdout: valid JSON output
# exit code: 0
```

## Suggested fix

Install the `CollectingErrorListener` on the lexer as well, so that
lexer errors are collected alongside parser errors and treated as
fatal. This would make the Rust tool **stricter** than Java, but
arguably more correct: if the lexer can't tokenize the input, the
output may be silently wrong.

```rust
// After creating the lexer:
let mut lexer = IdlLexer::new(input_stream);
let lexer_errors: Rc<RefCell<Vec<SyntaxError>>> = Rc::new(RefCell::new(Vec::new()));
lexer.remove_error_listeners();
lexer.add_error_listener(Box::new(CollectingErrorListener {
    errors: Rc::clone(&lexer_errors),
}));
let token_stream = CommonTokenStream::new(lexer);
```

Then check `lexer_errors` alongside `syntax_errors` after parsing.

**Trade-off**: Making this change would make the Rust tool stricter
than Java avro-tools on this edge case. The benefit is more robust
error detection; the risk is rejecting files that Java accepts (though
such files would need to contain characters that the ANTLR grammar
can't tokenize, which is a strong signal of corruption or encoding
issues).

A middle ground would be to surface lexer errors as **warnings** on
stderr (which we already do inadvertently via the default listener)
rather than as fatal errors, matching Java's behavior.
