# Incomplete octal escape handling in `unescape_java`

## Symptom

`unescape_java` only handles single-digit octal escapes (e.g., `\0`
for null). Multi-digit octal escapes like `\012` (newline) or `\377`
(0xFF) are not fully parsed.

## Root cause

The octal escape branch in `unescape_java` reads only one digit
(`c2`) and doesn't consume up to 2 additional octal digits. The code
acknowledges this with a TODO comment.

## Location

- `src/reader.rs:1002-1021` — octal escape handling in `unescape_java`

## Expected behavior

Handle up to 3 octal digits following the `\`, matching Java's string
escape semantics. The value must be <= 0o377 (255).

## Difficulty

Moderate — needs proper lookahead on the char iterator. Consider
converting `chars()` to `Peekable` or collecting into a `Vec<char>`
with index-based access.
