# Missing reserved property name validation for `@` annotations

## Symptom

The Rust tool silently accepts `@doc("...")`, `@type("...")`, and
other reserved property names as custom annotations. Java rejects
these with "Can't set reserved property: doc" (or similar).

Example:

```avdl
@namespace("test")
@doc("Protocol doc via annotation")
protocol P {
  @doc("Record doc")
  record R {
    @doc("Field doc") string name;
  }
}
```

- **Java**: Throws `AvroRuntimeException: Can't set reserved property: doc`
- **Rust**: Silently accepts, producing output with `"doc"` as a custom
  property on the schema. For the field case (`@doc("...") string name`),
  this goes through `walk_full_type` and becomes a property on the type
  schema (`{"type": "string", "doc": "Field doc"}`), which is semantically
  wrong.

## Root cause

Java's `JsonProperties.addProp()` (line 288) checks each property name
against a set of reserved names before setting it, and throws if the name
is reserved. The reserved sets are defined per context:

- **Schema**: `name`, `type`, `doc`, `fields`, `items`, `values`,
  `symbols`, `namespace`, `size`, `logicalType`, `aliases`
- **Enum**: all Schema reserved + `default`
- **Field**: `name`, `type`, `doc`, `default`, `aliases`
- **Protocol**: `namespace`, `protocol`, `doc`, `messages`, `types`,
  `version`
- **Message**: `doc`, `response`, `request`, `errors`, `one-way`

The Rust tool has no equivalent validation. Most reserved names are
handled by special-cased annotation processing (e.g., `@namespace` is
intercepted in `walk_schema_properties`), but `@doc` is not intercepted
because doc comes from `/** ... */` comments, not annotations. When
`@doc` appears as an annotation, it falls through to the generic
custom-property path.

## Affected files

- `src/reader.rs` -- `walk_schema_properties` (generic property handling)
- `src/model/json.rs` -- serialization may output reserved names as
  custom properties

## Reproduction

```sh
# Create test file:
cat > tmp/reserved-test.avdl << 'EOF'
@namespace("test")
@doc("test doc")
protocol P {
  record R { string name; }
}
EOF

# Rust succeeds with @doc as a protocol property:
cargo run -- idl tmp/reserved-test.avdl
# Output includes "doc": "test doc" as if it were a doc comment

# Java rejects it:
java -jar ../avro-tools-1.12.1.jar idl tmp/reserved-test.avdl
# Error: Can't set reserved property: doc
```

## Suggested fix

Add validation in `walk_schema_properties` to reject annotations whose
names match reserved property names for the current context. When a
reserved name is used as an `@` annotation, produce an error like
Java's "Can't set reserved property: {name}".

The simplest approach is to add a check in the generic `_` match arm
of `walk_schema_properties`:

```rust
const SCHEMA_RESERVED: &[&str] = &[
    "name", "type", "doc", "fields", "items", "values",
    "symbols", "namespace", "size", "logicalType", "aliases",
];

// In the _ arm:
if SCHEMA_RESERVED.contains(&name.as_str()) {
    return Err(make_diagnostic(
        src, &**prop,
        format!("Can't set reserved property: {name}"),
    ));
}
result.properties.insert(name, value);
```

However, the reserved set varies by context, so a more nuanced
approach using the context flags from issue `98a5d266` would be
better.

## Priority

Low-Medium. This mostly matters for error reporting (rejecting invalid
input that Java would reject). The one case with data impact is
`@doc("...")` on a field type (before the type), which produces
semantically wrong output: a `doc` property inside the type schema
object rather than as a field doc or an error.
