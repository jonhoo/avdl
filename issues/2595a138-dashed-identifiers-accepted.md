# Dashed identifiers accepted but invalid per Avro spec

## Symptom

The Rust tool accepts dashed identifiers like `my-record` as type
names, producing schemas that may not round-trip through other Avro
implementations. Java's `validateName` rejects them.

## Root cause

The ANTLR grammar's `IdentifierToken` allows `[.-]` between identifier
parts, so `my-record` is syntactically valid. But the Avro spec requires
names to match `[A-Za-z_][A-Za-z0-9_]*`, and Java enforces this via the
`VALID_NAME` regex (`[_\p{L}][_\p{LD}]*`) plus per-segment namespace
validation in `validateName`.

The Rust tool has `INVALID_TYPE_NAMES` (preventing reserved words like
`int` via backtick escapes) but lacks the `VALID_NAME` regex check.

## Affected files

- `src/reader.rs` — where type names are accepted
- `src/resolve.rs` — `SchemaRegistry` name validation

## Reproduction

```sh
cat > tmp/dashed.avdl <<'EOF'
protocol Test {
  record my-record {
    string name;
  }
}
EOF
cargo run -- idl tmp/dashed.avdl
# Rust: succeeds, produces JSON with "my-record" as record name
# Java: rejects with validation error
```

## Suggested fix

Add a `VALID_NAME` regex check (matching Java's `[_\p{L}][_\p{LD}]*`)
when registering type names in the `SchemaRegistry`. Reject names that
don't match with a clear error message. This is separate from the
existing `INVALID_TYPE_NAMES` reserved-word check.

Low priority — dashed identifiers in `.avdl` files are rare in practice.
