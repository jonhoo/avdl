# Regex `unwrap()` calls in `doc_comments.rs` should use `expect()`

## Symptom

Five `LazyLock<Regex>` initializations in `doc_comments.rs` use `.unwrap()`
instead of `.expect()` with a message explaining why the call cannot fail.

## Root cause

The CLAUDE.md guidelines state: "Prefer `expect()` over `unwrap()`. The
`expect` message should be very concise, and should explain why that
expect call cannot fail."

These are constant regex patterns compiled at static-init time. They are
guaranteed to be valid regex syntax, so `unwrap()` is safe, but `expect()`
is preferred per project convention.

## Affected files

- `src/doc_comments.rs`: lines 110, 115, 124, 130, 136

## Reproduction

```
grep '\.unwrap()' src/doc_comments.rs
```

## Suggested fix

Replace each `.unwrap()` with `.expect("regex is a valid constant pattern")`:

```rust
// Before:
Regex::new(r"...").unwrap()

// After:
Regex::new(r"...").expect("regex is a valid constant pattern")
```
