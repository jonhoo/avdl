# `decimal` without parameters produces misleading "Undefined name" error

## Symptom

When a user writes `decimal value;` without the required precision
parameters, the error says:

```
Undefined name: decimal
```

This is misleading because `decimal` is a recognized keyword (for
`decimal(precision, scale)` logical types). The user may think `decimal`
is not supported, when the real issue is missing parameters.

## Root cause

In the ANTLR grammar, `decimal` with parameters (`decimal(10,2)`) is
handled by a specific production rule that includes `LParen`. Without
parentheses, `decimal` falls through to the `identifier` /
`referenceName` alternative in `nullableType`, and then fails type
resolution as an undefined named type.

The compiler's type resolution does not distinguish between keywords
requiring parameters (used without them) and genuinely unknown names.

## Affected files

- `src/resolve.rs` or `src/compiler.rs` -- type resolution
- `src/reader.rs` -- could add special-case detection

## Reproduction

```sh
cat > tmp/decimal-no-params.avdl <<'EOF'
protocol Test {
  record Foo {
    decimal value;
  }
}
EOF
cargo run -- idl tmp/decimal-no-params.avdl
```

Produces: `Undefined name: decimal`

Java produces the same unhelpful error (`Undefined schema: decimal`).

## Suggested fix

In the undefined-name suggestion logic, add a special case: if the
undefined name is `decimal`, produce:

```
`decimal` requires precision and scale parameters: use `decimal(precision, scale)` syntax
```

For example, `decimal(10, 2)` for a decimal with precision 10 and
scale 2. This is similar to how `@beta` without `(value)` already
gets a specialized "missing its value" message.
