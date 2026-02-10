# No range validation for int/long default values

## Symptom

Default values for `int` fields that overflow the 32-bit signed
integer range are silently accepted:

```avro
@namespace("test")
protocol P {
  record R {
    int x = 9999999999;
  }
}
```

Our tool produces `"default": 9999999999` in the JSON output, which is
not a valid Avro `int` value (max 2,147,483,647). Similarly, `long`
defaults exceeding the 64-bit signed integer range would be accepted.

Java's `IdlTool` crashes with a `NoSuchElementException` on the same
input (an upstream bug, see `upstream-issues/` if tracked), so there is
no "correct" reference behavior to compare against. However, a proper
implementation should reject out-of-range integer defaults with a clear
error message.

## Root cause

The `is_valid_default` function in `src/model/schema.rs` (line 333)
validates `Int` and `Long` defaults with:

```rust
AvroSchema::Int | AvroSchema::Long => {
    matches!(value, Value::Number(n) if is_json_integer(n))
}
```

This checks that the JSON value is an integer (not floating-point) but
does not check whether the value fits in the schema's numeric range
(`-2^31` to `2^31-1` for `int`, `-2^63` to `2^63-1` for `long`).

## Affected files

- `src/model/schema.rs` (line 333, `is_valid_default` function)

## Reproduction

```sh
cat > tmp/test-int-overflow.avdl <<'EOF'
@namespace("test")
protocol P {
  record R {
    int x = 9999999999;
    int y = -2147483649;
    long z = 99999999999999999999;
  }
}
EOF
cargo run -- idl tmp/test-int-overflow.avdl
# Expected: error about out-of-range default
# Actual: silently accepted with the out-of-range value in JSON
```

## Suggested fix

Split the `Int` and `Long` cases in `is_valid_default` and add range
checks:

```rust
AvroSchema::Int => {
    matches!(value, Value::Number(n) if n.is_i64()
        && n.as_i64().map_or(false, |v| v >= i32::MIN as i64 && v <= i32::MAX as i64))
}
AvroSchema::Long => {
    matches!(value, Value::Number(n) if is_json_integer(n))
}
```

This matches the Avro specification which defines `int` as a 32-bit
signed integer and `long` as a 64-bit signed integer. Note that
`Long` can keep the existing check since `serde_json::Number::is_i64()`
already ensures the value fits in 64 bits.
