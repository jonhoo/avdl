# Unknown logical types silently dropped during import

## Symptom

When importing `.avsc` files with unknown `logicalType` values (e.g.,
custom logical types from other Avro implementations), the logical type
name and all associated properties are silently discarded.

## Root cause

In `src/import.rs`, `parse_annotated_primitive` matches known logical
types (`date`, `time-millis`, etc.) but the catch-all arm for unknown
types just returns the bare primitive:

    _ => {
        // TODO: Handle unknown logical types more gracefully...
        return Ok(primitive_from_str(prim));
    }

## Location

- `src/import.rs:467-473` — unknown logical type handling

## Expected behavior

Unknown logical types should be preserved. Options:
- Extend `LogicalType` to include an `Unknown(String)` variant
- Use `AnnotatedPrimitive` with the `logicalType` as a property
- Add a new variant to `AvroSchema`

## Difficulty

Medium — requires a model decision on how to represent unknown logical
types.
