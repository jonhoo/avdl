# Replace `3.14`/`2.718` test values to avoid `approx_constant` warnings

## Symptom

`cargo clippy --all-targets -- -W clippy::all` reports 9 `clippy::approx_constant`
warnings for the literal `3.14` in test code, and 1 for `2.718`. Clippy thinks
these are imprecise approximations of `PI` and `E`, but the tests are
intentionally using arbitrary decimal numbers to test float parsing and default
validation — they have nothing to do with the mathematical constants.

## Root cause

Clippy's `approx_constant` lint triggers on any floating-point literal that is
"close enough" to a known constant, regardless of context. In these tests, `3.14`
is used as a representative non-integer value for testing that the float parser
strips Java-style suffixes (`f`, `F`, `d`, `D`) and that `is_valid_default`
accepts numeric values for float/double schemas.

## Affected files

- `src/reader.rs`: `parse_float_text` unit tests using `3.14` with various
  suffixes
- `src/model/schema.rs`: `is_valid_default` unit tests using `3.14` and `2.718`

## Reproduction

```sh
cargo clippy --all-targets -- -W clippy::approx_constant
```

## Suggested fix

Replace the test values with floats that don't trigger the lint:
- `3.14` → `3.25` (or similar)
- `2.718` → `2.75` (or similar)

This avoids adding lint suppressions entirely. The tests are verifying
float parsing mechanics, not any specific numeric value, so any
non-integer decimal works equally well.
