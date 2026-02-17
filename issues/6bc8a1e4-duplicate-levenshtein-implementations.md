# Duplicate Levenshtein distance implementations

## Symptom

Two independent implementations of the Levenshtein edit distance
algorithm exist in the codebase:

- `reader.rs:824` -- `fn levenshtein_distance(a: &str, b: &str) -> usize`
- `compiler.rs:940` -- `fn levenshtein(a: &str, b: &str) -> usize`

Both are private (`fn`, not `pub`), implement the same algorithm with
minor stylistic differences (one uses `chars().enumerate()` while the
other collects to `Vec<char>` and indexes), and have their own separate
unit test suites.

## Root cause

`levenshtein_distance` was added in `reader.rs` for ANTLR error
enrichment (suggesting keywords for misspelled tokens).
`levenshtein` was later added in `compiler.rs` for "did you mean?"
suggestions on unresolved type names. Neither author noticed the
other.

## Affected files

- `src/reader.rs` (lines 824-849, tests at 6551-6567)
- `src/compiler.rs` (lines 940-974, tests at 1786-1821)

## Suggested fix

Extract a shared `fn levenshtein(a: &str, b: &str) -> usize` into a
utility location (e.g., a small `pub(crate)` function in a shared
module, or in `error.rs` alongside other diagnostic helpers). Both
call sites import from there. Consolidate the two test suites into
one, keeping the best tests from each.

The `max_distance` helper in `compiler.rs` (lines 980-986) and the
`suggest_keyword` function in `reader.rs` (lines 855-882) both
define the same "short names allow distance 1, longer names allow
distance 2" policy. If extracted together, this threshold logic could
also be shared.
