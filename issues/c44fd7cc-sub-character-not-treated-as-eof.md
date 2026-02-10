# SUB character (U+001A) not treated as end-of-file marker

## Symptom

The ANTLR grammar's `idlFile` rule includes `('\u001a' .*?)? EOF` to
treat the ASCII SUB character (U+001A) as an end-of-file marker,
ignoring any trailing content. The Rust tool does not handle this,
causing a parse error on files that contain a SUB character followed
by trailing content.

```
$ printf 'protocol P { record R { int x; } }\x1a trailing garbage' > tmp/test.avdl
$ cargo run -- idl tmp/test.avdl
Error: line 1:36 no viable alternative at input 'trailing'
```

Expected: successful parse, ignoring content after `\x1a`.

## Root cause

The antlr4rust runtime may not match the `\u001a` literal in the
grammar rule correctly (as noted in the existing TODO comment at
`reader.rs:678`). The generated parser either doesn't recognize the
SUB character as the grammar intends, or the antlr4rust runtime
handles the predicate differently from the Java runtime.

## Affected files

- `src/reader.rs` (line 678, existing TODO comment)

## Reproduction

```sh
printf 'protocol P { record R { int x; } }\x1a trailing garbage' > tmp/test-sub.avdl
cargo run -- idl tmp/test-sub.avdl
# Expected: success
# Actual: parse error about 'trailing'
```

## Suggested fix

Strip `\u001a` and everything after it from the input string in
`parse_idl_named` before passing it to the lexer, as the existing
TODO comment suggests. This is a straightforward pre-processing step:

```rust
let input = if let Some(pos) = input.find('\u{001a}') {
    &input[..pos]
} else {
    input
};
```

This matches what the grammar intends without relying on the antlr4rust
runtime's handling of the character.
