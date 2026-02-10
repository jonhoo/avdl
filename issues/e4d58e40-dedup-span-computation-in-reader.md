# Extract shared span computation helper in reader.rs

## Symptom

The same start/stop token-to-`SourceSpan` calculation logic is
duplicated three times in `reader.rs`:

1. `make_diagnostic` (line 842) -- from `ParserRuleContext`
2. `make_diagnostic_from_token` (line 873) -- from raw `Token`
3. `span_from_context` (line 905) -- from `ParserRuleContext`

Each contains the identical pattern:

```rust
let (offset, length) = if offset >= 0 && stop >= offset {
    (offset as usize, (stop - offset + 1) as usize)
} else if offset >= 0 {
    (offset as usize, 1)
} else {
    (0, 0)
};
```

## Root cause

These functions were written at different times to serve different
call sites. The core span arithmetic is identical but has not been
extracted into a shared helper.

## Affected files

- `src/reader.rs` (lines 842, 873, 905)

## Reproduction

Not a bug, but a code quality issue. Search for
`offset >= 0 && stop >= offset` in `reader.rs`.

## Suggested fix

Extract a small helper function:

```rust
/// Compute a `SourceSpan` from ANTLR's inclusive start/stop byte offsets.
/// Returns `(offset, length)` covering at least one character when possible.
fn span_from_offsets(start: isize, stop: isize) -> (usize, usize) {
    if start >= 0 && stop >= start {
        (start as usize, (stop - start + 1) as usize)
    } else if start >= 0 {
        (start as usize, 1)
    } else {
        (0, 0)
    }
}
```

Then use it in all three locations. This eliminates the duplication
and creates a single place to update if the offset logic changes.
