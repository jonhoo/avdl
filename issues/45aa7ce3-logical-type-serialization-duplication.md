# LogicalType serialization repeats type/name mapping inline

## Symptom

The `LogicalType` match arm in `schema_to_json` (`model/json.rs`,
lines 432-492) repeats the same pattern nine times:

```rust
LogicalType::Date => {
    obj.insert("type".to_string(), Value::String("int".to_string()));
    obj.insert("logicalType".to_string(), Value::String("date".to_string()));
}
LogicalType::TimeMillis => {
    obj.insert("type".to_string(), Value::String("int".to_string()));
    obj.insert("logicalType".to_string(), Value::String("time-millis".to_string()));
}
// ... 7 more arms with the same structure
```

The base type is already available via `LogicalType::expected_base_type()`
(which returns `PrimitiveType`, with `PrimitiveType::as_str()`). The
logical type name string, however, exists only in the `parse_logical_type`
function (the forward mapping) and this match arm (the reverse mapping)
-- there is no `LogicalType::name() -> &'static str` method.

## Root cause

The reverse mapping (variant to name string) was never extracted into
a method on `LogicalType`.

## Affected files

- `src/model/json.rs` (lines 432-492)
- `src/model/schema.rs` (`LogicalType` enum, `parse_logical_type` function)

## Suggested fix

Add a `pub(crate) fn name(&self) -> &'static str` method to
`LogicalType` that returns the Avro logical type name string (e.g.,
`"date"`, `"time-millis"`, `"decimal"`). Then the entire match arm
in `json.rs` can be replaced with:

```rust
AvroSchema::Logical { logical_type, properties } => {
    let mut obj = Map::new();
    obj.insert("type".to_string(),
        Value::String(logical_type.expected_base_type().as_str().to_string()));
    obj.insert("logicalType".to_string(),
        Value::String(logical_type.name().to_string()));
    if let LogicalType::Decimal { precision, scale } = logical_type {
        obj.insert("precision".to_string(), Value::Number((*precision).into()));
        obj.insert("scale".to_string(), Value::Number((*scale).into()));
    }
    for (k, v) in properties {
        obj.insert(k.clone(), v.clone());
    }
    Value::Object(obj)
}
```

This also makes the `parse_logical_type` function's name-to-variant
table and the new `name()` method into a single source of truth pair
that can be cross-checked in tests.
