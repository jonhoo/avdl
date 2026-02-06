# idl2schemata omits namespace on schemas that inherit from protocol

## Symptom

When extracting individual `.avsc` files via `idl2schemata`, schemas
that inherit their namespace from the enclosing protocol are missing
the `"namespace"` key in the output JSON. Since each `.avsc` file is
standalone (no enclosing protocol context), the namespace must be
included explicitly for the schema to be valid.

For example, `echo.avdl` has `@namespace("org.apache.avro.echo")` on
the protocol, and both `Ping` and `Pong` records inherit this
namespace. The Rust `idl2schemata` output for `Ping.avsc` is:

```json
{
  "type": "record",
  "name": "Ping",
  "fields": [...]
}
```

The correct output (matching Java) should be:

```json
{
  "type": "record",
  "name": "Ping",
  "namespace": "org.apache.avro.echo",
  "fields": [...]
}
```

This affects every idl2schemata file where the type inherits the
protocol namespace. The `compare-golden.sh idl2schemata` script
shows 14 failures out of 18 tested schema files.

Types with an explicit `@namespace` annotation that differs from the
protocol namespace DO get their namespace correctly (e.g.,
`EnumInOtherNamespace.avsc` from `namespaces.avdl` correctly has
`"namespace": "avro.test.enum"`).

## Root cause

In `src/main.rs` `run_idl2schemata()` (line 194), each schema is
serialized via:

```rust
let json_value = schema_to_json(
    schema,
    &mut known_names,
    namespace.as_deref(),  // <-- protocol namespace
    &all_lookup,
);
```

The `enclosing_namespace` parameter is set to the protocol's
namespace. The `schema_to_json` function in `src/model/json.rs`
(lines 219-223 for records, similar for enum/fixed) suppresses the
`"namespace"` key when it matches `enclosing_namespace`:

```rust
if namespace.as_deref() != enclosing_namespace
    && let Some(ns) = namespace
{
    obj.insert("namespace".to_string(), Value::String(ns.clone()));
}
```

This is correct for `idl` mode (types inside a protocol omit
redundant namespace), but wrong for `idl2schemata` mode (standalone
schemas need explicit namespace).

## Affected files

- `src/main.rs` -- `run_idl2schemata()`, line 194
- `src/model/json.rs` -- `schema_to_json()`, namespace suppression
  logic at lines 219-223, 282-285, 338-342

## Reproduction

```sh
mkdir -p tmp/i2s-test
cargo run -- idl2schemata avro/lang/java/idl/src/test/idl/input/echo.avdl tmp/i2s-test/
jq . tmp/i2s-test/Ping.avsc
# "namespace" key is missing

scripts/compare-golden.sh idl2schemata
# Shows 14 failures
```

## Suggested fix

In `run_idl2schemata()`, pass `None` instead of the protocol
namespace as the `enclosing_namespace` parameter to `schema_to_json`.
This will cause all schemas to include their namespace explicitly:

```rust
let json_value = schema_to_json(schema, &mut known_names, None, &all_lookup);
```

However, this may break the namespace-shortening for type references
*within* each schema (e.g., a field referencing another type in the
same namespace should use the short name). The correct approach may
be to pass `None` as `enclosing_namespace` for the top-level schema
serialization but still use the protocol namespace for reference
shortening within fields. This requires either:

1. A separate parameter for "should this top-level schema emit its
   namespace" vs "what namespace should be used for reference
   shortening", or
2. Passing `None` for enclosing namespace (to emit namespace) and
   relying on the schema's own namespace for internal reference
   shortening (which `field_to_json` already does via
   `namespace.as_deref().or(enclosing_namespace)`).

Option 2 is likely sufficient: the schema's own namespace field is
used as enclosing namespace for its fields, so internal references
will still be shortened correctly.
