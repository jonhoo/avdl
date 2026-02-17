# Iteration 20

## Starting state

- 0 open issues
- 726 tests passing (unit + integration + doc)
- All clean

## Phase 1: Discovery

Launched 4 focused discovery agents:

1. Java edge case comparison (80+ .avdl files) — filed 1 issue
   (float formatting, closed as non-goal during triage)
2. Error message quality audit (60 mutations) — filed 5 issues
3. Code quality and TODO audit — filed 5 issues
4. Spec compliance and Java source audit — no new functional issues

3 additional issues filed from SESSION.md observations (test
boilerplate, .contains() assertions, logical type validation gap).

**Results:**

- 13 issues filed after triage (1 float-formatting issue discarded as
  non-goal before filing)
- 6 functional issues: 5 error message quality gaps + 1 validation gap
- 7 non-functional: code duplication, style compliance, test quality,
  documentation staleness

## Phase 2: Resolution

### Wave 1 (parent-applied, 4 small fixes in worktree wt-a)

- `ImportContext` fields made private (import.rs)
- 5 regex `unwrap()` → `expect()` (doc_comments.rs)
- Java line-number references → method names (reader.rs comments)
- CLAUDE.md project layout updated with `compiler.rs` and `suggest.rs`

No conflicts.

### Wave 2 (1 agent in wt-b, 3 reader.rs error fixes)

- Negative fixed size (`fixed Hash(-5)`) now shows "value must be a
  non-negative integer" instead of Rust's `IntErrorKind::InvalidDigit`
- `map<>`/`array<>` cascading errors collapsed into single targeted
  message
- Missing record `}` before another declaration detected and pointed
  at the unclosed construct

8 new tests, 5 new snapshots.

### Wave 3a (1 agent in wt-c, 2 compiler.rs error fixes — parallel with 2 and 3b)

- `void` as field type now explains it's only valid as message return
  type instead of "Undefined name: void"
- `decimal` without parameters now explains `(precision, scale)` syntax
  instead of "Undefined name: decimal"

2 new tests, 2 new snapshots.

### Wave 3b (1 agent in wt-d, schema.rs DRY refactor — parallel with 2 and 3a)

- Deduplicated `LogicalType`-to-base-type mapping: `union_type_key()`
  and `is_valid_default()` now delegate to `expected_base_type()`.
  Net reduction of ~80 lines.

No conflicts on cherry-pick. CHANGELOG.md auto-merged cleanly.

### Wave 4 (1 agent in wt-e, logical type validation)

- Extended `try_promote_logical_type` to validate `@logicalType` on
  `Fixed` schemas (`duration` requires fixed(12), `decimal` precision
  checked against byte size)
- Also called from `walk_fixed` for named fixed declarations
- Design trade-off: `Duration` not added as `LogicalType` variant
  since `Logical` requires a `PrimitiveType` base; Fixed
  representation with `logicalType` property already produces correct
  JSON

15 new tests, 1 new snapshot.

### Wave 5a (1 agent in wt-f, test boilerplate reduction)

- Added `Field::simple()`, `AvroSchema::simple_record()`,
  `AvroSchema::simple_enum()` test helpers
- Replaced 60+ verbose constructions across `model/schema.rs` and
  `model/json.rs`
- Net reduction of ~330 lines

### Wave 5b (1 agent in wt-g, .contains() → snapshot migration)

- Converted 53 `.contains()` error assertions to `insta::assert_snapshot!`
  across reader.rs, compiler.rs, resolve.rs, model/schema.rs
- Added `format_enriched()`/`format_syntax_error()` test helpers for
  private error structs
- Fixed path-dependent snapshot for `import_resolution_error_has_source_span`
  (CWD normalization)
- 53 new snapshot files

### Drive-by fix

- Fixed path-dependent snapshot that used worktree path instead of
  normalized `<cwd>` placeholder (import resolution error test)

## End state

- 0 open issues
- 751 tests passing (up from 726, +25)
- 8 commits in main (excluding discovery commit)

## Key learnings

- **4 focused agents outperformed 5 broad agents**: The Java edge case
  agent tested 80+ files but found only 1 formatting-only difference
  (non-goal). The spec/Java audit agent confirmed all 32 grammar
  productions are handled and found zero functional gaps. The error UX
  and code quality agents were more productive.
- **Path-dependent snapshots need CWD normalization**: When a test uses
  `convert_str()` (which resolves imports relative to CWD), the
  snapshot must normalize the CWD to a placeholder like `<cwd>`. The
  worktree path baked into the snapshot broke when cherry-picked to
  main.
- **Parallel Waves 2+3a+3b had zero conflicts**: reader.rs, compiler.rs,
  and schema.rs were completely independent. CHANGELOG.md auto-merged
  cleanly.
- **Test boilerplate reduction was high-value**: Removing ~330 lines of
  boilerplate from test code made the snapshot migration (Wave 5b)
  slightly easier since the test code was cleaner.
- **The codebase is approaching maturity**: 138 edge case tests (across
  iterations 19-20) found zero functional discrepancies with Java.
  All grammar productions are covered. Future iterations should focus
  on error UX, performance, and documentation rather than correctness.
