# Missing logical type promotion from custom annotations

## Symptom

Java's `SchemaProperties.copyProperties()` method (line 1057-1065 of
`IdlReader.java`) calls `LogicalTypes.fromSchemaIgnoreInvalid(schema)`
after applying custom properties to a Schema. If the properties
contain a `logicalType` key that matches a known logical type, it is
promoted to a proper `LogicalType` on the schema. The Rust code does
not perform this promotion.

## Root cause

After applying custom annotations, Java checks if the resulting
schema now has properties that constitute a recognized logical type.
For example, if a user writes:

```avdl
@logicalType("date") int myField;
```

Java would:
1. Create an `int` schema
2. Add `{"logicalType": "date"}` as a custom property
3. Call `LogicalTypes.fromSchemaIgnoreInvalid(schema)` which
   recognizes `date` and calls `logicalType.addToSchema(schema)`,
   properly registering it as a logical type

The Rust code in `walk_schema_properties` and `apply_properties`
treats all non-intercepted annotations as opaque custom properties.
There is no post-hoc promotion step.

## Impact on output

This primarily affects exotic usage patterns where users annotate
types with `@logicalType(...)` instead of using the built-in IDL
syntax (`date`, `time_ms`, etc.). In normal usage, the built-in
primitive types (`date`, `time_ms`, `timestamp_ms`, etc.) are parsed
directly into `AvroSchema::Logical` variants without needing this
promotion.

The impact is limited because:
1. The IDL syntax provides direct keywords for all standard logical
   types, so `@logicalType("date")` is an unusual way to express this.
2. The JSON output may still be correct since the custom property
   `"logicalType": "date"` would appear in the output either way.

However, Java's promotion step also validates the logical type (e.g.,
that `decimal` has `precision`), which means Java may reject invalid
logical types that Rust silently accepts.

## Affected files

- `src/reader.rs` -- `walk_schema_properties`, `apply_properties`

## Reproduction

```avdl
@namespace("test")
protocol P {
    record R { @logicalType("date") int myDate; }
}
```

In Java, the `int` field gets a proper `date` logical type registered.
In Rust, it gets `AnnotatedPrimitive { kind: Int, properties:
{"logicalType": "date"} }`.

The JSON output is likely the same (`{"type": "int", "logicalType":
"date"}`), but the semantic model differs.

## Suggested fix

After applying custom properties, check if the result contains a
`logicalType` key and if so, try to promote it to a recognized
`LogicalType`. This is low priority since the standard IDL syntax
covers all common cases.
