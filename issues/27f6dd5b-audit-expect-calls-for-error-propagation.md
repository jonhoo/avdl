# Audit `.expect()` calls for error propagation opportunities

## Symptom

Production code uses `.expect()` in 8 locations where a panic is
currently considered "impossible by construction." While each call
has a justifying message, some of these could be refactored to
propagate errors via `?` or restructured to avoid the need for the
`expect` entirely — improving robustness and eliminating potential
panic paths.

## Locations to investigate

| File | Line | Message | Possible improvement |
|---|---|---|---|
| `src/reader.rs:1236` | `"dot presence checked above"` | Refactor to use `split_once('.')` with pattern matching |
| `src/reader.rs:1496` | `"dot presence checked above"` | Same as above |
| `src/import.rs:287` | `"dot presence checked above"` | Same as above |
| `src/model/json.rs:695` | `"format {:E} always produces an 'E'"` | Consider `find` + propagate or keep as-is (Rust format guarantee) |
| `src/model/json.rs:822` | `"serde_json produces valid UTF-8"` | Consider `from_utf8_lossy` or propagate error |
| `src/model/schema.rs:236` | `"named type always has full_name"` | Pattern match could be restructured to avoid the `expect` |
| `src/main.rs:304` | `"checked for None above"` | Refactor with `if let` or `match` to avoid the `expect` |
| `src/main.rs:553` | `"checked for None above"` | Same as above |

## Root cause

These patterns arose naturally during development: a guard condition
checks for the happy path, and then `expect()` is used downstream to
extract the value. This is idiomatic Rust for infallible cases, but
some of these could be rewritten using `if let`, `match`, or
`split_once` to make the infallibility structural rather than
comment-based.

## Suggested approach

For each location, evaluate:

1. **Can the code be restructured** so the `expect` is unnecessary?
   (e.g., `if let Some((ns, name)) = full.split_once('.')` instead
   of `if full.contains('.') { ... full.rsplit_once('.').expect(...) }`)

2. **Should the error propagate** via `?` instead of panicking?
   This is especially relevant for `main.rs` where user input could
   theoretically reach these paths.

3. **Is the `expect` justified** by a genuine Rust/library guarantee
   that can never fail? If so, document why and leave it.

Priority: low. All current `expect` calls are correctly guarded and
the codebase has zero `.unwrap()` calls in production code.

## Affected files

- `src/reader.rs`
- `src/import.rs`
- `src/model/json.rs`
- `src/model/schema.rs`
- `src/main.rs`
