# Imported .avsc nested named types not promoted to protocol-level types

## Symptom

When importing a `.avsc` file that contains nested named types (records,
enums, or fixed types defined inline within record fields), those nested
types are not promoted to the protocol-level `types` array. Instead, they
remain inline within the parent record.

Java's behavior is to register all nested named types from imported schemas
as top-level types in the protocol. In the serialized `.avpr` output,
these types appear as separate entries in the `types` array, and their
parent record references them by name string instead of containing inline
definitions.

## Root cause

`import_schema()` in `src/import.rs` calls `register_all_named_types()`
which registers nested types in the `SchemaRegistry` for reference
resolution. However, the types are registered as standalone entries in the
registry alongside the parent schema, not instead of their inline
positions within the parent.

The problem is in the schema *serialization* path: when `schema_to_json`
in `src/model/json.rs` serializes the imported record, it sees the nested
named types for the first time and emits them inline (full definition)
rather than as name-string references. This is because the imported schema
was stored in the registry as-is (with nested types inline), and the
JSON serializer's `known_names` set has not yet seen those nested type
names when it encounters them inside the parent record.

In Java, `Schema.parse()` recursively registers all named schemas in the
`Names` map, and subsequent serialization always emits name references for
already-registered types. The key difference is that Java's types list is
built from the flat registry, not from the hierarchical schema tree.

## Affected files

- `src/import.rs` — `import_schema()` and `register_all_named_types()`
- `src/model/json.rs` — `schema_to_json()` serialization
- `src/main.rs` — `process_decl_items()` and protocol type list construction

## Reproduction

```sh
# Create nested-types.avsc:
cat > tmp/nested-types.avsc <<'EOF'
{
  "type": "record",
  "name": "OuterRecord",
  "namespace": "org.nested",
  "fields": [
    {
      "name": "inner",
      "type": {
        "type": "record",
        "name": "InnerRecord",
        "fields": [{"name": "x", "type": "int"}]
      }
    }
  ]
}
EOF

# Create test.avdl:
cat > tmp/import-nested-schema.avdl <<'EOF'
@namespace("org.test")
protocol ImportNestedSchema {
  import schema "nested-types.avsc";
  record UseNested {
    org.nested.OuterRecord outer;
    org.nested.InnerRecord inner;
  }
}
EOF

# Compare Rust vs Java:
scripts/compare-adhoc.sh tmp/import-nested-schema.avdl
```

**Expected** (Java output): `InnerRecord` appears as a separate top-level
type, and `OuterRecord` references it by name string `"InnerRecord"`.

**Actual** (Rust output): `InnerRecord` is defined inline within
`OuterRecord`'s field, and does not appear as a separate top-level type.

## Scope

This affects:
- `import schema` of `.avsc` files with nested named types
- `import protocol` of `.avpr` files with nested named types in their
  `types` array (same serialization path)
- Any imported schema containing enums, fixed types, or records defined
  inline within record fields, union branches, array items, or map values

The existing golden tests are not affected because the golden `.avsc`
files (`baz.avsc`, `foo.avsc`, etc.) are simple records without nested
named types.

## Suggested fix

Two complementary changes are needed:

1. **Flatten imported schemas**: When `import_schema()` registers an
   imported schema, it should also add the nested named types as separate
   entries in the registry (not just for reference resolution, but as
   actual protocol-level types). This may require the registry to also
   store the flattened nested types as independent entries.

2. **Replace inline definitions with references in imported schemas**:
   After extracting nested named types, replace their inline definitions
   within the parent record with `Reference` nodes. This ensures the
   serializer emits name strings instead of inline definitions.

Alternatively, the serializer could be changed to pre-populate
`known_names` with all registered type names before serialization begins,
so that any nested named type encountered during serialization is
emitted as a reference rather than inline. This would be a simpler
change but might have unintended effects on locally-defined types.
