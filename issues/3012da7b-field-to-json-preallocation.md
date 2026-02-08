# Pre-allocate `serde_json::Map` in `field_to_json`

## Symptom

`field_to_json` at `json.rs:555` shows 3 samples of self-time
(~0.84%) in profiling on a 1 MB input. It is the leaf function with
the highest self-time in our code. With hundreds of fields across
multiplied record types, the per-field `Map` allocation adds up.

## Root cause

`field_to_json` creates a `Map::new()` (which starts empty with zero
capacity) and then inserts keys one at a time:

```rust
fn field_to_json(...) -> Value {
    let mut obj = Map::new();
    obj.insert("name".to_string(), ...);
    obj.insert("type".to_string(), ...);
    // optional: doc, default, order, aliases, properties
    ...
}
```

Every field has at least 2 keys (`name`, `type`), and most have 2–4
(adding `doc`, `default`, `order`). The `Map` (backed by
`BTreeMap<String, Value>`) reallocates its internal nodes as entries
are inserted. Pre-allocating with the expected capacity avoids
intermediate allocations.

Note: `serde_json::Map` wraps `BTreeMap` by default (not `HashMap`),
so `with_capacity` is not directly available. However, the same
principle applies to the other `Map::new()` call sites in
`schema_to_json` for records, enums, fixed, array, map, and logical
types.

## Affected files

- `src/model/json.rs`: `field_to_json` (line 555–598)
- `src/model/json.rs`: `schema_to_json` — similar pattern for named
  type objects

## Reproduction

No functional bug — observable under profiling. Look for
`field_to_json` self-time and `BTreeMap` allocation stacks.

## Suggested fix

1. **Use `serde_json`'s `preserve_order` feature.** When
   `preserve_order` is enabled, `serde_json::Map` is backed by
   `IndexMap` instead of `BTreeMap`, which supports
   `with_capacity()`. This would also naturally preserve insertion
   order (which we currently achieve via sorted keys). Enable the
   feature in `Cargo.toml`:

   ```toml
   serde_json = { version = "1", features = ["preserve_order"] }
   ```

   Then use `Map::with_capacity(4)` (or a computed count) in
   `field_to_json` and the named-type branches of `schema_to_json`.

2. **Alternatively, build a `Vec` and convert.** Collect key-value
   pairs into a pre-allocated `Vec<(String, Value)>` and convert to
   `Map` at the end. This avoids incremental insertion overhead
   regardless of the backing collection.

Option 1 is preferred because it also eliminates the need for
alphabetical key sorting (which we currently get from `BTreeMap`)
since we already insert keys in the correct order.
