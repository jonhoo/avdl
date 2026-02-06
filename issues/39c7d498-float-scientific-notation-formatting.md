# Large float defaults use expanded decimal instead of scientific notation

## Symptom

When a floating-point field default value has a large magnitude (e.g.,
`-1.0e12`), the Rust tool serializes it as an expanded decimal
(`-1000000000000.0`) while the Java tool and golden files use
scientific notation (`-1.0E+12`).

This affects `interop.avdl`, which has:

```
double doubleField = -1.0e12;
```

Rust output:

```json
{"default": -1000000000000.0, "name": "doubleField", "type": "double"}
```

Golden file (Java output):

```json
{"default": -1.0E+12, "name": "doubleField", "type": "double"}
```

Both representations are semantically identical JSON numbers, but the
textual difference causes a byte-level mismatch with the golden files.

## Root cause

In `src/reader.rs:1289-1307`, `parse_floating_point_literal` parses the
IDL literal `-1.0e12` into an `f64`, then stores it as a
`serde_json::Number` via `Number::from_f64`. When `serde_json`
serializes this number, it uses Rust's `ryu` crate, which always
outputs the shortest decimal representation without scientific notation
for values in a certain range. The result is `-1000000000000.0`.

Java's Jackson library, by contrast, uses `JsonGenerator` which
defaults to scientific notation for large values, producing `-1.0E+12`.

This is a consequence of `serde_json`'s number formatting, not a bug
in the parsing logic itself.

## Affected files

- `src/reader.rs:1289-1307` -- `parse_floating_point_literal`
- `interop.avdl` golden file comparison

## Reproduction

```sh
cargo run -- idl avro/lang/java/idl/src/test/idl/input/interop.avdl /dev/stdout \
  | python3 -c "
import json, sys
d = json.load(sys.stdin)
for t in d['types']:
    for f in t.get('fields', []):
        if f['name'] == 'doubleField':
            print(repr(f['default']))
"
# Actual:   -1000000000000.0
# Expected: -1.0E+12 (or any scientific notation form)
```

## Impact

- Only one test file (`interop.avdl`) is affected in the current test
  suite.
- The JSON values are semantically identical, so no functional
  difference in any Avro consumer.
- Fails byte-level golden file comparison but passes semantic (parsed)
  comparison.

## Suggested fix

Custom-format `f64` values to match Java's Jackson output: use
scientific notation for values whose absolute magnitude is >= 1e7 or
<= 1e-3 (matching Jackson's `NumberOutput.outDouble` behavior). One
approach:

```rust
fn f64_to_json_value(val: f64) -> Value {
    let abs = val.abs();
    if abs >= 1e7 || (abs > 0.0 && abs < 1e-3) {
        // Format in scientific notation matching Java
        let s = format!("{:E}", val);
        // Parse back to serde_json number
        serde_json::from_str(&s).unwrap_or_else(|_| {
            serde_json::Number::from_f64(val)
                .map(Value::Number)
                .unwrap_or(Value::String(val.to_string()))
        })
    } else {
        serde_json::Number::from_f64(val)
            .map(Value::Number)
            .unwrap_or(Value::String(val.to_string()))
    }
}
```

Alternatively, this could be addressed at serialization time rather
than parse time, by post-processing `serde_json::Value::Number` nodes
during final JSON output.

## Priority

Low. The difference is purely cosmetic and does not affect
correctness. It only matters for byte-level golden file matching.
