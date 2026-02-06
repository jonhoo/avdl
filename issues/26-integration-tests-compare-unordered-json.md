# Integration tests compare unordered JSON, masking key-ordering bugs

## Summary

The Rust integration tests compare parsed `serde_json::Value` trees,
which use `BTreeMap` internally and therefore normalize JSON key order.
This means the tests cannot detect key-ordering regressions, even
though the Java test suite's comparison is order-sensitive.

## Java comparison method

In `TestIdlReader.GenTest.run()`, both the expected output and the
actual output are parsed through Jackson's `ObjectMapper.readTree()`
and then re-serialized via `ObjectMapper.writer().writeValueAsString()`.
The comparison uses `assertEquals(slurped.trim(), output.trim())` --
a **string comparison** of the re-serialized JSON.

Jackson's `ObjectMapper` preserves insertion order in JSON objects by
default (using `LinkedHashMap`), so this comparison is sensitive to
key ordering. If the Java IDL compiler emits keys in the wrong order,
the test fails.

## Rust comparison method

In `tests/integration.rs`, the expected output is loaded via
`serde_json::from_str` into a `serde_json::Value`, and the actual
output is built as a `serde_json::Value` via `protocol_to_json` /
`schema_to_json`. The comparison uses `assert_eq!(actual, expected)`.

Because `serde_json::Value::Object` is backed by `BTreeMap<String,
Value>` (without the `preserve_order` feature), all JSON object keys
are sorted alphabetically before comparison. This means:

- A bug that emits `{"name": "X", "type": "record"}` instead of the
  correct `{"type": "record", "name": "X"}` will NOT be detected.
- Issue #25 documents that key ordering is currently wrong, but even
  after fixing #25, there would be no test to prevent regressions.

## Relationship to other issues

- **Issue #25**: Documents that `serde_json` lacks `preserve_order`
  and keys are sorted. Fixing #25 by enabling `preserve_order` would
  also fix this testing gap, because `serde_json::Value::Object`
  would then use `IndexMap` and preserve insertion order during both
  deserialization and comparison.
- **Issue #24, section 2**: Documents that `test_status_schema` uses
  workarounds, but does not mention the broader comparison weakness.

## Suggested fix

Once issue #25 is fixed (enabling `preserve_order` on `serde_json`),
the integration tests will automatically become order-sensitive
because `serde_json::Value` will use `IndexMap` instead of
`BTreeMap`. No further test changes would be needed.

However, if #25 is deferred, the tests could be strengthened
independently by comparing serialized JSON strings (after
pretty-printing both sides) instead of parsed `Value` trees. This
would catch ordering bugs even without `preserve_order`:

```rust
let actual_str = serde_json::to_string_pretty(&actual).unwrap();
let expected_str = serde_json::to_string_pretty(&expected).unwrap();
assert_eq!(actual_str, expected_str);
```

Note: this only works if the expected golden files are also
pretty-printed in a consistent format. Since they come from the Java
tools (which use a different pretty-printer), minor whitespace
differences would need to be normalized.
