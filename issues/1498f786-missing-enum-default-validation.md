# Missing validation of enum default symbol against symbol list

## Symptom

The Rust tool accepts an enum default symbol that does not exist in the
enum's symbol list, producing invalid Avro JSON output. Java rejects
this with a `SchemaParseException`.

## Root cause

Java's `Schema.createEnum()` delegates to the `EnumSchema` constructor,
which validates at line 1100-1102 of `Schema.java`:

```java
if (enumDefault != null && !symbols.contains(enumDefault)) {
    throw new SchemaParseException(
        "The Enum Default: " + enumDefault + " is not in the enum symbol set: " + symbols);
}
```

The Rust `walk_enum` in `reader.rs` collects the default symbol from the
parse tree but does not check it against the symbols list before
building the `AvroSchema::Enum`.

## Affected files

- `src/reader.rs` -- `walk_enum` function (around line 954-958)

## Reproduction

```sh
cat > tmp/test-enum-default-invalid.avdl <<'EOF'
@namespace("test")
protocol P {
  enum E {
    A, B, C
  } = NONEXISTENT;
}
EOF
cargo run -- idl tmp/test-enum-default-invalid.avdl
# Produces JSON with "default": "NONEXISTENT" instead of an error
```

Expected: error similar to Java's
`"The Enum Default: NONEXISTENT is not in the enum symbol set: [A, B, C]"`

Actual: silently produces `{"default": "NONEXISTENT"}` in the output.

## Suggested fix

After collecting the `default_symbol` in `walk_enum`, validate that it
exists in the `symbols` list:

```rust
if let Some(ref default) = default_symbol {
    if !symbols.contains(default) {
        return Err(make_diagnostic(
            src,
            &*default_ctx,
            format!(
                "Enum default '{}' is not in the symbol set: {:?}",
                default, symbols
            ),
        ));
    }
}
```

Priority: **Medium**. Invalid enum defaults produce semantically
incorrect JSON that downstream consumers may reject. This differs from
the `fixDefaultValue` issue (which is a no-op in JSON) because it
produces genuinely invalid output.
