# Reference inlining not supported in schema mode

## Symptom

`test_schema_syntax` fails: the main schema is `array<StatusUpdate>`
where `StatusUpdate` is imported via `import idl "status_schema.avdl"`.
Our output has `"items": "StatusUpdate"` (bare string reference) but
the expected output inlines the full record definition.

## Root cause

In schema mode, `schema_to_json` is called with an empty
`SchemaLookup`. References cannot be resolved because:

1. Schema-mode files don't build a lookup table (only protocol mode
   does via `build_lookup` in `protocol_to_json`).
2. The IDL import pipeline registers types into the registry, but
   after importing, the test's `parse_and_serialize_with_idl_imports`
   function doesn't build a lookup from the merged registry.

## Location

- `tests/integration.rs`: `parse_and_serialize_with_idl_imports` —
  passes `SchemaLookup::new()` (empty) for schema mode
- `src/model/json.rs`: `schema_to_json` — Reference arm can't inline
  without a populated lookup

## Expected behavior

When serializing a standalone schema, any `Reference` nodes should be
resolved against a lookup built from the registry (which contains all
imported types). This requires building a `SchemaLookup` from the
registry before calling `schema_to_json`.

## Difficulty

Moderate — need to build a lookup from the registry in schema mode,
similar to how `protocol_to_json` builds one from `protocol.types`.
May also need the import pipeline to handle nested IDL imports
recursively for the `status_schema.avdl` case.
