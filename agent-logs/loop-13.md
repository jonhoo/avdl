# Iteration 13 (2026-02-09)

TODO-driven discovery from `TODOs.md` (warning source spans and
Debug output) plus one broader exploration agent. Original iteration
was redone with two adapted approaches: warning tests use
`insta::assert_debug_snapshot!` instead of string assertions, and
`JavaPrettyFormatter` is not restored (Rust integration tests already
do semantic `Value` comparison, immune to float notation).

**Phase 1**: 2 discovery agents filed 4 issues:
- 3 warning-system issues: lexer spans always (0,0), warnings
  downgraded to `Vec<String>` losing rich diagnostics, and verbose
  `#[derive(Debug)]` output.
- 1 float formatting regression: `JavaPrettyFormatter` removed in
  prior iteration caused `compare-golden.sh` (text-based `jq -S`
  comparison) to report `interop.avdl` as a failure.

**Phase 2**: 1 wave, 2 parallel agents:
- **Agent A** (`fix/warning-system`): Fixed all 3 warning issues.
  Added `line_col_to_byte_offset` for lexer error spans, custom
  `Debug` impl for compact output, changed `IdlOutput`/
  `SchemataOutput` to `Vec<Warning>`, added miette rendering in CLI.
  Re-exported `Warning` from crate root. Converted all 5 warning
  tests to `insta::assert_debug_snapshot!` (6 new/updated snapshot
  files).
- **Agent B** (`fix/delete-compare-golden`): Deleted
  `scripts/compare-golden.sh` (459 lines) — its functionality is
  fully covered by `cargo test` which uses semantic `Value`
  comparison. Updated CLAUDE.md and `workflow-prompts/refinement-loop.md`
  to reference `cargo test` as the canonical correctness check. Added
  `tests/testdata/idl2schemata-golden/README.md` documenting
  provenance and regeneration of the 61 golden `.avsc` files. Verified
  all golden files match Java tool output. Closed the float formatting
  issue (moot without text-based comparison).

**Result**: 4 issues closed (0 remaining). 464 tests pass (370 unit
+ 9 CLI + 30 error reporting + 51 integration + 4 doctests). All 18
idl + 61 idl2schemata golden comparisons pass via `cargo test`.
`compare-golden.sh` removed; `compare-adhoc.sh` retained for manual
edge-case exploration.
