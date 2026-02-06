# Import path drops properties on primitives without logical type

## Symptom

When importing `.avsc` files, primitives with custom properties but no
logical type lose those properties. For example,
`{"type": "int", "customProp": "value"}` becomes bare `AvroSchema::Int`.

## Root cause

In `src/import.rs`, `parse_annotated_primitive` collects extra
properties but then discards them when returning a bare primitive
(line ~486):

    Ok(primitive_from_str(prim))

The reader-side equivalent was fixed by adding
`AvroSchema::AnnotatedPrimitive`, but the import-side JSON parser
still drops properties.

## Location

- `src/import.rs:486-489` — `parse_annotated_primitive` discards
  properties for non-logical-type primitives

## Expected behavior

Custom properties should be preserved, likely by returning
`AvroSchema::AnnotatedPrimitive { kind, properties }` when extra
properties are present.

## Difficulty

Easy — use the existing `AnnotatedPrimitive` variant.
