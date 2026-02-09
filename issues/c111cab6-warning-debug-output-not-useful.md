# Warning Debug output is not useful for diagnostics

## Symptom

When a `Warning` is printed with `{:?}` (Rust `Debug` format), the
output is deeply nested and hard to read:

```
Warning {
    message: "line 1:35 token recognition error at: '\u{1}'",
    source: Some(
        NamedSource {
            name: "<input>",
            source: "<redacted>",
            language: None,
        ,
    ),
    span: Some(
        SourceSpan {
            offset: SourceOffset(
                0,
            ),
            length: 0,
        },
    ),
}
```

Problems:
1. `NamedSource` debug-prints with `source: "<redacted>"` (miette
   intentionally hides the source content in Debug), so you cannot
   see what source text the span refers to.
2. `SourceSpan` is printed as nested structs (`SourceOffset(0)`)
   rather than a compact `0..0` or `offset=0, len=0` form.
3. `NamedSource` has a cosmetic issue: the closing `}` for the inner
   struct appears to be missing before the `,` and closing `)`.
4. The overall format is ~20 lines for a single warning, making it
   impractical for scanning multiple warnings.

The `lexer_error_produces_warning` snapshot test uses
`insta::assert_debug_snapshot!(warnings[0])` which captures this
verbose format.

## Root cause

`Warning` uses `#[derive(Debug)]` which produces the default Rust
debug format. The fields `source` (`miette::NamedSource<String>`)
and `span` (`miette::SourceSpan`) have their own Debug impls from
the miette crate, which are designed for internal use rather than
user-facing diagnostics.

## Affected files

- `src/reader.rs` — `Warning` struct (line 59), `#[derive(Debug)]`
- `src/snapshots/avdl__reader__tests__lexer_error_produces_warning.snap`

## Reproduction

```rust
let idl = "protocol Test { record Foo { string\x01 name; } }";
let (_, _, warnings) = parse_idl_for_test(idl).unwrap();
println!("{:#?}", warnings[0]);
// Produces the verbose output shown above
```

## Suggested fix

Implement a custom `Debug` for `Warning` that shows a compact,
informative representation:

```rust
impl fmt::Debug for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Warning")
            .field("message", &self.message)
            .field("file", &self.source.as_ref().map(|s| s.name()))
            .field("span", &self.span.map(|s| {
                format!("{}..{}", s.offset(), s.offset() + s.len())
            }))
            .finish()
    }
}
```

This would produce:
```
Warning {
    message: "line 1:35 token recognition error at: '\\u{1}'",
    file: Some("<input>"),
    span: Some("0..0"),
}
```

Much more compact and informative. The snapshot test would need to be
updated to match.

Alternatively, if the intent is that `Debug` printing should render
like a miette diagnostic with source context, implement it using
miette's `GraphicalReportHandler` in the `Debug` impl. But this is
unconventional for `Debug` and may be better served by a separate
`render()` method.
