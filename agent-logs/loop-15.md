# Iteration 15 (2026-02-10)

User-seeded issues (4) plus focused discovery (6 more issues from 3
agents), for 10 total issues resolved across 3 waves.

**Phase 1**: 3 focused discovery agents:

- **Schema mode deep dive**: 27 edge cases tested. Found one genuine
  bug: import-only schema-mode files rejected before imports resolve.
  Filed `59e6e811`. All other schema-mode behaviors match Java.

- **Recent refactor audit**: Confirmed iteration 14's miette migration
  is correct. No bugs found. Identified stale comments in `main.rs`
  and `pretty_assertions` missing from 2 minor modules (noted in
  SESSION.md, not worth standalone issues).

- **Error UX audit**: Tested ~30 malformed inputs. Found 4 diagnostic
  quality issues: `ecde4332` (ANTLR token-list wall of text),
  `a3184e6d` (import error source span lost), `df6af1a1` (import dirs
  not listed in error), `a84d32e3` (confusing outdir error). Also
  found `bf52a6d9` (poor English in "N import dir(s)").

**Phase 2**: 3 waves:

- **Wave 1** (3 parallel agents, functional fixes):
  - `59e6e811`: Moved "no schema nor protocol" check from parser to
    CLI layer. `idl2schemata` now accepts import-only files. 4 new
    tests.
  - `ecde4332`: Added `label` field to `ParseDiagnostic` to eliminate
    duplicate error text. Extended `enrich_antlr_error` to simplify
    large expected-token sets ("unexpected end of file", "unexpected
    token \`X\`"). 28 snapshot updates.
  - `a3184e6d`: Reversed wrapping order in `wrap_import_error` so
    `ParseDiagnostic` is root (source span rendered). 2 new snapshot
    tests with test fixtures.

- **Wave 2** (3 parallel agents, functional + UX):
  - `3f66bce1`: Warnings now emitted even on error path via
    `drain_warnings()` on the builder. Added 2 CLI stderr snapshot
    tests (warnings-only, warnings+error). Updated stale comments in
    `main.rs`.
  - `df6af1a1` + `bf52a6d9`: Import-not-found error now lists actual
    directory paths instead of a count. Both issues fixed in one
    commit.
  - `a84d32e3`: Pre-check before `create_dir_all` gives "path exists
    and is not a directory" instead of generic `EEXIST`. New CLI test.

- **Wave 3** (parent-applied batch, non-functional):
  - `1948d4f7`: Added float formatting bullet to README divergences.
  - `98484321`: Expanded Cargo.toml `include` to cover tests, snapshots,
    and Avro submodule fixtures.
  - `b8efb300`: Added `normalize_crlf` helper to `cli.rs` and
    `integration.rs` for Windows CRLF compatibility.

**Phase 3**: Updated SESSION.md with resolved items. Added 3 cross-
cutting tips to the workflow prompt. Cleaned up worktrees.

**Result**: 10 issues filed, 10 resolved, 0 remaining. Test count
grew from 464 to 477. All tests pass. Key improvements: better error
diagnostics (ANTLR message simplification, import error source spans),
warnings survive compilation failures, Windows CRLF compatibility,
and `cargo package` now includes the full test suite.
