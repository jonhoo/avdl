# `idl2schemata` serializes schemas without namespace context

## Symptom

The `idl2schemata` subcommand produces `.avsc` files with several
differences from the Java `idl2schemata` output:

1. **Type references use fully-qualified names** instead of short
   names within the same namespace (e.g., `"org.apache.avro.echo.Ping"`
   instead of `"Ping"`).
2. **Redundant namespace keys**: standalone `.avsc` files include an
   explicit `"namespace"` field even when the types should inherit it
   from the protocol namespace context (needs verification against
   Java behavior -- see SESSION.md).

## Partially fixed

The empty lookup problem was fixed: `run_idl2schemata` now builds a
`SchemaLookup` from registry schemas via `build_lookup`, so
`Reference` nodes can be resolved and inlined. However, two problems
remain.

## Remaining root cause

In `src/main.rs:169`, the per-schema serialization call:

```rust
let json_value = schema_to_json(schema, &mut IndexSet::new(), None, &all_lookup);
```

Two remaining problems:

1. **`None` enclosing namespace**: passing `None` for the namespace
   means `schema_ref_name` can never shorten fully-qualified names
   to simple names, even when the type is in the same namespace.
2. **Fresh `known_names` per schema**: each schema gets a fresh empty
   `IndexSet`, so there's no cross-schema tracking of which types
   have already been emitted as separate files vs. which should be
   inlined.

The Java `IdlToSchemataTool` calls `schema.toString(true)` on each
named schema from `idlFile.getNamedSchemas()`. Java's `Schema`
objects carry their full type graph, so `toString(true)` naturally
inlines referenced types using its internal `known_names` tracking.

## Affected files

- `src/main.rs:149-169` -- `run_idl2schemata` loop

## Suggested fix

Pass the protocol namespace as the enclosing namespace and share
`known_names` across schema iterations:

```rust
let mut known_names = IndexSet::new();
for schema in registry.schemas() {
    let json_value = schema_to_json(
        schema, &mut known_names,
        namespace.as_deref(), &all_lookup,
    );
    // ...
}
```

Sharing `known_names` across iterations ensures that types inlined
inside a record on first encounter are emitted as bare strings in
subsequent `.avsc` files, matching Java behavior.
