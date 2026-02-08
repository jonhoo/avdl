# Alias names not validated as valid Avro names

## Symptom

The Rust tool accepts invalid alias names (such as names starting with
digits or containing dashes) in `@aliases` annotations without any
error. Java validates alias names through the `Name` constructor when
`addAlias` is called, rejecting invalid names.

## Root cause

Java's `NamedSchema.addAlias(String name, String space)` creates a
`new Name(name, space)`, and the `Name` constructor validates the name
against the `VALID_NAME` pattern. This rejects aliases like `"123bad"`
or `"my-alias"`.

The Rust `walk_schema_properties` function in `reader.rs` parses
`@aliases` values into a `Vec<String>` but only validates that each
element is a string -- it does not check whether those strings are
valid Avro names.

Similarly, field aliases (parsed in `VARIABLE_PROPS` context) are not
validated.

## Affected files

- `src/reader.rs` -- `walk_schema_properties` alias parsing (around
  lines 498-519)

## Reproduction

```sh
cat > tmp/test-alias-validation.avdl <<'EOF'
@namespace("test")
protocol P {
  @aliases(["123bad", "my-alias", "good"])
  record Foo {
    string name;
  }
}
EOF
cargo run -- idl tmp/test-alias-validation.avdl
# Produces JSON with aliases: ["123bad", "my-alias", "good"]
# Java would reject "123bad" as an invalid name
```

## Suggested fix

After extracting alias strings, validate each one using
`is_valid_avro_name` from `resolve.rs`, or replicate the check inline:

```rust
for alias in &aliases {
    let simple_name = alias.rsplit('.').next().unwrap_or(alias);
    if !is_valid_avro_name(simple_name) {
        return Err(make_diagnostic(
            src,
            &**prop,
            format!("invalid alias name: {alias}"),
        ));
    }
}
```

Note: aliases can be fully-qualified (e.g., `"com.example.OldName"`),
so each dot-separated segment should be validated independently, not
just the final segment.

Priority: **Low**. Invalid alias names in practice are extremely rare
and typically indicate a typo. The JSON output is technically valid
even with invalid alias names, but downstream Avro consumers may
reject it.
