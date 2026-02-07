# Float literal suffixes (f/F/d/D) and hex floats not parsed

## Symptom

The Rust tool rejects floating-point literals with Java-style type
suffixes (`f`, `F`, `d`, `D`) and hexadecimal floating-point literals
(`0x1.0p10`). Java avro-tools handles these correctly by stripping the
suffix and parsing the numeric value.

Examples that fail in Rust but succeed in Java:
- `3.14f` (float suffix)
- `3.14F` (float suffix uppercase)
- `3.14d` (double suffix)
- `3.14D` (double suffix uppercase)
- `1e5f` (scientific notation with suffix)
- `0x1.0p10` (hex floating-point literal, parsed as 1024.0)

## Root cause

`parse_floating_point_literal` in `reader.rs` calls
`text.parse::<f64>()` directly. Rust's `f64::from_str` does not
understand Java-style type suffixes or hex float syntax. The ANTLR
grammar's `FloatingPointLiteral` rule explicitly allows `[fFdD]?`
suffixes and `HexadecimalFloatingPointLiteral` syntax.

The integer literal parser (`parse_integer_literal`) already handles
the analogous `L`/`l` suffix by stripping it before parsing, but
`parse_floating_point_literal` has no equivalent logic.

## Affected files

- `src/reader.rs`: `parse_floating_point_literal` function (around line 1508)

## Reproduction

```sh
cat > tmp/test-float-suffix.avdl <<'EOF'
protocol FloatSuffixProto {
  record R { float f = 3.14f; }
}
EOF

# Rust: fails with "invalid floating point literal '3.14f'"
cargo run -- idl tmp/test-float-suffix.avdl

# Java: succeeds, outputs default as 3.14
java -jar ../avro-tools-1.12.1.jar idl tmp/test-float-suffix.avdl
```

For hex floats:
```sh
cat > tmp/test-hex-float.avdl <<'EOF'
protocol HexFloatProto {
  record R { double d = 0x1.0p10; }
}
EOF

# Rust: fails
cargo run -- idl tmp/test-hex-float.avdl

# Java: succeeds, outputs default as 1024.0
java -jar ../avro-tools-1.12.1.jar idl tmp/test-hex-float.avdl
```

## Suggested fix

1. Strip trailing `[fFdD]` suffix before calling `text.parse::<f64>()`,
   similar to how `parse_integer_literal` strips `[lL]`.

2. For hex floats (`0x...p...`), implement a custom parser or use a
   crate like `hexf-parse` to convert hex floating-point notation to
   `f64`. Alternatively, manually parse the mantissa and exponent:
   `0x1.0p10` means `1.0 * 2^10 = 1024.0`.
