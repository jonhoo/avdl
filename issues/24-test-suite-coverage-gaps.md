# Test suite coverage gaps

## Summary

The integration test suite covers 16 of the 18 `.avdl` input files but
is missing tests for the two import-heavy files (`import.avdl`,
`nestedimport.avdl`). Beyond those, there are no tests at all for the
`idl2schemata` subcommand, no tests for `putOnClassPath/` import
resolution, no negative/error-case tests, and no coverage of the
`extra/` test inputs or the Java-specific test resources
(`logicalTypes.avdl`, `AnnotationOnTypeReference.avdl`). The
`test_status_schema` test silently passes on parse failure and uses
manual array-wrapping rather than asserting the correct output format.

The gaps are organized below by category, with the most impactful gaps
listed first.

---

## 1. Missing input files: `import.avdl` and `nestedimport.avdl`

**Priority: High** -- These are the only two `.avdl` files in `input/`
with no corresponding integration test. Both exercise the import
pipeline, which is the most complex and bug-prone part of the system.

### 1a. `import.avdl` (expected output: `import.avpr`)

This file exercises every import kind in a single protocol:

- `import idl "reservedwords.avdl"` -- IDL import (recursive parsing)
- `import idl "nestedimport.avdl"` -- IDL import with its own nested
  imports
- `import idl "OnTheClasspath.avdl"` -- IDL import resolved via search
  path (not relative path)
- `import protocol "OnTheClasspath.avpr"` -- protocol import from
  search path
- `import schema "OnTheClasspath.avsc"` -- schema import from search
  path
- `import schema "baz.avsc"` -- schema import with fully-qualified
  `name` field (`ns.other.schema.Baz`)
- `import schema "foo.avsc"` -- schema import with fully-qualified
  `name` field (`org.foo.Foo`)
- `import protocol "bar.avpr"` -- protocol import

A correct test requires passing both the `input/` directory and the
`putOnClassPath/` directory as import search paths. It validates:

- Type ordering (imported types before local types, see issue #21)
- Qualified name splitting in `.avsc` imports (see issue #19)
- Message merging from imported protocols (the expected output includes
  messages `error`, `void`, `idl`, `import`, `oneway`, `null`,
  `local_timestamp_ms`, `bar` from `reservedwords.avdl` via transitive
  IDL import)
- Cross-protocol type references (`ns.other.schema.Baz`, `Foo`)

This single test would exercise three existing known issues (#19, #21,
and partially #22), making it a high-value regression target.

### 1b. `nestedimport.avdl` (expected output: `nestedimport.avpr`)

This file exercises nested import chains:

- `import idl "reservedwords.avdl"` -- IDL import
- `import protocol "bar.avpr"` -- protocol import
- `import schema "position.avsc"` -- schema import
- `import schema "player.avsc"` -- schema import (references
  `Position` from `position.avsc`)

It validates that the merged type list includes types from all import
kinds in the correct order, and that the protocol's `@version`
annotation propagates correctly to the output.

---

## 2. `test_status_schema` uses workarounds instead of strict assertions

**Priority: High** -- This test silently passes even when parsing fails.

The current `test_status_schema` implementation:

1. Catches parse errors and prints a note to stderr instead of failing
   the test (lines 421-431 in `integration.rs`).
2. If parsing succeeds, it manually wraps the single schema in an array
   to match the expected `status.avsc` output (lines 409-416).

Both behaviors mask real bugs. Issue #20 tracks the root cause (schema
mode without `schema` keyword should output an array of all named
schemas), but the test should either:

- Assert the correct output format (a JSON array) and fail if it does
  not match, or
- Be marked `#[ignore]` with a comment referencing issue #20, so the
  failure is visible in `cargo test -- --ignored`.

The current approach means this test will continue to "pass" even if the
underlying bug regresses further.

---

## 3. No `idl2schemata` integration tests

**Priority: High** -- The `idl2schemata` subcommand has zero integration
tests. Issue #22 documents known bugs in its serialization, but there is
no test to detect regressions or verify fixes.

### Recommended tests

**3a. `idl2schemata` for `simple.avdl`**: Parse `simple.avdl` through
`idl2schemata`, collect the output `.avsc` files, and verify:

- Correct file names: `Kind.avsc`, `Status.avsc`, `TestRecord.avsc`,
  `MD5.avsc`, `TestError.avsc`.
- Each file's JSON content matches what the Java tool produces.
- Type references within each file use short names when in the same
  namespace.

**3b. `idl2schemata` for `echo.avdl`**: Simpler protocol with one
record type (`Ping`). Verifies the basic pipeline works end-to-end.

**3c. `idl2schemata` for `interop.avdl`**: Multiple record, enum, and
fixed types. Verifies correct handling of self-referential records
(`Node` contains `array<Node>`).

**3d. `idl2schemata` for a file with imports** (once import tests
work): Verifies that imported types are also written as individual
`.avsc` files.

---

## 4. No tests for `putOnClassPath/` import resolution

**Priority: Medium** -- The `putOnClassPath/` directory contains files
that the Java test suite resolves via classpath. Our tool resolves them
via `--import-dir`. These files are exercised transitively by
`import.avdl`, but there is no targeted test.

### Files not tested

- `OnTheClasspath.avdl` -- defines protocol `OnTheClasspath` with
  `import idl "folder/relativePath.avdl"` and a record `FromAfar`
- `OnTheClasspath.avpr` -- protocol import target with record `VeryFar`
- `OnTheClasspath.avsc` -- schema import target with record `FarAway`
- `nestedtypes.avdl` -- defines `NestedType` record
- `folder/relativePath.avdl` -- imports `../nestedtypes.avdl` via
  relative path traversal

### What this would exercise

- Relative path resolution within imported IDL files (the `../`
  traversal in `relativePath.avdl`)
- Import search path resolution (finding `OnTheClasspath.avdl` in a
  non-relative directory)
- Mixing relative and search-path resolution in the same import chain

These are covered implicitly when `import.avdl` is tested (gap #1), but
a focused test for the `putOnClassPath/` chain in isolation would help
localize failures.

---

## 5. No negative/error-case tests

**Priority: Medium** -- The test suite only verifies successful parsing.
There are no tests that verify the parser correctly rejects invalid
input or produces appropriate error messages.

### 5a. Annotation on type reference (`AnnotationOnTypeReference.avdl`)

The Java `TestReferenceAnnotationNotAllowed` test verifies that
`@foo("bar") MD5 hash = ...` (annotating a type reference) produces
an error: "Type references may not be annotated, at line 29, column 16".

Our test should parse `AnnotationOnTypeReference.avdl` (located at
`avro/lang/java/idl/src/test/idl/AnnotationOnTypeReference.avdl`) and
assert that it returns a specific error. This validates that the parser
enforces the semantic rule that annotations on type references are
forbidden.

### 5b. Invalid syntax inputs

Add at least a few hand-crafted `.avdl` snippets that exercise parser
error paths:

- Missing semicolons in record fields
- Undeclared type references (unresolved forward references to
  nonexistent types)
- Duplicate type definitions
- Malformed `@namespace` annotations
- Import of nonexistent files

These can be inline strings rather than files, using `parse_idl`
directly.

### 5c. Import cycle detection

The `ImportContext` has cycle prevention logic, but no test verifies
that parsing a file that imports itself (or two files that import each
other) produces a graceful error rather than an infinite loop or stack
overflow.

---

## 6. No tests for `extra/` directory inputs

**Priority: Medium** -- The `extra/` directory contains two files that
the Java `TestIdlReader` tests against, but our test suite ignores.

### 6a. `extra/protocolSyntax.avdl`

A minimal protocol definition. Java's `validateProtocolParsingResult`
test verifies:

- `getNamedSchemas()` has size 1
- `getNamedSchema("communication.Message")` is not null
- `getProtocol()` is not null
- `getMainSchema()` is null

Our equivalent test should parse this file and verify that
`IdlFile::ProtocolFile` is returned, the protocol has one type named
`communication.Message`, and that the protocol namespace and name are
correct.

### 6b. `extra/schemaSyntax.avdl`

A schema-mode file with `schema array<Message>;`. Java's
`validateSchemaParsingResult` test verifies:

- `getNamedSchemas()` has size 1
- `getNamedSchema("communication.Message")` is not null
- `getProtocol()` is null
- `getMainSchema()` is an ARRAY type whose element type is
  `communication.Message`

Our equivalent test should parse this and verify that
`IdlFile::SchemaFile` is returned, the schema is an array type, and
the registry contains `communication.Message`.

---

## 7. No doc comment and warning tests

**Priority: Medium** -- Java's `testDocCommentsAndWarnings` test
(in `TestIdlReader.java`) verifies that the `comments.avdl` file
produces correct doc strings on specific types, fields, and methods,
and also that out-of-place doc comments generate warnings at specific
line/column positions.

### What this would exercise

Our integration test for `comments.avdl` compares the full JSON output
against `comments.avpr`, which implicitly tests doc comment content.
However, it does not test:

- **Warning generation for misplaced doc comments**: The Java test
  asserts 24 specific warnings with line and column numbers. Our parser
  should detect these and either emit warnings or silently handle them.
  There is currently no mechanism to test this.
- **Individual doc field assertions**: If the golden file comparison
  passes, the doc comments are correct. But if it starts failing, a
  targeted test that checks specific doc strings would help diagnose
  which doc comment extraction went wrong.

---

## 8. No logical type field tests

**Priority: Low** -- Java's `TestLogicalTypes` test parses
`logicalTypes.avdl` (located at
`avro/lang/java/idl/src/test/idl/logicalTypes.avdl`) and verifies that
specific fields have the correct logical type in the parsed schema
objects.

### What this would exercise

- `date`, `time_ms`, `timestamp_ms`, `local_timestamp_ms` built-in
  logical types
- `decimal(6,2)` with precision and scale
- `uuid` logical type
- `@logicalType("timestamp-micros") long` -- annotation-based logical
  type on a primitive
- `@logicalType("decimal") @precision(6) @scale(2) bytes` --
  annotation-based decimal on bytes
- `@logicalType("decimal") @precision(3000000000) @scale(0) bytes` --
  invalid precision (exceeds int range), which should produce an
  annotated primitive without a valid logical type

The existing golden-file tests for `simple.avdl` and `interop.avdl`
cover some logical types implicitly, but `logicalTypes.avdl` is the
dedicated stress test for logical type handling and is not in the
`input/` directory, so it has no golden `.avpr` file. A unit-level test
that parses it and checks field-level logical type metadata would be
needed.

---

## 9. `TestCycle` functionality not covered

**Priority: Low** -- The Java `TestCycle` test does more than just
parse `cycle.avdl` -- it builds `GenericRecord` instances from the
parsed schemas and round-trips them through binary serialization. This
validates that the schemas are structurally correct for actual Avro
data operations, not just JSON-equivalent.

Note: The `cycle.avdl` in the test root (different namespace
`org.apache.avro.gen.test`, different records `Record1/Record2/Record3`)
is different from the `input/cycle.avdl` (`org.apache.avro.gen`,
records `SampleNode/Method/SamplePair/SelfRef`). The `TestCycle` test
uses the one outside `input/` that is not covered by our `test_cycle`
integration test. However, since our tool is an IDL compiler (not a
data serialization library), the serialization round-trip is out of
scope. What matters is ensuring both `cycle.avdl` variants parse
correctly and produce valid schema JSON.

### Recommended test

Parse the test-root `cycle.avdl` (the `Record1`/`Record2`/`Record3`
variant at `avro/lang/java/idl/src/test/idl/cycle.avdl`) and verify it
produces valid JSON with correctly interlinked record references. This
variant has more complex cycles (three records referencing each other
in a cycle through forward references).

---

## 10. No CLI-level integration tests

**Priority: Low** -- All existing integration tests call the library
API directly (`parse_idl`, `parse_and_serialize`). There are no tests
that exercise the CLI binary (`cargo run -- idl ...` or
`cargo run -- idl2schemata ...`).

### What this would exercise

- Stdin/stdout piping (reading from stdin, writing to stdout)
- File argument handling
- `--import-dir` flag parsing and behavior
- Error output formatting (miette diagnostics)
- Exit codes for success and failure

CLI-level tests would use `std::process::Command` to invoke the built
binary and assert on stdout, stderr, and exit code.

---

## Priority summary

| # | Gap                                            | Priority |
|---|------------------------------------------------|----------|
| 1 | `import.avdl` and `nestedimport.avdl` tests   | High     |
| 2 | `test_status_schema` workaround                | High     |
| 3 | `idl2schemata` integration tests               | High     |
| 4 | `putOnClassPath/` import resolution tests      | Medium   |
| 5 | Negative/error-case tests                      | Medium   |
| 6 | `extra/` directory input tests                 | Medium   |
| 7 | Doc comment and warning tests                  | Medium   |
| 8 | Logical type field tests                       | Low      |
| 9 | Second `cycle.avdl` variant                    | Low      |
| 10| CLI-level integration tests                    | Low      |
