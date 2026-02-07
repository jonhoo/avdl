# Silent `u64` to `u32` truncation for fixed size and decimal precision/scale in JSON imports

## Symptom

When importing `.avsc` or `.avpr` files, the JSON import path silently
truncates `u64` values to `u32` using the `as u32` cast. If a JSON file
contains a `"size"` value exceeding `u32::MAX` (4,294,967,295), the
value wraps around silently, producing a corrupt fixed type with a
garbage size.

The same issue affects `"precision"` and `"scale"` for decimal logical
types. Additionally, `precision` defaults to 0 via `unwrap_or(0)`,
which violates the Avro spec requirement that precision must be >= 1.

## Root cause

In `src/import.rs`, the `parse_fixed` function uses:

```rust
let size = obj
    .get("size")
    .and_then(|s| s.as_u64())
    .ok_or_else(|| IdlError::Parse("fixed missing 'size'".to_string()))?
    as u32;
```

The `as u32` cast performs wrapping truncation without checking whether
the value fits. Similarly, `parse_annotated_primitive` uses:

```rust
let precision = obj.get("precision").and_then(|p| p.as_u64()).unwrap_or(0) as u32;
let scale = obj.get("scale").and_then(|s| s.as_u64()).unwrap_or(0) as u32;
```

## Affected files

- `src/import.rs:406-407` -- `parse_fixed`, `size` truncation
- `src/import.rs:484-486` -- `parse_annotated_primitive`, decimal
  precision/scale truncation and invalid default

## Reproduction

```sh
cat > tmp/big-fixed.avsc << 'EOF'
{"type": "fixed", "name": "BigHash", "size": 5000000000}
EOF
cargo run -- idl tmp/big-fixed-wrapper.avdl
# The imported fixed type will have size 705032704 instead of an error
```

## Suggested fix

Use `u32::try_from(val)` with proper error reporting:

```rust
let size_u64 = obj
    .get("size")
    .and_then(|s| s.as_u64())
    .ok_or_else(|| IdlError::Parse("fixed missing 'size'".to_string()))?;
let size = u32::try_from(size_u64)
    .map_err(|_| IdlError::Parse(format!(
        "fixed size {size_u64} exceeds maximum ({})", u32::MAX
    )))?;
```

For decimal precision, validate that it is >= 1:

```rust
let precision = obj.get("precision").and_then(|p| p.as_u64()).unwrap_or(0);
if precision < 1 {
    return Err(IdlError::Parse("decimal precision must be >= 1".to_string()));
}
let precision = u32::try_from(precision)
    .map_err(|_| IdlError::Parse("decimal precision too large".to_string()))?;
```

## Priority

Low. In practice, no valid Avro schema has a fixed size exceeding
`u32::MAX`, and decimal precision/scale values are always small
numbers. But the silent truncation is a latent data corruption bug
that should be fixed for robustness.
