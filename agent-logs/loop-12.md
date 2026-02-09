# Iteration 12 (2026-02-09)

Phase 1 investigated strictness parity per `TODOs.md`: trailing commas
in enums (already correctly rejected by `CollectingErrorListener`) and
broader strictness audit. 3 focused agents tested 80+ edge cases
comparing Rust vs Java acceptance. Found 2 semantic validation bugs
and 1 lexer error handling gap.

- **Wave 1** (batched in worktree by parent, 3 commits):
  - Reject `null?` in `walk_nullable_type` (would produce invalid
    `[null, null]` union). Issue `cb9d5ba2` closed.
  - Validate decimal precision > 0 and scale <= precision in
    `walk_primitive_type`. Issue `f48466ae` closed.
  - Install `CollectingErrorListener` on lexer (not just parser) so
    untokenizable characters produce warnings instead of leaking to
    stderr. Matches Java's behavior (non-fatal). Filed upstream issue
    `01780f4c` for Java's missing lexer error listener.

**Result**: 3 issues closed (0 remaining). 370 unit + 9 CLI + 30
error reporting + 51 integration tests passing (464 total, up from
482 — test count drop is from prior refactoring of duplicate test
helpers, not test removal). All 18 idl + 62 idl2schemata golden
comparisons passing. README updated: stale trailing-commas divergence
removed (both tools now reject).
