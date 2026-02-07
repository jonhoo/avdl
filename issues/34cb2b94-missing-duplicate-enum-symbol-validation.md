# Missing validation: duplicate enum symbols

## Symptom

Enum declarations with duplicate symbols are accepted without error:

```avdl
@namespace("test")
protocol P {
    enum Color { RED, GREEN, BLUE, RED }
}
```

This produces JSON output with `"symbols": ["RED", "GREEN", "BLUE",
"RED"]`, which is semantically invalid according to the Avro
specification. Downstream Avro consumers would reject this schema.

## Root cause

The `walk_enum` function in `src/reader.rs` collects symbols from
`enumSymbol_all()` into a `Vec<String>` by iterating. There is no check
for duplicate symbol names.

The Java `Schema` constructor for enums (in `Schema.java` line 1097)
explicitly checks for duplicates and throws
`SchemaParseException("Duplicate enum symbol: X")`.

## Affected files

- `src/reader.rs` -- `walk_enum` function (around line 703-708)

## Reproduction

```sh
echo '@namespace("test") protocol P { enum Color { RED, GREEN, BLUE, RED } }' \
  | cargo run -- idl
```

Expected: error about duplicate enum symbol `RED`.
Actual: JSON output with duplicate `RED` in the symbols array.

## Suggested fix

After collecting all symbols in `walk_enum`, check for duplicates:

```rust
let mut seen_symbols = HashSet::new();
for (i, sym) in symbols.iter().enumerate() {
    if !seen_symbols.insert(sym.as_str()) {
        // Use the source span of the duplicate symbol's context for
        // a precise error location.
        return Err(make_diagnostic(
            src,
            &*ctx.enumSymbol_all()[i],
            format!("duplicate enum symbol: {sym}"),
        ));
    }
}
```

## Priority

Medium. This is a semantic correctness gap -- the tool accepts invalid
schemas that Java rejects. Duplicate symbols in Avro enums would cause
errors in downstream tooling (e.g., schema registration, code
generation).
