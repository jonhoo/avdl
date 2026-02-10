# Missing micros-precision logical type variants in `LogicalType` enum

## Symptom

The Avro specification defines the following logical types that use
`long` as the underlying type:

- `timestamp-millis` (long)
- `timestamp-micros` (long)
- `local-timestamp-millis` (long)
- `local-timestamp-micros` (long)
- `time-millis` (int)
- `time-micros` (long)

The `LogicalType` enum in `schema.rs` only includes millis-precision
variants:

```rust
pub enum LogicalType {
    Date,
    TimeMillis,
    TimestampMillis,
    LocalTimestampMillis,
    Uuid,
    Decimal { precision: u32, scale: u32 },
}
```

Missing variants: `TimeMicros`, `TimestampMicros`,
`LocalTimestampMicros`.

The IDL spec says there are no built-in keywords for micros types --
they must be expressed via `@logicalType` annotations:

```avdl
@logicalType("timestamp-micros") long finishTime;
```

Currently, `@logicalType("timestamp-micros")` is handled as an
`AnnotatedPrimitive` (the unknown-type path in `try_promote_logical`).
This is functionally correct for JSON output: it emits
`{"type": "long", "logicalType": "timestamp-micros"}`, which is the
right serialization.

However, this means:

1. **Default value validation for micros types is not type-aware.**
   `AnnotatedPrimitive` validates against the underlying primitive,
   which happens to be correct for long-based types but is accidental
   rather than intentional.

2. **`union_type_key()` for micros types returns `"long"` instead of
   the logical type key.** If a union contains both
   `@logicalType("timestamp-micros") long` and a plain `long`, they
   would both have union key `"long"` and would (correctly) be
   rejected as duplicates. But if the intent was to allow both (as
   Java does for some logical type combinations), this would be wrong.

3. **The `logicalTypes.avdl` golden test uses
   `@logicalType("timestamp-micros")`** (line 28), and it is tested --
   the field propagates correctly to JSON. But the test only checks the
   JSON key, not the internal model.

The Avro `duration` logical type (fixed 12-byte) is also absent from
the enum. The IDL spec does not mention `duration` as a built-in
keyword, and the Java test suite does not use it, so this is lower
priority.

## Root cause

The initial implementation only added logical type variants for the
six types that have IDL keywords (`date`, `time_ms`, `timestamp_ms`,
`local_timestamp_ms`, `uuid`, `decimal`). The micros-precision types
and `duration` were omitted because they lack keywords and are handled
via the annotation fallback path.

## Affected files

- `src/model/schema.rs` (`LogicalType` enum)
- `src/reader.rs` (`try_promote_logical` function)
- `src/model/json.rs` (serialization logic for logical types)

## Reproduction

The existing `test_logical_types_file` integration test passes because
`@logicalType("timestamp-micros")` produces correct JSON via
`AnnotatedPrimitive`. No test verifies the internal model
representation.

## Suggested fix

Add `TimeMicros`, `TimestampMicros`, and `LocalTimestampMicros`
variants to `LogicalType`. Update `try_promote_logical` to recognize
these annotation values and promote them to proper `Logical` variants
(with correct base type checking: `time-micros` -> long,
`timestamp-micros` -> long, `local-timestamp-micros` -> long). Add
unit tests for each.

Optionally add `Duration` (fixed 12 bytes) but this is lower priority
since it is not used in the test suite.
