# Duplicated LogicalType-to-base-type mapping in `union_type_key` and `is_valid_default`

## Symptom

The mapping from `LogicalType` variant to its underlying primitive type is
duplicated in three places:

1. `LogicalType::expected_base_type()` (line 182 of `model/schema.rs`) --
   the canonical definition
2. `AvroSchema::union_type_key()` (line 446-455 of `model/schema.rs`) --
   duplicates the same mapping to produce a type-name string
3. `is_valid_default()` (line 764-773 of `model/schema.rs`) --
   duplicates the same mapping to produce an `AvroSchema` variant

## Root cause

These two call sites were written before `expected_base_type()` existed (or
before it was recognized as reusable). Each manually matches all 9 variants
instead of delegating to the shared method.

## Affected files

- `src/model/schema.rs`: lines 446-455 (`union_type_key`) and 764-773 (`is_valid_default`)

## Reproduction

Search for `LogicalType::Date` in `model/schema.rs` -- it appears 8 times,
several of which are redundant with `expected_base_type()`.

## Suggested fix

In `union_type_key`:
```rust
// Before:
AvroSchema::Logical { logical_type, .. } => match logical_type {
    LogicalType::Date | LogicalType::TimeMillis => "int".to_string(),
    // ... 6 more arms
},

// After:
AvroSchema::Logical { logical_type, .. } => {
    logical_type.expected_base_type().as_str().to_string()
}
```

In `is_valid_default`:
```rust
// Before:
AvroSchema::Logical { logical_type, .. } => {
    let underlying = match logical_type {
        LogicalType::Date | LogicalType::TimeMillis => AvroSchema::Int,
        // ... 6 more arms
    };
    is_valid_default(value, &underlying)
}

// After:
AvroSchema::Logical { logical_type, .. } => {
    is_valid_default(value, &logical_type.expected_base_type().to_schema())
}
```

This is a safe, mechanical change that reduces ~20 lines to ~4 and ensures
all three sites stay in sync if a new logical type is added.
