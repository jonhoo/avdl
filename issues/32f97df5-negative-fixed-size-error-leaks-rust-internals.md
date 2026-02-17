# Negative fixed size error leaks internal Rust parse error message

## Symptom

When a user writes `fixed Hash(-5);`, the error message includes Rust's
internal `FromStr` error:

```
invalid fixed size for `Hash`: invalid integer '-5': invalid digit found in string
```

"invalid digit found in string" is the `Display` output of
`std::num::IntErrorKind::InvalidDigit` when parsing `-5` as `u64`.
This is confusing because `-5` does contain valid digits -- the real
issue is the minus sign (negative numbers aren't valid for fixed sizes).

## Root cause

The fixed size is parsed with `str::parse::<u64>()`, and the `Err`
variant is formatted directly into the error message. The underlying
Rust error distinguishes `InvalidDigit` from `PosOverflow`, but neither
produces a user-friendly message for this context.

## Affected files

- `src/reader.rs` -- the fixed size parsing code (look for
  `walk_fixed` or the fixed size parsing logic)

## Reproduction

```sh
cat > tmp/negative-fixed.avdl <<'EOF'
protocol Test {
  fixed Hash(-5);
}
EOF
cargo run -- idl tmp/negative-fixed.avdl
```

Produces:

```
invalid fixed size for `Hash`: invalid integer '-5': invalid digit found in string
```

## Suggested fix

Catch the specific case where the integer literal starts with `-` and
produce a targeted message like:

```
invalid fixed size for `Hash`: fixed size must be a non-negative integer, got `-5`
```

Alternatively, match on the `IntErrorKind` and map each variant to a
user-friendly message (e.g., "number is negative", "number is too
large").
