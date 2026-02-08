**`unescape_java` does not handle surrogate pairs.**

`\uD800\uDC00`-style surrogate pairs for U+10000+ characters are not
decoded. Each `\uXXXX` is processed independently, so lone surrogates
are rejected by `char::from_u32` and fall through to literal text.

- **Symptom**: supplementary Unicode characters (U+10000+) encoded as
  surrogate pairs in `.avdl` string literals are not properly decoded
- **Root cause**: `unescape_java` processes each `\uXXXX` escape
  independently without detecting high+low surrogate sequences
- **Affected files**: `src/reader.rs` (unescape_java function)
- **Reproduction**: create a string literal with `\uD83D\uDE00` (U+1F600
  GRINNING FACE) — will not produce the correct character
- **Suggested fix**: after parsing a `\uXXXX` in the D800-DBFF range,
  check if the next escape is a low surrogate (DC00-DFFF) and combine
  them into a single code point
- **Priority**: very low — supplementary Unicode in `.avdl` string
  literals is extremely rare
