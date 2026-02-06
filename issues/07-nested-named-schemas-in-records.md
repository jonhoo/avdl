# Nested named schema declarations inside records not handled

## Symptom

The Java IDL parser supports declaring named schemas (records, enums,
fixed) inside a record body. Our parser ignores the `_registry`
parameter in `walk_record` and does not walk nested named schema
declarations.

## Root cause

`walk_record` receives a `_registry: &mut SchemaRegistry` parameter
but never uses it. The function body doesn't look for
`namedSchemaDeclaration` children within the record context.

## Location

- `src/reader.rs:366-420` — `walk_record` function

## Expected behavior

Walk `namedSchemaDeclaration` children inside the record, register
them in the schema registry, and include them in the protocol's type
list (or inline them at first reference).

## Difficulty

Complex — requires understanding how nested types interact with
namespacing and registration order.
