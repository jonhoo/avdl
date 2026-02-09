# Default value validation skipped for Reference types

## Symptom

Rust accepts invalid default values for fields whose type is a named
type that appears as an `AvroSchema::Reference` at validation time.
The divergence is specifically about **record-typed fields** where Java
rejects non-object defaults but Rust accepts them.

Examples that Rust incorrectly accepts (Java rejects all four):

```avdl
@namespace("org.apache.avro.test")
protocol Simple {
  record Inner { string name; }
  record Outer {
    Inner inner = "not a record";   // string default for record type
    Inner inner2 = 42;              // int default for record type
    Inner inner3 = [1, 2, 3];      // array default for record type
    Inner inner4 = null;            // null default for non-nullable record
  }
}
```

All of these produce valid-looking JSON output with the wrong default
value embedded, which would cause errors at deserialization time.

Java rejects all four cases (though with unhelpful `NoSuchElementException`
stack traces rather than clean error messages).

**Note:** For enum and fixed field defaults, both tools are lenient at
the IDL stage. Both Rust and Java accept `Status status = "X"` even
when "X" is not a valid enum symbol. This is consistent behavior (the
validation only happens at the enum declaration level with `} = X;`
syntax, not at the field level). The divergence is specifically about
record defaults where Java validates that the default is a JSON object
but Rust does not.

## Root cause

In `src/model/schema.rs`, `is_valid_default` has:

```rust
AvroSchema::Reference { .. } => true,
```

This intentionally skips validation for forward references because the
referenced type is not yet resolved. However, in practice, most named
type references remain as `Reference` at the time `validate_default`
is called in `walk_field` (`src/reader.rs` around line 1124), so
validation is effectively skipped for all named types.

The `Record`, `Enum`, and `Fixed` match arms do have correct
validation logic (e.g., `Record => value.is_object()`), but these
arms are never reached for named type references.

## Affected files

- `src/model/schema.rs` (`is_valid_default` function, line ~406)
- `src/reader.rs` (`walk_field` function, line ~1124)

## Reproduction

```sh
# Write a test file with a string default for a record-typed field:
cat > tmp/record-bad-default.avdl <<'EOF'
@namespace("org.apache.avro.test")
protocol Simple {
  record Inner { string name; }
  record Outer { Inner inner = "not a record"; }
}
EOF

# Rust incorrectly accepts:
cargo run -- idl tmp/record-bad-default.avdl
# Produces output with: "default": "not a record"

# Java rejects:
java -jar ../avro-tools-1.12.1.jar idl tmp/record-bad-default.avdl
# Throws NoSuchElementException
```

## Suggested fix

When validating defaults for `Reference` types, look up the referenced
type in the `SchemaRegistry` (which is available in `walk_field`) and
validate against the resolved schema. This requires threading the
registry (or a lookup closure) through to `validate_default`.

Alternatively, defer default validation until after all references are
resolved (a post-parse validation pass). This would also catch the
edge case of forward-referenced types.

A minimal fix for the common case: at the `walk_field` call site in
`reader.rs`, check whether the `final_type` is a `Reference` and
attempt to resolve it from the registry before calling
`validate_default`. If resolution fails (true forward reference),
skip validation as currently done.
