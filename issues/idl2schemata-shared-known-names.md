# idl2schemata shares `known_names` across schema files, causing bare string output

## Symptom

When running `idl2schemata`, schemas that were already inlined inside
a previously-written `.avsc` file are emitted as bare JSON strings
(e.g., `"MD5"`) rather than their full definition. This produces
invalid `.avsc` files that cannot be parsed as standalone schemas.

For example, `simple/MD5.avsc` contains just `"MD5"` instead of:

```json
{
  "type": "fixed",
  "name": "MD5",
  "namespace": "org.apache.avro.test",
  "doc": "An MD5 hash.",
  "size": 16
}
```

This happens because MD5 was already inlined inside `TestRecord.avsc`
(which processes first and adds `MD5` to `known_names`). Similarly,
`echo/Pong.avsc` emits `"type": "Ping"` (bare reference) for the
`ping` field instead of inlining the full `Ping` record definition.

## Root cause

In `src/main.rs` line 177, `run_idl2schemata` declares a single
`let mut known_names = IndexSet::new()` and reuses it across all
schema iterations:

```rust
let mut known_names = IndexSet::new();

for schema in registry.schemas() {
    // ...
    let json_value = schema_to_json(schema, &mut known_names, ...);
    // ...
}
```

The Java implementation (`IdlToSchemataTool.java` line 102) calls
`schema.toString(true)`, which internally creates a **fresh**
`new HashSet<String>()` for each schema:

```java
public String toString(boolean pretty) {
    return toString(new HashSet<String>(), pretty);
}
```

This means each `.avsc` file is serialized independently with no
knowledge of what was already serialized in other files. Named types
referenced within a schema are inlined on first occurrence within
*that file*, and subsequent references within the same file use
bare strings.

## Affected files

- `src/main.rs` -- `run_idl2schemata` function (line 177)
- All idl2schemata outputs are affected

## Affected tests

All 14 failing comparisons in `scripts/compare-golden.sh idl2schemata`
are caused by this issue (either directly or in combination with the
missing-namespace issue).

Specific examples:
- `simple/MD5.avsc` -- bare string `"MD5"` instead of full fixed definition
- `echo/Pong.avsc` -- `"type": "Ping"` instead of inlined record
- `interop/Interop.avsc` -- bare refs for `Foo`, `Kind`, `MD5`, `Node`
- `namespaces/RefersToOthers.avsc` -- bare refs instead of inlined types

## Reproduction

```sh
scripts/compare-golden.sh idl2schemata simple
# simple/MD5.avsc will FAIL: Rust output is bare "MD5" string
```

Or directly:

```sh
cargo run -- idl2schemata avro/lang/java/idl/src/test/idl/input/simple.avdl tmp/test-simple/
cat tmp/test-simple/MD5.avsc
# Output: "MD5"   (should be a full JSON object)
```

## Suggested fix

Change `run_idl2schemata` to create a fresh `known_names` for each
schema, matching Java's `toString(true)` behavior:

```rust
for schema in registry.schemas() {
    // Each schema gets its own known_names set, just as Java's
    // Schema.toString(true) creates a fresh HashSet for each call.
    let mut known_names = IndexSet::new();

    let json_value = schema_to_json(schema, &mut known_names, ...);
    // ...
}
```

The comment on line 176 ("Share `known_names` across schema iterations
so that types inlined inside a record on first encounter are emitted
as bare string references in subsequent `.avsc` files, matching Java
behavior") is incorrect -- it describes the opposite of Java's actual
behavior.
