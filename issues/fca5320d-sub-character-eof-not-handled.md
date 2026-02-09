# SUB character (U+001A) EOF marker not handled

## Symptom

The ANTLR grammar's `idlFile` rule includes `('\u001a' .*?)? EOF` to
treat the ASCII SUB character as an end-of-file marker, ignoring any
trailing content. Java handles this correctly, but the Rust port fails
with a parse error when a SUB character appears after valid IDL content.

## Root cause

The antlr4rust runtime may not correctly handle the `\u001a` literal
in the grammar rule. The generated parser likely does not match the
SUB character, causing ANTLR to try to parse the trailing content as
part of the IDL file.

## Affected files

- `src/generated/idlparser.rs` (generated parser)
- `avro/share/idl_grammar/org/apache/avro/idl/Idl.g4` (grammar rule at line 44)

## Reproduction

```sh
printf 'protocol Test {\n  record Foo { string name; }\n}\x1a trailing garbage' > tmp/test-sub-eof.avdl
cargo run -- idl tmp/test-sub-eof.avdl
# Error: line 3:3 no viable alternative at input 'trailing'

java -jar ../avro-tools-1.12.1.jar idl tmp/test-sub-eof.avdl
# Succeeds, outputs valid JSON
```

## Suggested fix

This is likely a low-priority issue since modern files rarely use the
SUB character. If needed, a workaround could strip `\u001a` and
everything after it from the input string before passing it to the
ANTLR lexer. Alternatively, investigate whether the antlr4rust code
generator handles Unicode escapes in parser rules correctly.
