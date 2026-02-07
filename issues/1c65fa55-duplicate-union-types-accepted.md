# Duplicate types in union silently accepted

## Symptom

`union { null, string, null }` is accepted without error. Java rejects
with "Duplicate in union: null".

## Root cause

The Rust tool does not validate that union branches are unique. The Avro
specification requires that unions not contain more than one schema of
the same type (with exceptions for named types).

## Affected files

- `src/reader.rs` — union type construction (`walk_union_type` or
  similar)

## Reproduction

```sh
cat > tmp/dup-union.avdl <<'EOF'
protocol Test {
  record Foo {
    union { null, string, null } field1;
  }
}
EOF
cargo run -- idl tmp/dup-union.avdl
# Rust: succeeds, produces union with duplicate null
# Java: rejects with "Duplicate in union: null"
```

## Suggested fix

After collecting union branches, check for duplicates (by type for
anonymous types, by name for named types). Emit an error matching
Java's "Duplicate in union: <type>" message.

Low priority — duplicate union types are almost certainly a mistake in
the source `.avdl` file, and downstream Avro consumers will likely
reject the schema anyway.
