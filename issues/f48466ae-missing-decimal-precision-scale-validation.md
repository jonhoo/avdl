# Missing validation for decimal precision and scale

## Symptom

The Rust tool accepts `decimal(0)` (zero precision) and
`decimal(5, 10)` (scale greater than precision) without error,
producing JSON with these invalid values. Java avro-tools rejects
both cases:

- `decimal(0)` -> "Invalid decimal precision: 0 (must be positive)"
- `decimal(5, 10)` -> "Invalid decimal scale: 10 (greater than precision: 5)"

The Avro specification requires decimal precision to be a positive
integer and scale to be zero or a positive integer not exceeding
precision.

## Root cause

In `walk_primitive_type`, the code parses precision and scale as
`u32` values via `parse_integer_as_u32` but does not validate that:

1. `precision > 0`
2. `scale <= precision`

The Java reference implementation performs these checks in
`LogicalTypes$Decimal.validate()`, which is called when the decimal
logical type is added to the schema. The Rust code has no equivalent
validation step.

## Affected files

- `src/reader.rs` — `walk_primitive_type`, around line 1662 where
  decimal precision and scale are parsed

## Reproduction

```sh
# Zero precision:
cat > tmp/test-decimal-zero.avdl <<'EOF'
protocol Test {
  record Foo {
    decimal(0) value;
  }
}
EOF

# Rust accepts:
cargo run -- idl tmp/test-decimal-zero.avdl
# Output includes: "precision": 0, "scale": 0

# Java rejects:
java -jar avro-tools-1.12.1.jar idl tmp/test-decimal-zero.avdl
# Exception: Invalid decimal precision: 0 (must be positive)

# Scale > precision:
cat > tmp/test-decimal-scale.avdl <<'EOF'
protocol Test {
  record Foo {
    decimal(5, 10) value;
  }
}
EOF

# Rust accepts:
cargo run -- idl tmp/test-decimal-scale.avdl
# Output includes: "precision": 5, "scale": 10

# Java rejects:
java -jar avro-tools-1.12.1.jar idl tmp/test-decimal-scale.avdl
# Exception: Invalid decimal scale: 10 (greater than precision: 5)
```

## Suggested fix

After parsing precision and scale in `walk_primitive_type`, add
validation checks before constructing the `Logical` schema:

```rust
let precision = parse_integer_as_u32(precision_tok.get_text())
    .map_err(|e| { /* existing error handling */ })?;

if precision == 0 {
    return Err(make_diagnostic(
        src,
        &**precision_tok,
        "invalid decimal precision: 0 (must be positive)",
    ));
}

let scale = if let Some(scale_tok) = ctx.scale.as_ref() {
    parse_integer_as_u32(scale_tok.get_text())
        .map_err(|e| { /* existing error handling */ })?
} else {
    0
};

if scale > precision {
    return Err(make_diagnostic(
        src,
        ctx.scale.as_ref().expect("scale token present"),
        format!(
            "invalid decimal scale: {scale} \
             (greater than precision: {precision})"
        ),
    ));
}
```
