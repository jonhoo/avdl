# Reference nodes cannot carry custom schema properties

## Symptom

Properties applied to a named type reference via annotations
(e.g., `@prop("val") MyRecord field;`) are silently dropped because
`AvroSchema::Reference(String)` has no properties field.

## Root cause

`AvroSchema::Reference` is defined as a simple newtype wrapper around
`String`. The `apply_properties_to_schema` function in `reader.rs`
falls through to the catch-all arm for References:

    other => other,

which discards any properties.

## Location

- `src/model/schema.rs` — `Reference(String)` definition
- `src/reader.rs` — `apply_properties_to_schema` catch-all

## Expected behavior

References with properties should be preserved. Options:
- Change `Reference` to `Reference { name: String, properties: IndexMap }`
- Or resolve the reference first, then apply properties to the
  resolved schema

## Difficulty

Moderate — depends on how references are resolved in the pipeline.
