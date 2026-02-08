# Remaining test coverage gaps

## Symptom

The integration test suite has grown substantially but still has two
notable gaps from the original tracking issue.

## Status

Re-opened during audit of deleted issues. Original issue: `24-test-suite-coverage-gaps.md`.

## Evidence of partial fix

The following gaps from the original issue have been resolved:

- Gap 1: `import.avdl` and `nestedimport.avdl` tests added.
- Gap 2: `idl2schemata` tests for `interop.avdl` and `import.avdl` added.
- Gap 3: `putOnClassPath/` import resolution covered by `test_import`.
- Gap 4: Import cycle detection tests (`test_self_import_cycle_handled_gracefully`,
  `test_mutual_import_cycle_handled_gracefully`) added.
- Gap 5: Doc comment warning test (`test_comments_warnings_count`) added.
- Gap 6: Logical type propagation tests added.
- Gap 6b: `extra/` directory tests added.
- Gap 9b: Tools golden files (`test_tools_schema`, `test_tools_protocol`) added.
- Gap 9 partial: `idl2schemata` file count assertion (`test_idl2schemata_tools_protocol`) added.
- Workspace path: `test_workspace_path` added.

## Remaining work

### Gap 7: Second `cycle.avdl` variant -- RESOLVED

Resolved: `test_cycle_test_root` added to `tests/integration.rs`.

### Gap 8: CLI-level integration tests (Low priority, deferred)

All existing integration tests call the library API directly. No tests
exercise the CLI binary, including stdin/stdout piping, `--import-dir`
flag parsing, error output formatting, or exit codes. This means the
broken-pipe fix in `write_output` and the `idl2schemata` required-arg
change have no dedicated regression tests.

This is deferred as low priority. The library-level integration tests
cover the parsing and serialization logic thoroughly, and the
`compare-golden.sh` script exercises the CLI end-to-end.

### Gap 10: `logicalTypes.avdl` not in integration tests (Low priority)

Java's `TestLogicalTypes` exercises
`avro/lang/java/idl/src/test/idl/logicalTypes.avdl`, which covers all
built-in logical type keywords (`date`, `time_ms`, `timestamp_ms`,
`local_timestamp_ms`, `uuid`, `decimal`), custom `@logicalType`
annotations (`timestamp-micros`), and an oversized-precision edge case
(`@precision(3000000000)`). Our inline tests
(`test_builtin_logical_types_propagate_to_json`,
`test_custom_logical_type_annotation_propagates_to_json`) cover the
same patterns, but there is no integration test against the actual
file. CLI comparison confirms identical output to Java.

### Gap 11: Warning assertions for tools test files (Low priority)

Java's `TestIdlTool` and `TestIdlToSchemataTool` assert that a
specific out-of-place doc comment warning is emitted when processing
`tools/protocol.avdl` and `tools/schema.avdl`. Our `test_tools_*`
integration tests only verify JSON output, not warnings. The
warnings are emitted correctly by the CLI (verified manually).

### Gap 12: `AnnotationOnTypeReference.avdl` not tested at file level (Low priority)

Java's `TestReferenceAnnotationNotAllowed` parses
`AnnotationOnTypeReference.avdl` and asserts it's rejected with
"Type references may not be annotated". Our unit test
`annotation_on_type_reference_is_rejected` covers the same behavior
with inline input, but does not test the actual file. CLI test
confirms correct rejection.

## Java test suite audit (2026-02-08)

Comprehensive audit of all Java test files:

- `TestIdlReader.java`: 4 tests. `runTests` (18 golden files) -- fully
  covered. `validateProtocolParsingResult` and `validateSchemaParsingResult`
  (`extra/` files) -- covered by `test_extra_protocol_syntax` and
  `test_extra_schema_syntax`. `testDocCommentsAndWarnings` -- covered by
  `test_comments_warnings_count` with exact position matching.
- `TestIdlTool.java`: 3 tests. `testWriteIdlAsSchema` and
  `writeIdlAsProtocol` -- JSON output covered by `test_tools_schema` and
  `test_tools_protocol`; warning assertions are Gap 11.
  `testWriteIdlAsProtocolUsingJavaCC` -- out of scope (old parser).
- `TestIdlToSchemataTool.java`: 2 tests. `splitIdlIntoSchemata` -- covered
  by `test_idl2schemata_tools_protocol`; warning assertions are Gap 11.
  `testSplitIdlIntoSchemataUsingJavaCC` -- out of scope (old parser).
- `TestLogicalTypes.java`: 8 tests -- equivalent inline tests exist
  (Gap 10).
- `TestReferenceAnnotationNotAllowed.java`: 1 test -- equivalent unit
  test exists (Gap 12).
- `TestCycle.java`: 1 test -- binary ser/deser is out of scope;
  parsing covered by `test_cycle_test_root`.
- `IdlUtilsTest.java`: IDL round-trip writing -- out of scope.
- `IdlSchemaFormatterFactoryTest.java`: IDL formatting -- out of scope.

Additional .avdl files from other Java modules (`grpc/TestService.avdl`,
`maven-plugin/User.avdl`, `integration-test/custom_conversion_idl.avdl`)
were verified to produce identical output to Java via `compare-adhoc.sh`.

No correctness bugs were found during this audit.

## Affected files

- `tests/integration.rs`

## Reproduction

Gap 8: No test exercises `cargo run -- idl` or `cargo run -- idl2schemata`
as a subprocess to verify CLI behavior (exit codes, error formatting,
pipe handling).

Gaps 10-12: Run the relevant file through `scripts/compare-adhoc.sh` to
verify output matches Java. All three produce correct output.
