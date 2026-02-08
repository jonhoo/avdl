# Doc comment tab-after-star not stripped

## Symptom

When a doc comment continuation line uses `*<TAB>text` instead of
`* text` (i.e., a tab character after the star instead of a space),
Rust preserves the tab in the extracted doc string while Java strips
it. This causes semantic differences in the `"doc"` field of the
output JSON.

For example:

```
/**
 * First line
 *	Tabbed line
 */
```

Rust produces `"First line\n\tTabbed line"` while Java produces
`"First line\nTabbed line"`.

## Root cause

`src/doc_comments.rs` uses `strip_prefix(' ')` to remove the
optional whitespace character after `*` on continuation lines. This
only matches ASCII space (U+0020), not tab (U+0009). Java's regex
uses `\h?` which matches any horizontal whitespace character,
including tabs.

## Affected files

- `src/doc_comments.rs`

## Reproduction

Regression test file:
`tests/testdata/regressions/doc-comment-leading-whitespace.avdl`

```sh
# Rust:
cargo run -- idl tests/testdata/regressions/doc-comment-leading-whitespace.avdl

# Compare doc field against Java:
java -jar avro-tools-1.12.1.jar idl tests/testdata/regressions/doc-comment-leading-whitespace.avdl
```

## Suggested fix

Replace `strip_prefix(' ')` with a function that strips one optional
horizontal whitespace character (space or tab). For example:

```rust
// Instead of:
line.strip_prefix(' ').unwrap_or(line)

// Use:
line.strip_prefix([' ', '\t']).unwrap_or(line)
```

`strip_prefix` with a char slice matches any single character from
the slice, which is exactly the semantics needed here.

## Spec reference

The Avro specification is silent on doc comment processing details.
Behavior is defined by the Java reference implementation.

## Source

Discovered during fuzz testing of 229 real-world `.avdl` files.
