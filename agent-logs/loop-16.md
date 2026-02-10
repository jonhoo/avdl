# Iteration 16 (2026-02-10)

4 pre-existing issues plus 8 new issues from discovery (plus 3
upstream issues), for 12 total issues resolved across 4 waves.

**Phase 1**: 4 focused discovery agents:

- **Protocol message semantics**: Tested 16+ edge cases (one-way,
  throws, defaults, annotations, cross-namespace errors, forward refs).
  Found one bug: `2046bfd8` (unresolved type references in message
  return types, parameters, and throws clauses not validated). Verified
  15 other message behaviors as correct.

- **Complex types and unions**: Tested 18 categories of edge cases
  (nested types, nullable reordering, recursive types, logical types,
  schema mode, cross-namespace refs, annotated primitives, number
  literals). Found one bug: `0b893e93` (`idl` accepts bare named types
  that Java rejects). Filed upstream bug `dafbaa0f` (Java crashes on
  `union { int, date }`).

- **TODO/code quality audit**: Scanned all source and test files. Found
  2 TODOs worth addressing (`c44fd7cc` SUB character, `b1e68c9a` union
  annotation warning), 2 duplication issues (`e4d58e40` span
  computation, `ac756fab` test helpers), and 1 coverage gap (`65b23e51`
  stdin tests).

- **Broad exploration**: Tested imports, CLI behavior, idl2schemata,
  extra test files, spec compliance, property handling. Found `cc035ef6`
  (no int/long range validation). Filed 2 upstream bugs (`1ec6f2bf`
  Java crashes on out-of-range int, `98dd6525` Java NPE on forward ref
  in return type). Verified extensive correct behaviors.

One duplicate was removed: `ec974209` duplicated `0b893e93`.

**Phase 2**: 4 waves:

- **Wave 1** (3 parallel agents, functional fixes):
  - `2046bfd8`: Added message reference validation in
    `validate_all_references` for response, request, and error schemas.
    5 new tests.
  - `cc035ef6`: Split Int/Long validation in `is_valid_default` with
    range checks (i32 for int, i64 for long). Range-specific error
    messages. 16 new tests.
  - `c44fd7cc`: Strip `\u001a` and trailing content before lexing, as
    the existing TODO suggested. 2 new tests.

- **Wave 2** (3 parallel agents, functional + UX):
  - `0b893e93`: Reject all `NamedSchemas` in `Idl::convert_impl`,
    matching Java's `IdlTool.run()`. Updated integration test. 2 new
    tests.
  - `41259f93`: Fixed `make_diagnostic` to use `ctx.stop()` for the end
    of the span instead of only `start_token.get_stop()`. 19 snapshot
    updates.
  - `47ae3953`: Added bare-identifier detection in ANTLR error
    enrichment. Suggests quoting when identifier appears where
    `StringLiteral` is expected. 14 new tests.

- **Wave 3** (2 parallel agents, UX + code quality):
  - `b1e68c9a`: Added `warnings` field to `SourceInfo`, emit warning
    when annotations are dropped on non-nullable unions. 2 new tests.
  - `e4d58e40`: Extracted `span_from_offsets` helper, replaced 3
    duplicated blocks. Pure refactoring, -65 net lines.

- **Wave 4** (2 parallel agents, test/doc/infra):
  - `c245ee0a` + `ac756fab` + `65b23e51`: Created `tests/common/mod.rs`
    with shared helpers (including Java normalization comment). Added 2
    CLI stdin tests. Consolidated `normalize_crlf` and `render_warnings`.
  - `d248b402`: Created `.github/workflows/staleness.yml` with weekly
    avro-submodule and antlr4rust release checks.

**Phase 3**: SESSION.md cleaned (3 minor observations retained). No new
issues to file from remaining observations.

**Result**: 12 issues filed (8 new + 4 pre-existing), 12 resolved, 0
remaining. Test count grew from 477 to 520. All tests pass. Key
improvements: message reference validation, int/long range checking,
SUB character EOF handling, wider error annotation spans, bare enum
default quoting hint, union annotation drop warning, shared test
helpers, CI staleness detection.
