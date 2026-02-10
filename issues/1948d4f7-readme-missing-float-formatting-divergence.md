# README omits float formatting divergence from Java

## Symptom

The README's "Intentional divergences from Java" section documents
cosmetic JSON differences (whitespace, key ordering) but does not
mention that floating-point default values are formatted differently.

## Details

The tool uses `serde_json::to_string_pretty` for all JSON output.
For float/double default values, serde_json's formatting differs from
Java's `JsonGenerator` (which delegates to `Double.toString()`):

| Value         | Java output  | Rust output          |
|---------------|--------------|----------------------|
| `-1.0e12`     | `-1.0E12`    | `-1000000000000.0`   |
| `5.0E-324`    | `4.9E-324`   | `5.0E-324`           |

Java uses scientific notation for values where |value| >= 1e7 or
|value| < 1e-3. Rust/serde_json uses decimal notation in all cases
where the value fits. The subnormal case (`5.0E-324` vs `4.9E-324`)
reflects different shortest-representation algorithms for the same
IEEE 754 bit pattern (`0x0000000000000001`).

Both representations parse to the same `f64` value — the difference
is purely cosmetic, like the whitespace and key-ordering differences
already documented.

## History

A `JavaPrettyFormatter` that matched Java's `Double.toString()` style
was added, removed (937ce33), restored (1ad93b2), and removed again.
The current position is that byte-identical output is a non-goal and
integration tests compare parsed `serde_json::Value` trees, making
the formatter unnecessary.

## Suggested fix

Add a bullet to the "Intentional divergences from Java" section of
`README.md`, e.g.:

> **Float formatting uses serde_json defaults.** Java renders large
> or small float/double values in scientific notation (e.g.,
> `-1.0E12`); this tool uses `serde_json`'s decimal representation
> (e.g., `-1000000000000.0`). Both parse to the same numeric value.

## Affected files

- `README.md` — add the missing bullet
