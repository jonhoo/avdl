# Undefined type errors lack source spans

## Symptom

`SchemaRegistry::validate_references()` returns `Vec<String>` with
type names that couldn't be resolved, but no source locations. The
`test_error_undefined_type` snapshot test shows plain text like:

    unresolved references: ["test.Nonexistent"]

There is no source highlighting pointing at the offending field
declaration.

## Root cause

`validate_references()` walks registered schemas and checks that
every `AvroSchema::Reference` resolves to a known name. But
`Reference` only stores the type name (`String`), not where in the
source it was declared. Without a byte offset, we can't construct a
`ParseDiagnostic`.

## Affected files

- `src/resolve.rs` — `validate_references()`, `validate_schema()`
- `src/model/schema.rs` — `AvroSchema::Reference` variant

## Suggested fix

Add `Option<SourceSpan>` to `AvroSchema::Reference`:

    Reference { name: String, span: Option<miette::SourceSpan> }

This requires updating every `Reference` construction site in
`reader.rs`, all match arms across the codebase, and the JSON
serialization in `model/json.rs`. The span should come from
`span_from_context()` at the point where the reference type is
parsed.

This is a moderate-to-large change with cascading effects through
the model, tests, and JSON serialization layers.
