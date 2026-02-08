# Missing default value type validation

## Symptom

The Rust tool accepts field default values that do not match the
field's Avro type, producing protocol/schema JSON that is invalid
according to the Avro specification. The Java tool (via
`Schema.Field` constructor with `validate=true`) rejects these
mismatches at compile time.

Examples of invalid defaults that Rust silently accepts:

- `int count = "not_a_number";` -- string default for int field
- `boolean flag = 42;` -- int default for boolean field
- `int count = [1, 2, 3];` -- array default for int field
- `int count = 3.14;` -- float default for int field
- `string name = 42;` -- int default for string field
- `int count = null;` -- null default for non-nullable int field
- `int count = {"key": "value"};` -- object default for int field
- `bytes data = 42;` -- int default for bytes field

In all cases, Rust exits 0 and writes JSON with the invalid default
value, while Java exits 1.

## Root cause

The `walk_variable` function in `reader.rs` (around line 988) parses
the default value as a generic `serde_json::Value` via
`walk_json_value` and stores it directly in the `Field` struct
without validating it against the field's `AvroSchema` type. There is
a TODO comment at line 1000 about `fixDefaultValue` (int-to-long
coercion), but no validation of the JSON value's type against the
Avro schema type.

Java's `Schema.Field` constructor (line 574 of `Schema.java`) calls
`validateDefault(name, schema, defaultValue)` when the `validate`
parameter is true. The `VALIDATE_DEFAULTS` thread-local defaults to
`true`, so all field construction in `IdlReader.exitVariableDeclaration`
triggers validation. When validation fails, Java throws
`AvroTypeException`, which propagates uncaught through the ANTLR
listener (manifesting as `NoSuchElementException` from stack
corruption -- a separate Java bug in error recovery).

## Avro specification requirement

The Avro 1.12.0 specification explicitly defines which JSON types are
valid defaults for each Avro type:

| Avro type    | Valid JSON type |
|-------------|-----------------|
| null        | null            |
| boolean     | boolean         |
| int, long   | integer         |
| float, double | number        |
| bytes, fixed | string (Unicode code points 0-255) |
| string      | string          |
| record      | object          |
| enum        | string (must be a symbol name) |
| array       | array           |
| map, record | object          |
| union       | type matching first schema in union |

## Affected files

- `src/reader.rs` -- `walk_variable` function (around line 988)
- `src/model/schema.rs` -- would need an `is_valid_default` method

## Reproduction

```sh
# Write test file:
cat > tmp/test-default-validation.avdl <<'EOF'
protocol P {
  record R {
    int count = "not_a_number";
    boolean flag = 42;
    string name = [1, 2, 3];
    int n = null;
  }
}
EOF

# Rust accepts (should reject):
cargo run -- idl tmp/test-default-validation.avdl tmp/test.avpr
echo $?  # 0

# Java rejects:
java -jar ../avro-tools-1.12.1.jar idl tmp/test-default-validation.avdl tmp/test-java.avpr
echo $?  # 1
```

## Suggested fix

1. Add an `is_valid_default(value: &serde_json::Value, schema: &AvroSchema) -> bool`
   function to `src/model/schema.rs` implementing the spec table above.

2. In `walk_variable`, after parsing the default value and resolving
   the field type, call `is_valid_default` and emit a
   `ParseDiagnostic` error if validation fails. Skip validation for
   types containing unresolved forward references (matching Java's
   `SchemaResolver.isFullyResolvedSchema` guard on line 625).

3. For union defaults, validate that the value matches the first
   schema in the union (per Avro spec).

4. Consider making validation optional via a CLI flag (e.g.,
   `--no-validate-defaults`) for users who want lenient parsing,
   though the default should be strict to match Java.

Priority: medium-high. Invalid defaults in the output JSON will cause
failures in downstream Avro consumers (serializers, deserializers,
schema registries).

## Relationship to existing issues

This is related to but distinct from issue `445ea3c2` (missing
`fixDefaultValue` int-to-long coercion). That issue is about type
*promotion* of valid defaults; this issue is about *rejection* of
invalid defaults. Both could be addressed together.
