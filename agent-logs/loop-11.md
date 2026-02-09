# Iteration 11 (2026-02-09)

Phase 1 investigated 4 items from `TODOs.md` with one agent per item.
Two were non-issues (`to_json_string` vs `Serialize` impls — stateful
serialization is incompatible; `serde_path_to_error` — all JSON is
deserialized to untyped `Value`). Two yielded actionable issues
(deduplicate `parse_json_with_comments`, rename `parse_idl` test
helper). All 6 pre-existing issues resolved across 3 waves:

- **Wave 1** (batched in worktree by parent): Extracted
  `parse_json_with_comments` from `#[cfg(test)]` to module scope in
  `import.rs`. Renamed `parse_idl` to `parse_idl_for_test` in
  `reader.rs` (~58 call sites). 2 issues closed.
- **Wave 2** (2 parallel agents): Agent A generated 62 golden `.avsc`
  files from Java `idl2schemata` and rewrote
  `test_idl2schemata_golden_comparison` for full JSON content
  comparison (issue `deade9c4`). Agent B added
  `Option<miette::SourceSpan>` to `AvroSchema::Reference`, threading
  spans through 4 parser construction sites, 5 JSON import sites,
  `validate_references`, and `validate_all_references` to produce
  rich source-highlighted diagnostics (issue `4d95b38f`). 2 issues
  closed.
- **Wave 3** (2 parallel agents): Agent C strengthened 4 existing CLI
  tests with semantic JSON comparison and added 2 new error-path
  tests (`no_subcommand`, `unknown_subcommand`) (issue `1c9c723f`).
  Agent D added `enrich_antlr_error()` with pattern matching for
  annotation syntax mistakes — extracts merged `@word` tokens and
  suggests correct syntax (issue `aa4d9d53`). 2 issues closed.

**Result**: 6 issues closed (0 remaining). 392 unit + 9 CLI + 30
error reporting + 51 integration tests passing (482 total, up from
461). All 18 idl + 62 idl2schemata golden comparisons passing.
`TODOs.md` items fully investigated and resolved.
