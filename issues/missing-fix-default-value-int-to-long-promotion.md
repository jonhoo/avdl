# Missing `fixDefaultValue`: int-to-long default value promotion

## Symptom

When a field has a `long` type (or a union containing `long` but not
`int`) and the default value is a small integer that the parser
produces as an `i32` (`IntNode` in Java), the Rust tool serializes the
default as a JSON integer that may not match the Java output.

More importantly, the semantic mismatch means a downstream Avro
consumer expecting a `long` default may reject the schema if it
enforces strict type matching between the default value and the field
type.

## Root cause

The Java `IdlReader` has a `fixDefaultValue` method (lines 641-662)
that checks whether a parsed integer default value needs to be
promoted from `IntNode` to `LongNode` to match the field's schema
type:

```java
private JsonNode fixDefaultValue(JsonNode defaultValue, Schema fieldType) {
    if (!(defaultValue instanceof IntNode)) {
        return defaultValue;
    }
    if (fieldType.getType() == Schema.Type.UNION) {
        for (Schema unionedType : fieldType.getTypes()) {
            if (unionedType.getType() == Schema.Type.INT) {
                break;
            } else if (unionedType.getType() == Schema.Type.LONG) {
                return new LongNode(defaultValue.longValue());
            }
        }
        return defaultValue;
    }
    if (fieldType.getType() == Schema.Type.LONG) {
        return new LongNode(defaultValue.longValue());
    }
    return defaultValue;
}
```

This logic ensures that a default value like `0` on a `long` field is
stored as a `LongNode(0)` rather than an `IntNode(0)`.

The Rust `walk_variable` function (reader.rs line 630) does not
perform this promotion. The `parse_integer_literal` function returns
`i32` when the value fits, and this `i32` value is stored directly as
the field default without checking whether the field type requires
`i64`.

In practice, `serde_json` serializes both `i32` and `i64` as plain
JSON integers (no type distinction in JSON), so this gap may not cause
visible output differences. However, it is a semantic correctness gap
that could matter if the `Value` is ever inspected programmatically
(e.g., for type validation), and it means the domain model does not
match Java's.

## Affected files

- `src/reader.rs` -- `walk_variable` function (around line 630)

## Reproduction

```avdl
@namespace("test")
protocol P {
    record R {
        long count = 0;
        union { null, long } nullable_count = 0;
    }
}
```

In Java, both defaults are stored as `LongNode(0)`. In Rust, both are
stored as `Value::Number(0)` which is `i32`-backed.

## Suggested fix

Add a `fix_default_value` function that checks the field type and
promotes `i32`-range `Value::Number` to `i64` when the field type is
`Long` or a union where `Long` appears before `Int`.

## Priority

Low. JSON output is identical since JSON has no int/long distinction.
This is a semantic correctness gap in the domain model.
