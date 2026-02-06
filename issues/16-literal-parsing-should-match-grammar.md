# Literal parsing should match grammar, not Java builtins

## Symptom

String and numeric literal parsing (e.g., `unescape_java`) currently
tries to handle the full set of Java escape sequences. However, the
ANTLR grammar (`Idl.g4`) defines the exact set of legal literal
syntaxes — we only need to accept what the grammar permits, not
everything that Java's `String.decode()` or similar builtins support.

## Root cause

The literal parsing was ported from Java code that uses Java's own
string/number decoding utilities, which accept a broader syntax than
what the grammar actually generates.

## Location

- `src/reader.rs` — `unescape_java` and any numeric literal parsing
- `avro/share/idl_grammar/org/apache/avro/idl/Idl.g4` — the
  authoritative source for what literal syntax is legal

## Expected behavior

Review the `Idl.g4` grammar rules for string literals (`StringLiteral`),
integer literals (`IntegerLiteral`), float literals
(`FloatingPointLiteral`), and escape sequences. Simplify the Rust
parsing code to handle exactly what the grammar permits, rather than
trying to be a general-purpose Java literal decoder.

This may simplify issue #05 (octal escapes) — if the grammar doesn't
allow multi-digit octal escapes, we don't need to support them.

## Difficulty

Medium — requires reading the grammar rules and simplifying the
parsing code to match.
