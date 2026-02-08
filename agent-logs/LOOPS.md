# Loop Summaries

## Iteration 2 (2026-02-08)

Skipped Phase 1 (just completed). Cleaned SESSION.md, filed 3 new
issues from observations. Grouped 10 functional issues into 3 waves,
deferred 7 non-functional issues.

- **Wave 1** (4 issues): Batched 2 trivial main.rs/json.rs fixes on
  main directly. Agent A added ANTLR error listener. Agent B found
  cross-namespace resolution was already correct, instead made
  unresolved refs fatal errors. 5 issues closed.
- **Wave 2** (2 issues): Agent C fixed float literal suffixes (f/F/d/D
  and hex floats). Agent D added reserved property validation. 2 issues
  closed.
- **Wave 3** (3 issues): Agent E added logicaltype promotion from
  annotations. Agent F added Avro name validation (rejects dashed
  identifiers). Agent G added duplicate union type detection. 3 issues
  closed.

**Result**: 10 issues closed, 193 unit + 39 integration tests passing,
all 18 golden files passing. 7 non-functional issues remain.

## Iteration 3 (2026-02-08)

Phase 1 discovered 5 new functional issues (enum default validation,
protocol name validation, alias name validation, decimal precision
overflow, aliases/properties key ordering). All 5 were small enough to
batch on main in a single Wave 1 commit — no sub-agents needed.

**Result**: 5 issues closed, 201 unit + 39 integration tests passing,
all 18 golden files passing. 7 non-functional issues remain (same set
as iteration 2 — no new functional issues to find).

## Iteration 4 (2026-02-08)

Phase 1 explored schema mode, messages/imports, and default values.
Found 2 functional bugs and 1 design-choice difference. Agent A fixed
schema mode reference validation (unresolved `schema <type>;` now
errors). Agent B added recursive named-type registration from imported
`.avsc`/`.avpr` files.

**Result**: 2 functional issues closed, 216 unit + 39 integration
tests passing, all 18 golden files passing. 8 non-functional/design
issues remain.

## Iteration 5 (2026-02-08)

Skipped Phase 1. Cleaned SESSION.md, filed 2 new issues from
observations. Resolved all 10 remaining issues across 4 waves.

- **Wave 1** (3 issues): Batched on main — expanded
  `compare-golden.sh idl2schemata` from 6 to all 18 files (fixing
  nullglob handling), added TODO comments for `fixDefaultValue` and
  union property drop.
- **Wave 2** (2 issues, parallel agents): Agent A added `.context()`
  to ~25 bare `?` operators across import.rs/reader.rs/main.rs.
  Agent B added UTF-16 surrogate pair decoding to `unescape_java`.
- **Wave 3** (2 issues, sequential dependency): Single agent
  implemented out-of-place doc comment warnings (matching Java's 24
  warnings for `comments.avdl`) and import warning propagation with
  filename prefixing.
- **Wave 4** (2 issues, parallel agents): Agent D added 7 integration
  tests (idl2schemata interop/import, cycle detection, warning count,
  tools protocol). Agent E created `tests/error_reporting.rs` with 14
  insta snapshot tests. Also closed `d7e10ff5` as intentional design
  difference.

**Result**: 10 issues closed (0 remaining), 222 unit + 45 integration
+ 14 snapshot tests passing, all 18 idl + 62 idl2schemata golden
comparisons passing.

## Iteration 6 (2026-02-08)

Phase 1 launched 5 discovery agents (Java source audit, edge case
testing, TODO/test coverage scan, error path comparison, JSON
serialization audit). Found 3 correctness bugs, 4 test gaps, and 1
upstream Java bug.

- **Wave 1** (3 issues, 2 parallel agents): Agent A implemented
  Java's `shouldWriteFull` logic — added `SCHEMA_TYPE_NAMES` collision
  check to `schema_ref_name` and alias namespace shortening in
  `schema_to_json` (issues `4194dd45` + `7afa667a`). Agent B added
  default value type validation per Avro spec — new `is_valid_default`
  function in `schema.rs` and validation call in `walk_variable`
  (issue `01ee3f73`).
- **Wave 2** (2 issues, 2 parallel agents): Agent C replaced
  `format!("{:E}")` with ryu-based scientific notation in
  `format_f64_like_java` (issue `32df45fb`). Agent D documented that
  `serde_json`'s unified number representation makes `fixDefaultValue`
  int-to-long coercion implicit, removed TODO (issue `445ea3c2`).
- **Wave 3** (4 issues, 1 agent): Added `idl2schemata` golden-file
  comparison test, `idl2schemata` error path test, doc comment warning
  position verification (24 exact positions), and test-root `cycle.avdl`
  coverage (issues `200f5d4d`, `a97f4a5e`, `26de449d`, `5b2199d6`
  Gap 7).
- **Wave 4** (3 issues, 3 parallel agents): Agent F populated README
  "Intentional divergences from Java" section with 5 entries (issue
  `c041f540`). Agent G replaced `IdlError` enum with `miette::Result`
  across 7 files (issue `c7fca398`). Agent H audited `.expect()` calls,
  eliminating 5 via `rsplit_once`/`match` restructuring (issue
  `27f6dd5b`). One merge conflict resolved (IdlError + expect audit
  both modified `main.rs`).

**Result**: 11 issues closed (1 remaining: Gap 8 CLI-level tests,
deferred). 335 unit + 48 integration + 14 error reporting tests
passing (397 total, up from 281). All 18 idl + 62 idl2schemata golden
comparisons passing.
