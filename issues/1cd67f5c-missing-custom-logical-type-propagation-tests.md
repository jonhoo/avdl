# Missing tests for custom logical type propagation to JSON output

## Symptom

There are no tests verifying that custom logical types (e.g.,
`@logicalType("timestamp-millis") long`) are correctly propagated
to the JSON output. While the `interop.avdl` golden file comparison
implicitly covers built-in logical types (`date`, `time_ms`,
`timestamp_ms`, `decimal`), there are no tests specifically for:

- Custom/user-defined logical type annotations
- The `logicalType` key appearing correctly in the JSON output
- Interaction between `@logicalType` and other annotations
- Invalid or unknown logical type strings

## Root cause

The existing test suite relies on golden file comparison, which only
covers the logical types present in the Avro test `.avdl` files.
There are no targeted unit or integration tests for the logical type
serialization path.

## Affected files

- `src/model/json.rs` — serialization of `logicalType` key
- `src/reader.rs` — parsing of `@logicalType` annotations
- `tests/integration.rs` — missing test cases

## Reproduction

No test currently exercises:
```avdl
protocol LogicalTypes {
  record Event {
    @logicalType("timestamp-millis") long created_at;
    @logicalType("uuid") string id;
    @logicalType("custom-type") bytes payload;
  }
}
```

## Suggested fix

Add integration tests (or unit tests in `json.rs`) that:
1. Parse `.avdl` with `@logicalType` annotations on various base types
2. Verify the JSON output contains `{"type": "long", "logicalType": "timestamp-millis"}`
3. Test that unknown/custom logical type strings are preserved as-is
4. Test interaction with other annotations (`@logicalType` + `@order`, etc.)

Priority: Medium
