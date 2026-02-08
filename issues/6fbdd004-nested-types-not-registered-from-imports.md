# Nested named types not registered when importing .avsc or .avpr files

## Symptom

When importing an `.avsc` or `.avpr` file whose schemas contain nested
named types (records, enums, or fixed types embedded within record
fields, union branches, array items, or map values), the nested types
are parsed into the `AvroSchema` tree but are not individually
registered in the `SchemaRegistry`. Subsequent references to those
nested types by name fail with "Undefined name".

## Root cause

`import_schema` (in `src/import.rs`) calls `json_to_schema` to parse
the JSON and then calls `registry.register(schema)` only on the
top-level schema. It does not recursively walk the schema tree to
discover and register nested named types.

`import_protocol` has the same issue. It iterates the protocol's
`"types"` array and registers each top-level entry, but if any of
those entries contain inline nested named types, those nested types
are not separately registered.

By contrast, Java's `JsonSchemaParser.parse()` and `Protocol.parse()`
recursively register all named types encountered during JSON parsing
via `ParseContext.put()`, which makes them available for subsequent
references.

## Affected files

- `src/import.rs` -- `import_schema()` and `import_protocol()`
- `src/resolve.rs` -- `SchemaRegistry` (needs a method to recursively
  register all named types from a schema tree)

## Reproduction

```sh
# Create an .avsc with a nested record
cat > tmp/nested.avsc <<'JSON'
{
  "type": "record",
  "name": "Outer",
  "namespace": "test.nested",
  "fields": [{
    "name": "inner",
    "type": {
      "type": "record",
      "name": "Inner",
      "fields": [{"name": "value", "type": "int"}]
    }
  }]
}
JSON

# Import it and reference the nested type
cat > tmp/import-nested.avdl <<'AVDL'
@namespace("test.nested")
protocol P {
  import schema "nested.avsc";
  record Wrapper {
    Outer outer;
    Inner inner;
  }
}
AVDL

# Rust fails:
cargo run -- idl tmp/import-nested.avdl
# Error: Undefined name: test.nested.Inner

# Java succeeds:
java -jar ../avro-tools-1.12.1.jar idl tmp/import-nested.avdl
# Outputs valid JSON with both Outer and Inner types
```

The same issue occurs with:
- Nested records in union branches of imported `.avsc` files
- Nested records in array items of imported `.avsc` files
- Nested named types in `.avpr` protocol type arrays

## Suggested fix

Add a helper function (e.g., `register_all_named_types`) to
`SchemaRegistry` or `import.rs` that recursively walks an
`AvroSchema` tree and registers every named type it encounters.
Call this function from both `import_schema` and `import_protocol`
instead of only registering the top-level schema.

The recursive walk should handle:
- `AvroSchema::Record` -- register the record, then recurse into its
  fields' schemas
- `AvroSchema::Enum` and `AvroSchema::Fixed` -- register directly
- `AvroSchema::Union` -- recurse into each branch
- `AvroSchema::Array` -- recurse into items
- `AvroSchema::Map` -- recurse into values
- All other variants -- no action needed

Note: The `build_lookup` function in `src/model/json.rs` already
implements this recursive walk for serialization purposes. The
registration logic could follow the same pattern.
