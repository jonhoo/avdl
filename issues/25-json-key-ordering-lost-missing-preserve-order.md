# JSON key ordering lost due to missing `preserve_order` feature on `serde_json`

## Symptom

All JSON output has keys sorted alphabetically instead of in the
order specified by the Avro specification and the Java tools. For
example, compiling `status_schema.avdl` produces:

```json
{
  "default": "UNKNOWN",
  "name": "Status",
  "namespace": "system",
  "symbols": [...],
  "type": "enum"
}
```

The golden file `status.avsc` (and the Java tools) produce keys in
this order:

```json
{
  "type": "enum",
  "name": "Status",
  "namespace": "system",
  "symbols": [...],
  "default": "UNKNOWN"
}
```

This affects every output type: records have `fields, name, namespace,
type` instead of `type, name, namespace, fields`; fields have `doc,
name, type` instead of `name, type, doc`; arrays have `items, type`
instead of `type, items`; and so on.

## Root cause

In `src/model/json.rs`, the serialization functions carefully insert
keys into an `IndexMap` in the correct order (matching the Java
tools). For example, the enum serialization at line 273 inserts
`type`, then `name`, then `namespace`, etc.

However, the `indexmap_to_value` function at line 619 converts the
`IndexMap` to a `serde_json::Map<String, Value>`:

```rust
fn indexmap_to_value(map: IndexMap<String, Value>) -> Value {
    let json_map: Map<String, Value> = map.into_iter().collect();
    Value::Object(json_map)
}
```

Without the `preserve_order` feature enabled on `serde_json`,
`Map<String, Value>` is backed by a `BTreeMap<String, Value>`, which
sorts keys alphabetically. This destroys the carefully constructed
insertion order.

In `Cargo.toml`, the `serde_json` dependency is:

```toml
serde_json = "1"
```

It does not enable `preserve_order`.

## Affected files

- `Cargo.toml` -- missing `features = ["preserve_order"]` on
  `serde_json`
- `src/model/json.rs:619-622` -- `indexmap_to_value` relies on
  insertion-ordered `Map` but gets `BTreeMap` behavior instead

## Reproduction

```sh
cargo run -- idl avro/lang/java/idl/src/test/idl/input/status_schema.avdl
# Keys appear as: default, name, namespace, symbols, type
# Expected:       type, name, namespace, symbols, default
```

This affects all outputs, including `schema_syntax_schema.avdl` and
every protocol file.

## Suggested fix

Enable the `preserve_order` feature on `serde_json` in `Cargo.toml`:

```toml
serde_json = { version = "1", features = ["preserve_order"] }
```

With this feature, `serde_json::Map` is backed by an `IndexMap`
instead of a `BTreeMap`, so insertion order is preserved. No code
changes are needed in `json.rs` -- the `indexmap_to_value` function
will automatically preserve the order it already carefully constructs.
