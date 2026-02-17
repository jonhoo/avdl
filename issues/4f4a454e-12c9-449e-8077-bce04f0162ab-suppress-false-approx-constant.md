# Suppress false-positive `clippy::approx_constant` warnings in tests

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

- `src/reader.rs`: lines 4923, 4930, 4936, 4942, 4948 — `parse_float_text`
  unit tests using `3.14` with various suffixes
- `src/model/schema.rs`: lines 1210, 1220, 1279, 1307 — `is_valid_default`
  unit tests using `3.14` and `2.718`

## Reproduction

```sh
cargo clippy --all-targets -- -W clippy::approx_constant
```

## Suggested fix

Add `#[allow(clippy::approx_constant)]` to each affected test function (or to
the test module as a whole). Changing the values to something like `3.15` would
also silence the lint but obscures the intent — `3.14` is the conventional
"any float" test value.

Preferred approach: annotate each test function individually using
`#[expect]` (Rust 1.81+) with a `reason`, so the compiler warns if the
suppression ever becomes unnecessary (e.g., the test values change):

```rust
#[test]
#[expect(clippy::approx_constant, reason = "3.14 is an arbitrary test value, not pi")]
fn float_decimal_no_suffix() {
    let val = parse_float_text("3.14").expect("plain decimal");
    assert!((val - 3.14).abs() < f64::EPSILON);
}
```

Use `#[expect]` rather than `#[allow]` for all new lint suppressions
that are expected to trigger — this ensures suppressions don't silently
become dead annotations.
