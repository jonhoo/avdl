# Unresolved references in schema mode main schema are not detected

## Symptom

When a schema-mode `.avdl` file uses `schema <type>;` and the type
reference cannot be resolved (because the type does not exist, or
because the namespace is wrong), the Rust `idl` subcommand silently
outputs the unresolved reference as a bare JSON string and exits with
status 0. Java correctly detects this as "Undefined schema" and exits
with an error.

## Root cause

The reference validation in `src/main.rs` calls
`registry.validate_references()`, which checks references *within*
schemas that have been registered in the `SchemaRegistry`. However,
the main schema from the `schema <type>;` declaration is stored in
`IdlFile::SchemaFile(schema)` and is never registered in the registry.
If that schema (or any references within it) is unresolved, the
validation misses it.

The same gap exists for `idl2schemata`: when `schema DoesNotExist;`
is used with no named types, the `SchemaFile` variant's unresolved
reference is never checked, and `idl2schemata` exits 0 with no
output.

## Affected files

- `src/main.rs` -- `run_idl` and `run_idl2schemata` do not validate
  the main schema itself, only registry contents

## Reproduction

```sh
# Case 1: Completely undefined type
cat > tmp/schema-undefined.avdl <<'EOF'
namespace com.example;
schema DoesNotExist;
EOF

cargo run -- idl tmp/schema-undefined.avdl
# Output: "com.example.DoesNotExist"  (exit 0)
# Expected: error about undefined schema (exit 1)

java -jar avro-tools-1.12.1.jar idl tmp/schema-undefined.avdl
# Exception: Undefined schema: com.example.DoesNotExist (exit 1)

# Case 2: Namespace mismatch
cat > tmp/schema-ns-override.avdl <<'EOF'
namespace com.example;
schema MyRecord;

@namespace("com.other")
record MyRecord {
  string name;
}
EOF

cargo run -- idl tmp/schema-ns-override.avdl
# Output: "com.example.MyRecord"  (exit 0)
# Expected: error about undefined schema (exit 1)

java -jar avro-tools-1.12.1.jar idl tmp/schema-ns-override.avdl
# Exception: Undefined schema: com.example.MyRecord (exit 1)

# Case 3: idl2schemata with undefined main schema
cargo run -- idl2schemata tmp/schema-undefined.avdl tmp/out/
# Exit 0, no output files, no error
# Java fails with: Undefined schema: com.example.DoesNotExist
```

## Suggested fix

After serializing the `SchemaFile` variant's JSON in `run_idl`,
check whether the resulting JSON value contains any bare string
references that should have been inlined but were not. Alternatively,
add a dedicated validation pass that walks the `AvroSchema` tree and
checks all `Reference` nodes against the registry before
serialization.

A simpler approach: after `schema_to_json`, check if any `Reference`
nodes in the original `AvroSchema` tree point to names not present in
the registry's `SchemaLookup`. This catches both the top-level
reference (`schema DoesNotExist;`) and references nested inside
complex types (`schema array<DoesNotExist>;`).

For `idl2schemata`, the same validation should apply: even though
`idl2schemata` only writes named schemas, it should still validate
that the `schema` keyword's type is resolvable.

## Priority

High -- this is a silent data correctness issue. The tool exits 0
and produces invalid output (a bare string where a schema definition
should be).
