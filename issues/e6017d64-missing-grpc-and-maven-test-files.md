# Add `.avdl` files from other Java subprojects to integration tests

## Symptom

Several `.avdl` files from other Java subprojects are not part of our
test suite, even though they exercise useful feature combinations and
all pass when tested with `compare-adhoc.sh`:

1. `avro/lang/java/grpc/src/test/avro/TestService.avdl`
   - Union return type: `union {null, string} concatenate(...)`
   - `void` method with declared thrown error
   - `oneway` message
   - Fixed type with small size (`MD5(4)`)
   - Three-argument message

2. `avro/lang/java/maven-plugin/src/test/avro/User.avdl`
   - `union { null, IdlPrivacy }` field
   - `timestamp_ms` logical type as a bare field type
   - Enum with mixed-case symbols (`Public`, `Private`)

3. `avro/lang/java/integration-test/codegen-test/src/test/resources/avro/custom_conversion_idl.avdl`
   - `decimal(9,2)` inside nullable union
   - `@logicalType("fixed-size-string")` with `@minLength`/`@maxLength`
     on `bytes` inside nullable union
   - Non-null custom logical type field

## Root cause

These files are in subprojects outside the main `idl/` and `tools/`
test directories, so they were not discovered during the initial test
port.

## Reproduction

All pass:
```sh
scripts/compare-adhoc.sh avro/lang/java/grpc/src/test/avro/TestService.avdl
scripts/compare-adhoc.sh avro/lang/java/maven-plugin/src/test/avro/User.avdl
scripts/compare-adhoc.sh avro/lang/java/integration-test/codegen-test/src/test/resources/avro/custom_conversion_idl.avdl
```

## Suggested fix

Add integration tests (similar to `test_extra_protocol_syntax`) that
compile these files through `Idl` and spot-check key output properties.
Golden file comparison is not necessary since these files are covered
by the `compare-adhoc.sh` pass, but having them in `cargo test` guards
against future regressions.

Priority: Low. These features are already exercised by existing golden
file tests (`simple.avdl`, `union.avdl`, `interop.avdl`, etc.), so
this is for defense-in-depth only.
