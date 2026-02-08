# Test suite coverage gaps

## Summary

The integration test suite has grown significantly but still has
notable gaps. This issue tracks the remaining coverage gaps, updated
after Wave 1 and Wave 2 fixes.

### What's been addressed

- **Gap 1** (`import.avdl` and `nestedimport.avdl`): Fixed. Added
  `test_import` and `test_nestedimport` integration tests.
- **Gap 2** (`test_status_schema` workarounds): Fixed. The test now
  uses direct assertion with no manual array wrapping or error
  swallowing.
- **Gap 3** (`idl2schemata` tests): Partially addressed. Added
  `test_idl2schemata_echo` and `test_idl2schemata_simple`. Still
  missing tests for `interop` and import-heavy files.
- **Gap 5** (negative/error-case tests): Partially addressed. Added
  `test_nested_union_rejected`, `test_reserved_type_name_rejected`,
  `test_duplicate_type_definition`, `test_import_nonexistent_file`,
  plus unit tests for `annotation_on_type_reference_is_rejected`,
  `oneway_nonvoid_return_is_rejected`, and `oneway_void_return_is_accepted`.
- **Gap 6** (`extra/` directory tests): Addressed. Added
  `test_extra_protocol_syntax` and `test_extra_schema_syntax`.
- **Gap 6b** (logical type tests): Fixed. Added
  `test_builtin_logical_types_propagate_to_json`,
  `test_custom_logical_type_annotation_propagates_to_json`,
  `test_custom_logical_type_with_additional_annotations`, and
  `test_builtin_logical_type_with_custom_annotation`.
- **Gap 9b** (tools golden files): Fixed. Added `test_tools_schema`
  and `test_tools_protocol` covering `tools/src/test/idl/`.
- **Workspace path**: Fixed. Added `test_workspace_path` covering
  the AVRO-3706 edge case from `compiler/src/test/idl/work space/`.

### Remaining gaps

---

## ~~1. Missing input files: `import.avdl` and `nestedimport.avdl`~~

**RESOLVED.** Added `test_import` and `test_nestedimport`.

---

## 2. More `idl2schemata` integration tests

**Priority: Medium** -- `test_idl2schemata_echo` and
`test_idl2schemata_simple` cover the basic pipeline. Still needed:

- `idl2schemata` for `interop.avdl`: Multiple record, enum, and
  fixed types. Verifies correct handling of self-referential records
  (`Node` contains `array<Node>`).
- `idl2schemata` for a file with imports (once import tests work):
  Verifies that imported types are also written as individual `.avsc`
  files.

---

## ~~3. No tests for `putOnClassPath/` import resolution~~

**RESOLVED** — covered transitively by `test_import`, which passes
both `input/` and `putOnClassPath/` as import directories and
exercises relative path resolution, search-path resolution, and
mixed resolution in the same import chain.

---

## 4. Import cycle detection test

**Priority: Medium** -- The `ImportContext` has cycle prevention
logic, but no test verifies that parsing a file that imports itself
(or two files that import each other) produces a graceful result
rather than an infinite loop or stack overflow.

---

## 5. Doc comment and warning tests

**Priority: Medium** -- Java's `testDocCommentsAndWarnings` test
asserts 24 specific warnings with line/column positions for misplaced
doc comments. Our parser silently discards them (tracked separately
in `missing-doc-comment-warnings.md`). Once warning infrastructure
is added, tests should be added here.

---

## ~~6. Logical type field tests~~

**RESOLVED.** Added 4 logical type propagation tests covering
built-in keywords, custom `@logicalType` annotations, combined
annotations, and built-ins with custom annotations.

---

## 7. Second `cycle.avdl` variant

**Priority: Low** -- The test-root `cycle.avdl` (with
`Record1`/`Record2`/`Record3`) is different from `input/cycle.avdl`
and is not covered by our `test_cycle` integration test. It has more
complex cycles (three records referencing each other).

---

## 8. CLI-level integration tests

**Priority: Low** -- All existing integration tests call the library
API directly. There are no tests exercising the CLI binary, including
stdin/stdout piping, `--import-dir` flag parsing, error output
formatting, or exit codes.

---

## 9. Java test behaviors not yet mirrored

**Priority: Medium** -- From `TestIdlTool` and `TestIdlToSchemataTool`:

- Stderr warning assertions (e.g., license-header doc comment warning)
  -- both `TestIdlTool.testWriteIdlAsSchema` and
  `TestIdlToSchemataTool.splitIdlIntoSchemata` assert that a specific
  "Ignoring out-of-place documentation comment" warning appears on
  stderr. Depends on implementing the warning system (gap #5).
- `idl2schemata` output file count assertion -- Java's
  `TestIdlToSchemataTool.splitIdlIntoSchemata` asserts that
  `tools/protocol.avdl` produces exactly 4 `.avsc` files
  (Kind, MD5, TestRecord, TestError). The Rust tool produces the
  correct count (verified manually), but no Rust test asserts this.
- ~~Additional golden-file pairs in `tools/src/test/idl/` directory~~
  DONE (`test_tools_schema`, `test_tools_protocol`)

From `TestLogicalTypes` (compiler module):

- Invalid decimal precision overflow test -- Java's
  `incorrectlyAnnotatedBytesFieldHasNoLogicalType` asserts that
  `@precision(3000000000)` does NOT produce a promoted logical type
  because the value exceeds `Integer.MAX_VALUE`. The Rust tool
  currently promotes it as valid. Tracked in issue `b638adba`.

---

## Priority summary

| # | Gap                                            | Priority |
|---|------------------------------------------------|----------|
| 1 | ~~`import.avdl` and `nestedimport.avdl` tests~~ | ~~High~~ DONE |
| 2 | More `idl2schemata` tests                      | Medium   |
| 3 | ~~`putOnClassPath/` import resolution tests~~  | ~~Medium~~ (covered by #1) |
| 4 | Import cycle detection test                    | Medium   |
| 5 | Doc comment and warning tests                  | Medium   |
| 6 | ~~Logical type field tests~~                   | ~~Low~~ DONE |
| 7 | Second `cycle.avdl` variant                    | Low      |
| 8 | CLI-level integration tests                    | Low      |
| 9 | Java test behaviors (stderr, file count, precision) | Medium (tools golden DONE) |
