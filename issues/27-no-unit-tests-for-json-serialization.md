# No unit tests for `model/json.rs` serialization functions

## Summary

The `src/model/json.rs` module contains all JSON serialization logic
for converting Avro schemas and protocols to JSON. It has zero unit
tests. All testing happens indirectly through the integration tests,
which compare full protocol/schema JSON output against golden files.

The Java codebase tests JSON value serialization in isolation through
`IdlUtilsTest`, which verifies that individual values (null, maps,
collections, bytes, strings, enums, doubles, floats, longs, integers,
booleans) serialize correctly, and that unknown types produce errors.

## What unit tests would catch

Integration tests are effective at catching end-to-end regressions
but are poor at isolating which serialization function is broken. For
example, if the `protocol_to_json` output is wrong, the developer
must debug through the entire pipeline to find whether the bug is in:

- `record_to_json` (wrong field ordering, missing properties)
- `schema_to_json` (wrong reference resolution, missing first-use
  inlining)
- `schema_ref_name` (wrong namespace shortening)
- `field_to_json` (wrong default value serialization)
- `message_to_json` (wrong request/response/error serialization)
- `property_to_json` (wrong annotation serialization)

Unit tests for each function would immediately localize the failure.

## Recommended unit tests

### Value serialization

Test that `schema_to_json` correctly serializes:

- Primitive types: `"null"`, `"int"`, `"string"`, etc. as bare
  strings
- Annotated primitives: `{"type": "int", "foo": "bar"}` as objects
- Arrays: `{"type": "array", "items": "string"}`
- Maps: `{"type": "map", "values": "int"}`
- Unions: `["null", "string"]`
- Named types (first occurrence): full inline definition
- Named types (subsequent occurrence): bare string name
- References resolved from lookup table

### Namespace shortening

Test that `schema_ref_name` correctly:

- Returns simple name when namespace matches enclosing namespace
- Returns fully-qualified name when namespaces differ
- Returns simple name when no enclosing namespace is set

### Field serialization

Test that `field_to_json` correctly serializes:

- Fields with defaults (including `null`, `NaN`, `Infinity`,
  `"-Infinity"`)
- Fields with `@order` annotations
- Fields with `@aliases` annotations
- Fields with doc comments
- Fields with custom properties

### Protocol serialization

Test that `protocol_to_json` includes:

- `protocol` key with name
- `namespace` key
- `types` array in correct order
- `messages` object with correct structure

### Edge cases from Java `IdlUtilsTest`

The Java test verifies these specific serialization edge cases:

- `JsonProperties.NULL_VALUE` serializes as `"null"`
- Maps serialize as JSON objects with correct key-value pairs
- Collections serialize as JSON arrays
- Byte arrays serialize as quoted strings
- Floats and doubles serialize with full precision
- Longs serialize without loss of precision
- Unknown/unsupported types produce errors

Our equivalent would be testing the JSON value serialization in
`walk_json_value` / `walk_json_literal` (in `reader.rs`) and the
default value serialization in `field_to_json` (in `json.rs`).

## Priority

Medium. The integration tests provide broad coverage, so this is
about improving debuggability and preventing subtle serialization
bugs that could slip through whole-file comparisons.
