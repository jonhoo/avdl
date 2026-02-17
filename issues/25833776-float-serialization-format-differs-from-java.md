# Float/double serialization format differs from Java

## Symptom

Float and double default values with large magnitudes are serialized
differently from Java's output. Rust uses `serde_json`'s default
formatting, while Java uses `Double.toString()`/`Float.toString()`.

Examples:
- `1.0e7` renders as `10000000.0` (Rust) vs `1.0E+7` (Java)
- `1.0e8` renders as `100000000.0` (Rust) vs `1.0E+8` (Java)
- `1.0e16` renders as `1E+16` (Rust) vs `1.0E+16` (Java)
- `1.0e100` renders as `1E+100` (Rust) vs `1.0E+100` (Java)
- `1.0e308` renders as `1E+308` (Rust) vs `1.0E+308` (Java)

The values are numerically identical — this is a formatting-only
difference. However, `jq -S` does not normalize float formatting, so
semantic comparisons using `jq` will report diffs for files containing
these values.

## Root cause

`serde_json::Number::from_f64` uses Rust's `ryu` crate for float
formatting, which produces minimal representations (omitting the
fractional `.0` in scientific notation). Java's `Double.toString()`
always includes `.0` before the exponent and uses uppercase `E`.

Specifically:
- Rust: `1E+20` (no `.0`, uppercase `E`)
- Java: `1.0E+20` (always includes `.0`)
- Rust: `10000000.0` (expands small exponents)
- Java: `1.0E+7` (always uses scientific notation for large values)

## Affected files

- `src/reader.rs` line 3577: `serde_json::Number::from_f64(val)`
- `src/model/json.rs`: serialization pipeline

## Reproduction

```avdl
@namespace("test.floats")
protocol FloatTest {
  record R {
    float f = 1.0e7;
    double d = 1.0e100;
  }
}
```

```sh
scripts/compare-adhoc.sh --show-output tmp/edge-88-float-serialization.avdl
```

## Suggested fix

This is a cosmetic difference and may not warrant fixing, given the
project's explicit non-goal of byte-identical output. If parity is
desired, a custom float formatter matching Java's `Double.toString()`
behavior could be used instead of `serde_json::Number::from_f64`.

The most impactful aspect is that `jq -S` comparison fails on these
values, which means the `compare-adhoc.sh` script and potentially
integration tests will flag false-positive diffs for files with large
float defaults.
