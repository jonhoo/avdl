# Add `idl_utils_test_protocol.avdl` and `idl_utils_test_schema.avdl` to the test suite

## Symptom

The Java `IdlUtilsTest` class tests two `.avdl` files that exercise a
rich combination of features in a single file, but our Rust test suite
does not include them:

- `avro/lang/java/idl/src/test/resources/org/apache/avro/util/idl_utils_test_protocol.avdl`
- `avro/lang/java/idl/src/test/resources/org/apache/avro/util/idl_utils_test_schema.avdl`

Both files pass when run through `compare-adhoc.sh` (both `idl` and
`idl2schemata` modes), so there is no behavioral gap — only a test
coverage gap.

## Root cause

These files live under the Java `resources/` directory (for classpath
loading) rather than in `test/idl/input/`, so they were not picked up
by the golden-file test sweep.

## Features exercised

These files are valuable integration smoke tests because they combine
many features in a single file:

- `@version()` annotation with numeric value (not string)
- `@aliases()` on record definitions
- `@generator()` annotation on field type position
- Hyphenated annotation key (`@my-key("my-value")`)
- `@order("DESCENDING")` on a `map<>` type
- Cross-namespace field references (`common.Flag`)
- `oneway` message with `null` return type
- `throws` clause on messages
- `union{null, @my-key("my-value") array<Counter>}` — annotations
  inside union type
- `decimal(12,3)` and `@logicalType("time-micros") long`
- Schema mode with `namespace` and `schema` directives

## Reproduction

Both pass:
```sh
scripts/compare-adhoc.sh \
  avro/lang/java/idl/src/test/resources/org/apache/avro/util/idl_utils_test_protocol.avdl
scripts/compare-adhoc.sh \
  avro/lang/java/idl/src/test/resources/org/apache/avro/util/idl_utils_test_schema.avdl
scripts/compare-adhoc.sh --idl2schemata \
  avro/lang/java/idl/src/test/resources/org/apache/avro/util/idl_utils_test_protocol.avdl
scripts/compare-adhoc.sh --idl2schemata \
  avro/lang/java/idl/src/test/resources/org/apache/avro/util/idl_utils_test_schema.avdl
```

## Suggested fix

Add integration tests in `tests/integration.rs` that compile both
files through `Idl` and `Idl2Schemata` and compare against golden
files, similar to the existing `test_extra_protocol_syntax` and
`test_extra_schema_syntax` tests.
