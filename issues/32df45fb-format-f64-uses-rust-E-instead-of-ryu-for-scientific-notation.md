# `format_f64_like_java` uses Rust `{:E}` instead of `ryu` for scientific notation

## Symptom

For extreme edge-case `f64` values that require scientific notation
(abs >= 1e7 or abs < 1e-3), the `format_f64_like_java` function
produces a different (though semantically equivalent) string
representation compared to Java's `Double.toString()`.

Examples:
- Input `4.9e-324` (smallest denormalized double): Rust outputs
  `5.0E-324`, Java outputs `4.9E-324`
- Input `1.0000000000000002e15`: Rust outputs `1.0000000000000003E15`,
  Java outputs `1.0000000000000002E15`

Both representations round-trip to the same IEEE 754 bit pattern, so
there is no semantic difference. However, `jq -S` comparison treats
them as different numbers due to `jq`'s own formatting, which means
ad-hoc comparison scripts using `jq -S` may report false diffs.

## Root cause

`format_f64_like_java` (line 690 of `json.rs`) uses Rust's
`format!("{val:E}")` for the scientific notation path. Rust's `{:E}`
format uses a different algorithm (based on the Dragon4 family) than
Java's `Double.toString()`, which since JDK 12 uses the Ryu algorithm.
The `ryu` crate produces the shortest round-trip representation, while
`{:E}` may produce a slightly different significand.

For values in the non-scientific range (abs in [1e-3, 1e7)), the code
already uses `ryu::Buffer::new().format(val)`, which is correct. The
discrepancy only occurs in the scientific notation branch.

## Affected files

- `src/model/json.rs` -- `format_f64_like_java` function

## Reproduction

```avdl
@namespace("test")
protocol P {
  record R {
    double v1 = 4.9e-324;
    double v2 = 1.0000000000000002e15;
  }
}
```

```sh
# Rust output:
cargo run -- idl tmp/test.avdl  # "default": 5.0E-324

# Java output:
java -jar avro-tools-1.12.1.jar idl tmp/test.avdl  # "default": 4.9E-324
```

Both values parse back to the same f64. The difference is cosmetic.

## Suggested fix

Use `ryu::Buffer::new().format(val)` (which produces the shortest
decimal representation) and then convert from ryu's lowercase `e`
notation to uppercase `E` notation, rather than using
`format!("{val:E}")`.

Sketch:

```rust
let ryu_str = ryu::Buffer::new().format(val).to_string();
// ryu produces "4.9e-324" — convert to uppercase E notation: "4.9E-324"
ryu_str.replace('e', "E")
```

The `ryu` output already uses the minimal significand and always
includes a decimal point when needed, so no additional `.0` insertion
should be required. However, verify that `ryu` always produces a
decimal point in the significand for values in the scientific range
(it may produce `1E7` without a `.0`).

## Priority

Very low. Only affects extreme edge-case values that never appear in
real-world Avro IDL files. All 18 `idl` golden tests and all 62
`idl2schemata` golden tests pass. The difference is semantically
invisible (same IEEE 754 bit pattern).
