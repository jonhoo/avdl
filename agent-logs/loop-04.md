# Iteration 4 (2026-02-08)

Phase 1 explored schema mode, messages/imports, and default values.
Found 2 functional bugs and 1 design-choice difference. Agent A fixed
schema mode reference validation (unresolved `schema <type>;` now
errors). Agent B added recursive named-type registration from imported
`.avsc`/`.avpr` files.

**Result**: 2 functional issues closed, 216 unit + 39 integration
tests passing, all 18 golden files passing. 8 non-functional/design
issues remain.
