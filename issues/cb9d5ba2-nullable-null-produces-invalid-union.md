# `null?` type accepted, producing invalid `[null, null]` union

## Symptom

The Rust tool accepts `null?` as a field type and produces a union
containing two `null` members: `["null", "null"]`. This is invalid
per the Avro specification, which requires each type in a union to
be unique. Java avro-tools rejects this input with a
`NoSuchElementException` (its way of signaling semantic errors from
ANTLR error recovery).

The same bug manifests wherever `null?` appears in a type position:
top-level field types, array element types (`array<null?>`), map
value types (`map<null?>`), and union members.

## Root cause

In `walk_nullable_type`, when the base type is a primitive `null`
(token type `Idl_Null`) and the `?` suffix is present, the code
unconditionally wraps it in `Union { types: [Null, base_type] }`.
Since `base_type` is already `Null`, this creates `[Null, Null]`.

There is no check that the base type is not `null` before applying
the nullable transformation. The Java reference implementation
rejects this at the semantic level (its listener crashes on the
invalid input, which turns into a parse error).

## Affected files

- `src/reader.rs` — `walk_nullable_type` (around line 1611)

## Reproduction

```sh
# Write test file
cat > tmp/test-null-nullable.avdl <<'EOF'
protocol TestNullNullable {
  record Foo {
    null? value;
  }
}
EOF

# Rust accepts, producing invalid union:
cargo run -- idl tmp/test-null-nullable.avdl
# Output includes: "type": ["null", "null"]

# Java rejects:
java -jar avro-tools-1.12.1.jar idl tmp/test-null-nullable.avdl
# Exception: NoSuchElementException

# Also affects array element types:
cat > tmp/test-array-null-nullable.avdl <<'EOF'
protocol Test {
  record Foo {
    array<null?> values;
  }
}
EOF
cargo run -- idl tmp/test-array-null-nullable.avdl
# Output includes: "items": ["null", "null"]
```

## Suggested fix

In `walk_nullable_type`, after determining the base type and before
wrapping in a nullable union, check whether `base_type` is
`AvroSchema::Null`. If it is, return an error:

```rust
if ctx.optional.is_some() {
    if matches!(base_type, AvroSchema::Null) {
        return Err(make_diagnostic(
            src,
            ctx,
            "`null?` is not allowed: applying `?` to `null` \
             would produce the invalid union `[null, null]`",
        ));
    }
    Ok(AvroSchema::Union {
        types: vec![AvroSchema::Null, base_type],
        is_nullable_type: true,
    })
} else {
    Ok(base_type)
}
```
