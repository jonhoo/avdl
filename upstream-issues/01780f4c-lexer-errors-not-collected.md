# IdlReader does not install an error listener on the lexer

## Symptom

When the ANTLR lexer encounters an untokenizable character (e.g., an
embedded control character like `\x01`), the default
`ConsoleErrorListener` prints a "token recognition error" to stderr,
but the tool continues and may produce output. If the bad character
sits in a whitespace position (between tokens), the output is valid
and the tool exits 0 — the error on stderr is the only signal that
something was wrong.

## Root cause

In `IdlReader.parse()` (line 208-209), the custom error listener
(`SIMPLE_AVRO_ERROR_LISTENER`) is installed only on the **parser**:

    parser.removeErrorListeners();
    parser.addErrorListener(SIMPLE_AVRO_ERROR_LISTENER);

The **lexer** (line 202) retains its default `ConsoleErrorListener`,
which prints to stderr without throwing.

## Reproduction

```sh
# Create a file with a control character between two valid tokens:
printf 'protocol Test {\n  record Foo {\n    string\x01 name;\n  }\n}\n' > /tmp/ctrl.avdl

# Java prints a lexer error to stderr but produces valid output:
java -jar avro-tools-1.12.1.jar idl /tmp/ctrl.avdl
# stderr: line 3:10 token recognition error at: ''
# stdout: valid JSON
# exit code: 0
```

## Suggested fix

Install the error listener on the lexer as well:

    IdlLexer lexer = new IdlLexer(charStream);
    lexer.removeErrorListeners();
    lexer.addErrorListener(SIMPLE_AVRO_ERROR_LISTENER);

This would make lexer errors fatal, matching the parser error handling.
Alternatively, a less disruptive fix would be to collect lexer errors
and surface them as warnings through the existing warning mechanism
(similar to the out-of-place doc comment warnings).
