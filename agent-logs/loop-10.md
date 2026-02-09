# Iteration 10 (2026-02-09)

Phase 1 launched 3 focused discovery agents (edge-case fuzz testing,
Java source method-by-method audit, error mutation testing). Tested
~80 edge-case `.avdl` files. Found 4 new issues: 3 correctness bugs
and 1 compatibility issue.

- **Wave 1** (3 fixes + 1 close, batched on main): Rejected
  annotations on messages with named return types (matching Java's
  `exitNullableType`). Omitted empty namespace from JSON output.
  Rewrote doc comment indent stripping with `regex` to uniformly
  handle any horizontal whitespace around star prefixes. Closed JSON
  comments issue as intentional divergence (later reversed).
- **Wave 2** (2 parallel agents): Agent A added post-registration
  default value validation for `Reference`-typed fields — resolves
  references through `SchemaRegistry` before checking. Agent B
  implemented C-style comment stripping for imported `.avsc`/`.avpr`
  files (50-line state machine, matching Java's Jackson
  `ALLOW_COMMENTS`).
- **Phase 3**: Closed 5 issues. Cleaned SESSION.md. 13 non-functional
  issues remain (source spans, performance, code quality, CRLF
  handling, compatibility notes).

**Result**: 5 functional issues closed. 372 unit + 7 CLI + 30 error
reporting + 52 integration tests passing (461 total, up from 429).
All 18 idl + 62 idl2schemata golden comparisons passing. Added `regex`
dependency for doc comment handling. 13 non-functional issues remain.
