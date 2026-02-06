# Error reporting test suite for malformed/invalid `.avdl` inputs

## Symptom

There are no tests that verify the *content* of error messages produced
by malformed or invalid `.avdl` inputs. The two existing negative tests
(`test_duplicate_type_definition`, `test_import_nonexistent_file`) only
assert `.is_err()` without inspecting the rendered error output. This
means error quality can silently regress, and there is no pressure to
produce helpful diagnostics.

## Motivation

Error UX is a first-class feature of this tool. Since we're building a
from-scratch Rust port with `miette` diagnostics, we have the
opportunity to produce *better* error messages than Java's `avro-tools`
(which often surfaces raw ANTLR stack traces or terse
`AvroTypeException` messages). We should lock in that quality with
snapshot tests so it doesn't regress.

## Proposed approach

Create a dedicated test file `tests/error_reporting.rs` that:

1. Feeds malformed `.avdl` input strings (inline in the test, not
   external files) to the parser/compiler pipeline.
2. Captures the rendered error output (using `miette`'s
   `GraphicalReportHandler` or similar, so we test what the user
   actually sees).
3. Snapshots the output with `insta` (`assert_snapshot!`), so that
   changes to error messages are reviewed explicitly via
   `cargo insta review`.

Using inline input strings (rather than `.avdl` files) keeps each test
self-contained and makes it obvious what the malformed input is. Group
tests by error category using descriptive names like
`test_error_missing_semicolon`, `test_error_undefined_type`, etc.

## Categories of malformed input to cover

### Syntax errors (ANTLR parse failures)

These test that the ANTLR-generated parser produces usable error
locations and messages (not just "mismatched input").

- Missing semicolons (e.g., `record Foo { int x }`)
- Missing closing braces (e.g., `protocol P { record R { int x; }`)
- Invalid tokens in type position (e.g., `record Foo { 123 x; }`)
- Unclosed string literals in annotations
- Empty protocol body (`protocol P { }` — may or may not be an error)
- Malformed union syntax (e.g., `union { int, , string }`)
- Malformed enum (e.g., `enum E { A, B, }` — trailing comma)
- Malformed fixed (e.g., `fixed F(not_a_number);`)

### Semantic/validation errors

These test our own validation logic in `reader.rs` and `resolve.rs`.

- Undefined/unresolved type reference (`record R { Nonexistent x; }`)
- Duplicate type name within the same protocol
- Duplicate field name within the same record
- Invalid default value type (e.g., string default for int field)
- One-way message with non-void return type (issue `877f0e96`)
- Annotation on a type reference (issue `caeb40b1`)
- Invalid logical type parameters (e.g., `decimal(0, 0)`,
  `decimal(-1, 5)`)
- Recursive type without indirection (if we detect this)

### Import errors

- Importing a nonexistent file
- Importing a file with invalid JSON (for `.avsc`/`.avpr` imports)
- Import cycle detection (A imports B imports A)
- Importing a file that itself has parse errors

### Schema mode errors

- `schema` keyword with an invalid type expression
- Multiple `schema` declarations in one file

## Relationship to existing issues

- **Issue #23** (missing `.context()` propagation): Once context is
  added to bare `?` operators, the improved error messages should be
  snapshot-tested here.
- **Issue `877f0e96`** (one-way must return void): Once the validation
  is added, add a snapshot test for its error message.
- **Issue `caeb40b1`** (annotation on type reference): Same — add a
  snapshot test once the check is implemented.

## Suggested implementation order

1. Set up the test file with a helper that compiles an inline `.avdl`
   string and renders the error via `miette`.
2. Start with 3-5 syntax error cases to establish the pattern.
3. Add semantic validation cases as the corresponding checks are
   implemented.
4. Add import error cases.

## Affected files

- `tests/error_reporting.rs` (new)
- `Cargo.toml` (add `insta` dev-dependency if not already present)
