# Missing record `}` before another record produces confusing error

## Symptom

When a record's closing `}` is missing and another `record` keyword
follows inside the same protocol, the error says:

```
unexpected '{' expected ';' or ','
```

pointing at the opening `{` of the second record. This doesn't hint
at the actual problem: the first record is never closed.

```avdl
protocol Test {
  record Foo {
    string name;
    int age;

  record Bar {    // <-- error points here: "unexpected '{'"
    string value;
  }
}
```

The unclosed-brace detection (`detect_unclosed_brace`) only fires when
the error is "unexpected end of file" at EOF. In this case, the
protocol `}` serves as the closing brace for `Foo`, so the brace
count balances and the error manifests mid-file as a confusing
"expected ';' or ','".

## Root cause

ANTLR's error recovery interprets `record Bar` as a field declaration
(type `record`, name `Bar`) inside `Foo`. When it encounters `{`
instead of `;`, it reports "expected ';' or ','". The existing
`refine_errors_with_source` patterns don't detect this mid-file
unclosed-brace scenario.

## Affected files

- `src/reader.rs` -- `refine_errors_with_source` and related functions

## Reproduction

```sh
cat > tmp/missing-record-brace.avdl <<'EOF'
protocol Test {
  record Foo {
    string name;
    int age;

  record Bar {
    string value;
  }
}
EOF
cargo run -- idl tmp/missing-record-brace.avdl
```

## Suggested fix

Add a new detection pattern in `refine_errors_with_source`: when the
error is "expected ';' or ','" and the offending token is `{`, look
backwards in the source to see if a `record`/`enum`/`error` keyword
appears before the `{`. If so, check whether there is an unclosed
brace by counting braces from the start of the file up to this point.
If the brace stack is non-empty, suggest the missing `}` for the
innermost unclosed construct, similar to how `detect_unclosed_brace`
works for EOF errors.
