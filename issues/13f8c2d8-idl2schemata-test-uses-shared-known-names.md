# Integration test `parse_idl2schemata` uses shared `known_names` (wrong behavior)

## Symptom

The `parse_idl2schemata` helper function in `tests/integration.rs`
shares a single `known_names: IndexSet<String>` across all schema
serialization iterations. This causes schemas that were already
inlined inside a previous schema (e.g., `MD5` inside `TestRecord`)
to be serialized as bare name strings instead of full inline
definitions in their standalone `.avsc` output.

For example, `test_idl2schemata_simple` asserts:

```rust
let md5 = &schemata["MD5"];
assert_eq!(
    *md5,
    serde_json::Value::String("MD5".to_string()),
    "MD5 was already inlined in TestRecord, so its standalone entry should be a bare name"
);
```

But the correct behavior (matching Java) is for each schema to be
fully self-contained with all referenced types inlined. The Rust
CLI (`main.rs`) correctly uses fresh `known_names` per schema:

```rust
// main.rs line 171
let mut known_names = IndexSet::new();
```

The CLI produces the correct output:

```json
{
  "type": "fixed",
  "name": "MD5",
  "namespace": "org.apache.avro.test",
  "doc": "An MD5 hash.",
  "size": 16
}
```

## Root cause

The `parse_idl2schemata` function in `tests/integration.rs` (line
503) declares `let mut known_names = IndexSet::new()` outside the
loop, sharing it across iterations. The stale comment on lines
499-500 says "sharing `known_names` across iterations to match the
behavior of `run_idl2schemata` in main.rs", but `main.rs` was
updated to use fresh `known_names` per schema (matching Java's
`Schema.toString(true)` which creates a fresh `HashSet` per call).
The test helper was not updated to match.

## Affected files

- `tests/integration.rs` -- `parse_idl2schemata` function (line
  ~503) and `test_idl2schemata_simple` assertions (line ~593-598)

## Reproduction

```sh
# The CLI produces correct output (full inline MD5):
cargo run -- idl2schemata avro/lang/java/idl/src/test/idl/input/simple.avdl tmp/simple-split/
cat tmp/simple-split/MD5.avsc
# {"type": "fixed", "name": "MD5", "namespace": "org.apache.avro.test", ...}

# But the integration test passes because it tests the wrong behavior:
cargo test test_idl2schemata_simple
# Passes, but asserts MD5 should be a bare string "MD5"
```

## Suggested fix

1. Move `let mut known_names = IndexSet::new()` inside the loop in
   `parse_idl2schemata`, matching `main.rs`.

2. Update `test_idl2schemata_simple` to assert that `MD5` is a full
   inline definition `{"type": "fixed", "name": "MD5", ...}` instead
   of a bare string.

3. Update the stale comment on lines 499-500.

## Priority

Medium. The CLI behavior is correct; this is a test correctness
issue. The test currently validates the wrong expected output, which
means it would not catch a regression if `main.rs` were changed back
to shared `known_names`.
