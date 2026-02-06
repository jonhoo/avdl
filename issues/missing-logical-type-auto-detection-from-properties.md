# Missing logical type auto-detection from `@logicalType` annotation properties

## Symptom

When a user annotates a primitive type with `@logicalType("timestamp-micros")`
or `@logicalType("decimal")` plus `@precision(6) @scale(2)`, the Java
tool automatically detects the logical type from the custom property and
"activates" it (adds it to the schema properly). The Rust tool does not
perform this auto-detection -- it stores `logicalType` as a plain custom
property.

## Root cause

The Java `SchemaProperties.copyProperties` method (IdlReader.java lines
1055-1065) has this logic:

```java
public <T extends JsonProperties> T copyProperties(T jsonProperties) {
    properties.forEach(jsonProperties::addProp);
    if (jsonProperties instanceof Schema) {
        Schema schema = (Schema) jsonProperties;
        LogicalType logicalType = LogicalTypes.fromSchemaIgnoreInvalid(schema);
        if (logicalType != null) {
            logicalType.addToSchema(schema);
        }
    }
    return jsonProperties;
}
```

After copying all custom properties (including `logicalType`,
`precision`, `scale`) onto the schema, it calls
`LogicalTypes.fromSchemaIgnoreInvalid()` to see if those properties form
a valid logical type. If they do, the logical type is "activated" on the
schema, which means the schema is recognized as a proper logical type
(not just a primitive with custom properties).

The Rust `walk_schema_properties` and `apply_properties` functions in
`reader.rs` do not perform this auto-detection. Properties like
`@logicalType("timestamp-micros")` are stored as custom properties in
an `AnnotatedPrimitive` node, not as a `Logical` node.

This matters because the Java test file `logicalTypes.avdl` uses this
pattern:

```avdl
@logicalType("timestamp-micros") long timestampMicrosField;
@logicalType("decimal") @precision(6) @scale(2) bytes decimalField;
```

For `@logicalType("timestamp-micros") long`, the Java tool produces:

```json
{"type": "long", "logicalType": "timestamp-micros"}
```

The Rust tool also produces this (since it stores the property), but
the internal representation differs: Java has a full `LogicalType`
object on the schema, while Rust has a plain custom property. This
could matter for validation or for any code that inspects the
`AvroSchema` type rather than just serializing it.

## Important nuance

The Java implementation uses `fromSchemaIgnoreInvalid` -- note the
"IgnoreInvalid" suffix. This means that if the `@logicalType`
annotation specifies an unknown or invalid logical type (e.g.,
`@logicalType("timestamp-micros")` but on an `int` instead of `long`),
it silently falls through and the properties remain as custom
properties without error. The Rust implementation should match this
lenient behavior.

The Java test `logicalTypes.avdl` also tests invalid logical type
parameters:

```avdl
@logicalType("decimal") @precision(3000000000) @scale(0) bytes invalidDecimal;
```

Because the precision exceeds `Integer.MAX_VALUE`, the logical type
construction fails, and `fromSchemaIgnoreInvalid` returns null. The
field is then serialized as a plain annotated bytes type with the
custom properties preserved.

## Affected files

- `src/reader.rs` -- `apply_properties`, `apply_properties_to_schema`
- `src/model/schema.rs` -- may need to support converting annotated
  primitives to logical types

## Reproduction

```avdl
@namespace("test")
protocol P {
    record R {
        @logicalType("timestamp-micros") long timestampMicrosField;
    }
}
```

Both Java and Rust produce the same JSON output for this case. The gap
is in the internal model, not the serialized form. However, for invalid
logical type annotations, behavior may diverge when validation is
added.

## Suggested fix

In `apply_properties_to_schema`, after copying properties onto a
primitive or annotated primitive, check whether the properties contain
`logicalType` and, if so, attempt to construct the corresponding
`LogicalType` variant. If construction fails (unknown type, invalid
parameters), leave the properties as-is (matching Java's
`fromSchemaIgnoreInvalid` behavior).

## Priority

Low. The JSON output is identical for valid inputs. This only matters
for internal model correctness and for future validation features.
