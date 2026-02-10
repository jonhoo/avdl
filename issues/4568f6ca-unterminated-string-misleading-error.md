# Unterminated string literal produces misleading error about unexpected token

## Symptom

When a string literal is missing its closing quote, the error message
points to a token further downstream and says it is unexpected, with
no indication that the actual problem is an unterminated string.

For example:

```avdl
@namespace("org.test")
protocol Test {
  record Foo {
    string name = "unterminated;
  }
}
```

Produces:

```
Error:   x parse IDL source
  |-> line 5:2 unexpected token `}`
   ,----[tmp/mut28-unterminated-string.avdl:5:3]
 4 |     string name = "unterminated;
 5 |   }
   .   +
   .   `-- unexpected `}`
 6 | }
   `----
  help: expected one of: null, true, false, {, [, StringLiteral,
        IntegerLiteral, FloatingPointLiteral
```

The error points to line 5, column 2 (the `}`), but the real problem
is the unterminated string on line 4. The help text mentions
`StringLiteral` as expected but does not suggest that the string on
the previous line may be unclosed.

## Root cause

The ANTLR lexer recovers from the unterminated string by consuming
everything from the opening `"` to the end of the line as an
unrecognized token (or by treating the rest of the line as part of an
error recovery). The parser then sees the next valid token (`}`) and
reports that as unexpected. The error simplification logic in
`simplify_large_expecting_set` does not detect this specific failure
pattern.

## Affected files

- `src/reader.rs` -- error simplification / enrichment logic

## Reproduction

```sh
cat > tmp/unterm-str.avdl <<'EOF'
@namespace("org.test")
protocol Test {
  record Foo {
    string name = "unterminated;
  }
}
EOF
cargo run -- idl tmp/unterm-str.avdl 2>&1
```

## Suggested fix

When the parser error expects a `StringLiteral` and the preceding
source line contains an odd number of unescaped `"` characters, add a
hint like "possible unterminated string literal on line N" to the help
text. Alternatively, install a custom ANTLR error strategy that
detects unterminated string tokens during lexing and produces a
targeted error message.
