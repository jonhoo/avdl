# `idl2schemata` serializes schemas without lookup table or namespace context

## Symptom

The `idl2schemata` subcommand produces `.avsc` files with several
differences from the Java `idl2schemata` output:

1. **Type references use fully-qualified names** instead of short
   names within the same namespace (e.g., `"org.apache.avro.echo.Ping"`
   instead of `"Ping"`).
2. **Nested named types are not inlined**: a record that references
   an enum (e.g., `ANameValue` referencing `ValueType`) emits the
   reference as a bare string instead of inlining the full enum
   definition on first use.
3. **Redundant namespace keys**: standalone `.avsc` files include an
   explicit `"namespace"` field even when the types should inherit it
   from the protocol namespace context (needs verification against
   Java behavior -- see SESSION.md).

For example, `forward_ref.avdl`'s `ANameValue.avsc` has:

```json
{"name": "type", "type": "org.foo.ValueType"}
```

But Java's output inlines the full `ValueType` enum definition at
that position.

## Root cause

In `src/main.rs:158-159`, the `idl2schemata` loop serializes each
schema with:

```rust
let empty_lookup = SchemaLookup::new();
let json_value = schema_to_json(schema, &mut IndexSet::new(), None, &empty_lookup);
```

Three problems:

1. **Empty lookup**: `SchemaLookup::new()` means `Reference` nodes
   cannot be resolved, so nested named types are never inlined.
2. **`None` enclosing namespace**: passing `None` for the namespace
   means `schema_ref_name` can never shorten fully-qualified names
   to simple names, even when the type is in the same namespace.
3. **Fresh `known_names` per schema**: each schema gets a fresh empty
   `IndexSet`, so there's no cross-schema tracking of which types
   have already been emitted as separate files vs. which should be
   inlined.

The Java `IdlToSchemataTool` calls `schema.toString(true)` on each
named schema from `idlFile.getNamedSchemas()`. Java's `Schema`
objects carry their full type graph, so `toString(true)` naturally
inlines referenced types using its internal `known_names` tracking.

## Affected files

- `src/main.rs:143-169` -- `run_idl2schemata` loop

## Reproduction

```sh
mkdir -p /tmp/schemata
cargo run -- idl2schemata \
  avro/lang/java/idl/src/test/idl/input/forward_ref.avdl /tmp/schemata
cat /tmp/schemata/ANameValue.avsc
# Shows "type": "org.foo.ValueType" instead of inlined enum
ls /tmp/schemata/
# Shows ValueType.avsc as a separate file (Java might also emit it,
# but inlines it into ANameValue.avsc on first encounter)
```

## Suggested fix

Build a `SchemaLookup` from the registry (similar to how
`protocol_to_json` calls `build_lookup`) and pass the protocol
namespace as the enclosing namespace:

```rust
let lookup = build_lookup_from_registry(&registry, namespace.as_deref());
let mut known_names = IndexSet::new();
for schema in registry.schemas() {
    let json_value = schema_to_json(
        schema, &mut known_names,
        namespace.as_deref(), &lookup,
    );
    // ...
}
```

Sharing `known_names` across iterations ensures that types inlined
inside a record on first encounter are emitted as bare strings in
subsequent `.avsc` files, matching Java behavior.
