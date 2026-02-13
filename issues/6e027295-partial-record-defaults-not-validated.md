# Partial record defaults are not validated

## Symptom

Rust accepts record default values that omit required fields (fields without
their own defaults), producing invalid Avro protocol JSON. Java correctly
rejects such invalid defaults with a parse error.

## Root cause

The Rust implementation does not validate that record default values provide
values for all required fields. It simply passes through the JSON object
literal from the IDL source without checking completeness.

## Affected files

- `src/reader.rs` - The code that handles record field defaults
- `src/resolve.rs` - Could potentially add validation during schema resolution

## Reproduction

```avdl
// tmp/edge-record-default-bad-partial.avdl
@namespace("test.record.defaults")
protocol BadPartialTest {
  record Inner {
    string name;
    int value;  // No default - this field is required
  }

  record Outer {
    // This should fail: Inner.value is required but not provided
    Inner partial = {"name": "partial"};
  }
}
```

```sh
# Rust: succeeds but produces invalid JSON
cargo run -- idl tmp/edge-record-default-bad-partial.avdl
# Output includes: "default": {"name": "partial"} which is incomplete

# Java: correctly rejects
java -jar avro-tools-1.12.1.jar idl tmp/edge-record-default-bad-partial.avdl
# Exception: java.util.NoSuchElementException
```

## Suggested fix

After parsing a record default value, validate that:
1. All fields of the record type that lack defaults have values provided in the
   default object
2. The types of provided values match the expected field types

This validation should occur during semantic analysis, after the full schema
registry is populated (so we can look up the record type definition).
