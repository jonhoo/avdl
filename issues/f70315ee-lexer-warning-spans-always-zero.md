# Lexer error warning source spans are always (offset=0, length=0)

## Symptom

When the ANTLR lexer encounters an unrecognized character (e.g., a
control character like `\x01`), the resulting `Warning` has a
`SourceSpan` with `offset: 0` and `length: 0`, regardless of where
the error actually occurred in the source text.

The warning *message* contains the correct line and column
(`"line 1:35 token recognition error at: ..."`), but the miette
`SourceSpan` field does not reflect this position.

This is visible in the snapshot test for `lexer_error_produces_warning`:

    SourceSpan {
        offset: SourceOffset(0),
        length: 0,
    }

If the `Warning` type's `miette::Diagnostic` impl were ever used to
render rich diagnostics with source underlining, it would underline
position 0 instead of the actual error location.

## Root cause

In `CollectingErrorListener::syntax_error()` (reader.rs ~line 342),
byte offsets are extracted from `offending_symbol` via `get_start()`
/ `get_stop()`. For lexer errors, ANTLR passes `None` as the
offending symbol (the lexer has no token to report), so the code
falls back to `(0, 0)`:

```rust
let (offset, length) = offending_symbol
    .map(|tok| { ... })
    .unwrap_or((0, 0));  // <-- always hits this for lexer errors
```

The `line` and `column` parameters ARE available and correct, but
they are only used for the message text, not for computing the byte
offset. The ANTLR lexer also has `token_start_char_index` which
holds the exact byte offset, but the `syntax_error` callback API
does not pass it directly.

## Affected files

- `src/reader.rs` — `CollectingErrorListener::syntax_error()` (~line
  332) and the lexer error to `Warning` conversion (~line 516)

## Reproduction

```rust
// In a unit test (src/reader.rs):
#[test]
fn lexer_error_produces_warning() {
    let idl = "protocol Test { record Foo { string\x01 name; } }";
    let (_, _, warnings) = parse_idl_for_test(idl)
        .expect("lexer errors should not be fatal");
    assert_eq!(warnings.len(), 1);
    // The span is (0, 0) but should be (~35, 1):
    assert_eq!(warnings[0].span, Some(SourceSpan::new(35.into(), 1)));
    // ^ This would fail today
}
```

## Suggested fix

Option A (simplest): Compute the byte offset from `line` and
`column` by scanning the source text. The `_recognizer` parameter
in `syntax_error` is the lexer itself but is typed as a generic
`&T: Recognizer`, which doesn't expose `token_start_char_index`.
However, since the lexer error `Warning` already receives the full
source text (via `SourceInfo`), a helper function could convert
(line, column) to a byte offset:

```rust
fn line_col_to_byte_offset(source: &str, line: isize, column: isize) -> usize {
    let mut offset = 0;
    for (i, src_line) in source.lines().enumerate() {
        if i as isize + 1 == line {
            return offset + column as usize;
        }
        offset += src_line.len() + 1; // +1 for newline
    }
    0
}
```

Then use this in the `unwrap_or` fallback when `offending_symbol` is
`None`.

Option B: Parse the line/column from the error message string
(format: `"line L:C ..."`). Less robust but avoids needing the
source text at error collection time.
