# Float/double values use different notation than Java

## Symptom

Large float and double default values and annotation values are
formatted differently from Java's output. This causes the
`compare-golden.sh` semantic comparison to fail for `interop.avdl`
(the only golden test file containing a large float default).

Examples of formatting differences:

| Value       | Java (Jackson)       | Rust (serde_json)      |
|-------------|---------------------|------------------------|
| 1e8         | `1.0E+8`            | `100000000.0`          |
| 1e12        | `1.0E+12`           | `1000000000000.0`      |
| -1e12       | `-1.0E+12`          | `-1000000000000.0`     |
| 1e20        | `1.0E+20`           | `1E+20`                |
| 1e-20       | `1.0E-20`           | `1E-20`                |
| 9007199254740991.0 | `9007199254740991` | `9007199254740991.0` |

The values are semantically identical (same IEEE 754 bit pattern),
but `jq -S .` preserves the original formatting, causing textual
diffs.

## Root cause

`serde_json::to_string_pretty` uses the `ryu` crate for float
formatting, which chooses the shortest unique representation. Java's
Jackson `JsonGenerator` uses `Double.toString()` / scientific
notation with `1.0E+N` format for values >= 1e7 in magnitude.

The float values are stored as `serde_json::Number` (via
`Number::from_f64` in `parse_floating_point_literal`) and the
formatting happens at final serialization time.

## Affected files

- `src/reader.rs` — `parse_floating_point_literal` (line ~2338)
- `src/main.rs` — `serde_json::to_string_pretty` (lines 214, 247)
- Affects both `idl` and `idl2schemata` output
- Affects both field default values and annotation values

## Reproduction

```sh
scripts/compare-golden.sh idl interop
# Shows: "default": -1000000000000.0 vs -1.0E+12

# Or use adhoc test:
cat > tmp/test-float.avdl <<'EOF'
protocol Test { record R { double x = 1.0e12; } }
EOF
scripts/compare-adhoc.sh tmp/test-float.avdl
```

## Suggested fix

Use a custom JSON formatter/serializer instead of
`serde_json::to_string_pretty`. The custom formatter would need to
intercept `f64` serialization and format values using Java-compatible
scientific notation rules:

1. Use scientific notation (`X.YE+N`) when |value| >= 1e7
2. Always include a `.0` in the mantissa (e.g., `1.0E+12` not `1E+12`)
3. For exact integer doubles, Java drops the `.0` (e.g., `9007199254740991`)

This could be implemented as a `serde_json::ser::Formatter` that
wraps `PrettyFormatter` and overrides `write_f64`. Note that
`serde_json`'s `Formatter` trait has a `write_f64` method that can
be overridden for custom float formatting.

Alternatively, a post-processing pass on the `Value` tree could
replace `Number` values with pre-formatted strings, but this would
be fragile and harder to maintain.

## History

A `JavaPrettyFormatter` with `format_f64_like_java` and
`decimal_to_scientific` was previously implemented (commit `a7a895e`)
but then removed (commit `937ce33`) as "purely cosmetic complexity."
The removal commit's message stated that `compare-golden.sh`
"distinguishes semantic matches from byte-exact matches," but this
is incorrect: `compare-golden.sh` uses `jq -S .` for the "semantic"
comparison, and `jq` preserves float formatting, so the diff is
reported as a `FAIL`, not a cosmetic `PASS`.

Two options:
1. Restore `JavaPrettyFormatter` (was ~150 lines + ~30 tests).
2. Fix `compare-golden.sh` to do a true semantic comparison (e.g.,
   parse both files with Python/jq and compare values rather than
   text). This would make the golden comparison immune to float
   formatting differences but would not match Java's output format.
