# Decimal precision overflow (>i32) treated as valid logical type

## Symptom

When `@precision(3000000000)` is used on a `@logicalType("decimal")`
bytes field, the Rust tool promotes it to a fully valid `Logical`
`Decimal` schema. Java treats this as an invalid precision and does
NOT promote it to a logical type, because 3000000000 exceeds
`Integer.MAX_VALUE` (2,147,483,647).

The Java `TestLogicalTypes.incorrectlyAnnotatedBytesFieldHasNoLogicalType()`
test explicitly verifies this behavior:

```java
assertNull(fieldSchema.getLogicalType());
assertEquals("decimal", fieldSchema.getObjectProp("logicalType"));
assertEquals(3000000000L, fieldSchema.getObjectProp("precision"));
```

In Java, the field has no logicalType (null), but the `logicalType`
and `precision` properties are preserved as raw JSON properties on
the bytes type. In Rust, the field is promoted to
`Logical { Decimal { precision: 3000000000, scale: 0 } }`.

## Root cause

The Rust `try_promote_logical_type` function uses `json_value_as_u32`
to extract the `precision` value. Since 3000000000 fits in a `u32`
(max 4,294,967,295), it passes the check and promotion succeeds.

Java uses `int` (signed 32-bit) for precision values. Since
3000000000 exceeds `Integer.MAX_VALUE` (2,147,483,647), it is stored
as a `long` in the JSON properties rather than an `int`. Java's
`LogicalTypes.decimal()` factory expects an `int` precision, so the
long value fails validation and the schema is left without a logical
type.

The precision/scale fields in `LogicalType::Decimal` are defined as
`u32` in Rust. This is technically wrong for Java compatibility:
they should behave as signed 32-bit integers, treating values above
`i32::MAX` as invalid.

## Affected files

- `src/reader.rs` -- `json_value_as_u32` and `try_promote_logical_type`
- `src/model/schema.rs` -- `LogicalType::Decimal { precision: u32, scale: u32 }`

## Reproduction

```avdl
@namespace("org.apache.avro.test")
protocol P {
    record R {
        @logicalType("decimal") @precision(3000000000) @scale(0) bytes byteArray;
    }
}
```

Rust output (incorrect -- should not have `logicalType`):
```json
{
  "type": "bytes",
  "logicalType": "decimal",
  "precision": 3000000000,
  "scale": 0
}
```

Java output (correct):
```json
{
  "type": "bytes",
  "logicalType": "decimal",
  "precision": 3000000000,
  "scale": 0
}
```

Note: The JSON output looks identical, but in Java the `logicalType`
property is a raw JSON property (not a promoted LogicalType), so
downstream consumers see it as an opaque annotation. The distinction
matters for tools that check `schema.getLogicalType()` at the API
level.

## Suggested fix

Change `json_value_as_u32` to `json_value_as_i32` (or add a separate
function) that rejects values exceeding `i32::MAX`. Use this for
decimal precision and scale extraction in `try_promote_logical_type`.
Values exceeding `i32::MAX` should cause the promotion to fail,
leaving the schema as an `AnnotatedPrimitive`.

Low priority: the JSON output is identical. This only matters for
API-level consumers that distinguish between promoted logical types
and raw annotation properties.

## Test case (from Java test suite)

From `avro/lang/java/compiler/src/test/java/org/apache/avro/compiler/idl/TestLogicalTypes.java`:

```java
@Test
void incorrectlyAnnotatedBytesFieldHasNoLogicalType() {
    Schema fieldSchema = logicalTypeFields.getField("byteArray").schema();
    assertNull(fieldSchema.getLogicalType());
    assertEquals("decimal", fieldSchema.getObjectProp("logicalType"));
    assertEquals(3000000000L, fieldSchema.getObjectProp("precision"));
    assertEquals(0, fieldSchema.getObjectProp("scale"));
}
```

The test input is in `avro/lang/java/compiler/src/test/idl/logicalTypes.avdl`.
