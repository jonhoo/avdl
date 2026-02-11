# `AvroSchema::with_merged_properties` most variants untested

- **Symptom**: The `with_merged_properties` method on `AvroSchema` (lines
  403-445 of `model/schema.rs`) has many match arms at 0% coverage:
  - `Record` (lines 410-412)
  - `Enum` (lines 414-416)
  - `Fixed` (lines 418-420)
  - `Map` (lines 426-428)
  - `AnnotatedPrimitive` (lines 434-436)
  - `Reference` (lines 438-440)

  Only the `Array` and `Logical` arms (and the bare-primitive fast path)
  are exercised. Similarly, the `TimeMicros` and `LocalTimestampMicros`
  serialization branches in `model/json.rs` (lines 439-445, 467-473)
  are uncovered.

- **Root cause**: The IDL syntax rarely produces annotations directly on
  these schema types. Annotations on records/enums/fixed are typically
  handled at parse time, not via `with_merged_properties`. The `micros`
  logical types are not used in the golden test suite `.avdl` files
  (only `millis` variants and `timestamp-micros` via `@logicalType`
  annotation appear).

- **Affected files**: `src/model/schema.rs` (lines 410-440),
  `src/model/json.rs` (lines 439-445, 467-473)

- **Reproduction**: Run `cargo llvm-cov --text` and observe hit count 0
  on the variant arms listed above.

- **Suggested fix**:
  1. Add unit tests for `with_merged_properties` that construct each
     variant directly and verify properties are merged correctly.
  2. Add a test with IDL using `time_us` and `local_timestamp_us`
     built-in logical types to cover the `TimeMicros` and
     `LocalTimestampMicros` JSON serialization paths.
