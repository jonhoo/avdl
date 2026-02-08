# Doc comment CRLF normalization (low priority)

## Symptom

When source files use `\r\n` (CRLF) line endings, Rust normalizes
doc comment line endings to `\n` (LF), while Java preserves the
original `\r\n` endings. This produces a semantic difference in the
`"doc"` field of the output JSON.

For example, a doc comment in a CRLF file:

```
/**\r\n * First line\r\n * Second line\r\n */
```

Rust produces `"First line\nSecond line"` while Java produces
`"First line\r\nSecond line"`.

## Root cause

`src/doc_comments.rs` uses `str::lines()` to split doc comment text
into lines, then joins them back with `join("\n")`. Rust's
`str::lines()` treats both `\n` and `\r\n` as line terminators and
strips the terminator, so the original line ending style is lost.

## Affected files

- `src/doc_comments.rs`

## Reproduction

Regression test file:
`tests/testdata/regressions/doc-comment-crlf-preservation.avdl`
(this file uses CRLF endings, protected by `.gitattributes`)

```sh
# Rust:
cargo run -- idl tests/testdata/regressions/doc-comment-crlf-preservation.avdl

# Compare doc field against Java:
java -jar avro-tools-1.12.1.jar idl tests/testdata/regressions/doc-comment-crlf-preservation.avdl
```

## Suggested fix

Use a custom line splitter that preserves original line endings, or
use a regex-based split that retains the line terminator style and
re-joins with the original separator.

However, Rust's normalization to `\n` is arguably **better** behavior
than Java's preservation of `\r\n`, since it produces consistent
output regardless of the source file's line ending convention. Most
downstream JSON consumers will treat `\n` and `\r\n` identically.

## Priority

**Low.** Rust's behavior (normalizing to `\n`) is more principled
than Java's (preserving platform-dependent line endings). This only
matters for strict byte-for-byte compatibility with Java output,
which is explicitly a non-goal of this project. Fix only if a
real-world consumer depends on CRLF preservation in doc strings.

## Spec reference

The Avro specification is silent on line ending handling in doc
comments. Java preserves original line endings as an implementation
artifact, not by design.

## Source

Discovered during fuzz testing of 229 real-world `.avdl` files.
