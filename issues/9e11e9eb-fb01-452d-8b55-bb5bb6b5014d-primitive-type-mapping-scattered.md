# Primitive type name-to-schema mapping scattered across 4+ functions

## Symptom

The mapping from primitive type name strings (`"null"`, `"boolean"`,
`"int"`, `"long"`, `"float"`, `"double"`, `"bytes"`, `"string"`) to
their corresponding `AvroSchema` or `PrimitiveType` variants is
repeated in at least 5 separate match expressions across 3 modules:

1. `import.rs` `primitive_from_str` (line 738-750): maps `&str` to `AvroSchema` variant
2. `import.rs` `str_to_primitive_type` (line 756-768): maps `&str` to `PrimitiveType`
3. `import.rs` `string_to_schema` (line 409-438): maps `&str` to `AvroSchema` (with
   named-reference fallthrough)
4. `model/json.rs` `schema_to_json` primitive arms (line 188-195): maps `AvroSchema`
   variant to `Value::String`
5. `model/schema.rs` `PrimitiveType::as_str` (line 30-41): maps `PrimitiveType` to `&str`

Additionally, the union_type_key and type_description methods in
`schema.rs` both repeat the same 8-arm primitive-to-string mapping
(lines 223-230 and 266-273), although they at least delegate to
`full_name()` for named types.

## Root cause

Each function was written independently as the codebase grew. The
`PrimitiveType` enum (added later) centralizes the enum variants, but
its `as_str()` method is not consistently used, and there is no
corresponding `from_str()` / `FromStr` implementation.

## Affected files

- `src/import.rs` (3 functions)
- `src/model/json.rs` (1 match)
- `src/model/schema.rs` (3 methods/functions)

## Reproduction

Search for match arms containing `"null" => ` or `"boolean" => ` or
`AvroSchema::Null =>` in the codebase. Each hit is an instance of
this pattern.

## Suggested fix

1. Implement `FromStr` for `PrimitiveType` (or add a `PrimitiveType::from_str`
   associated function), making `str_to_primitive_type` in `import.rs`
   redundant.

2. Add `PrimitiveType::to_schema(&self) -> AvroSchema` to convert a
   `PrimitiveType` to its `AvroSchema` variant, making
   `primitive_from_str` in `import.rs` a thin wrapper.

3. In `string_to_schema`, use `PrimitiveType::from_str` with a
   fallthrough to the named-type-reference branch.

4. In `schema_to_json`, the 8 primitive arms could use a method on
   `AvroSchema` or match against a single arm with a helper, but this
   is a smaller win since the match is already concise.

Estimated savings: ~30 lines of duplicated match arms, plus a single
source of truth for the primitive type name set.
