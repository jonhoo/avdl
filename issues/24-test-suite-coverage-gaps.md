# Test suite coverage gaps

## Summary

The integration test suite has grown significantly but still has
notable gaps. This issue tracks the remaining coverage gaps, updated
after Wave 1 and Wave 2 fixes.

### What's been addressed

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

### Remaining gaps

---

## 1. Missing input files: `import.avdl` and `nestedimport.avdl`

**Priority: High** -- These are the only two `.avdl` files in `input/`
with no corresponding integration test. Both exercise the import
pipeline, which is the most complex and bug-prone part of the system.

`import.avdl` exercises every import kind (IDL, protocol, schema) in
a single protocol, including classpath-resolved imports. A correct
test requires passing both the `input/` and `putOnClassPath/`
directories as import search paths.

`nestedimport.avdl` exercises nested import chains with mixed import
kinds.

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

## 3. No tests for `putOnClassPath/` import resolution

**Priority: Medium** -- The `putOnClassPath/` directory contains files
that the Java test suite resolves via classpath. Our tool resolves them
via `--import-dir`. These files are exercised transitively by
`import.avdl` (gap #1), but there is no targeted test.

Key behaviors to test:
- Relative path resolution within imported IDL files (`../`
  traversal in `relativePath.avdl`)
- Import search path resolution (finding `OnTheClasspath.avdl` in a
  non-relative directory)
- Mixing relative and search-path resolution in the same import chain

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

## 6. Logical type field tests

**Priority: Low** -- `logicalTypes.avdl` is the dedicated stress test
for logical type handling but has no golden `.avpr` file. A unit-level
test that parses it and checks field-level logical type metadata
would be useful.

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
- `idl2schemata` output file count assertion
- Additional golden-file pairs in `tools/src/test/idl/` directory

---

## Priority summary

| # | Gap                                            | Priority |
|---|------------------------------------------------|----------|
| 1 | `import.avdl` and `nestedimport.avdl` tests   | High     |
| 2 | More `idl2schemata` tests                      | Medium   |
| 3 | `putOnClassPath/` import resolution tests      | Medium   |
| 4 | Import cycle detection test                    | Medium   |
| 5 | Doc comment and warning tests                  | Medium   |
| 6 | Logical type field tests                       | Low      |
| 7 | Second `cycle.avdl` variant                    | Low      |
| 8 | CLI-level integration tests                    | Low      |
| 9 | Java test behaviors (stderr, file count)       | Medium   |
