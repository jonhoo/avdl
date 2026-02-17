# `map<>` without value type produces unhelpful cascading errors

## Symptom

When a user writes `map<> data;` (empty type parameter for map),
the tool produces two cascading errors:

```
Error: unexpected token `>`
  help: expected one of: protocol, namespace, import, idl, schema, enum,
        fixed, error, record, array, map, union, boolean, int, long, float,
        double, string, bytes, null, ...

Error: unexpected ';' expected '?' or '>'
```

The first error dumps a huge expected-token list without explaining
that `map` requires a value type. The second error is a confusing
cascade. A user-friendly message would be:

```
map requires a value type parameter: use `map<type>` syntax
```

## Root cause

When ANTLR encounters `>` immediately after `<` in `map<>`, it tries
to match the `fullType` production inside the angle brackets. `>` is
not a valid start for a type, so ANTLR reports the full set of tokens
that could start a type declaration. There is no special-case
detection for this pattern in `refine_errors_with_source`.

The same issue applies to `array<>` (empty array type parameter).

## Affected files

- `src/reader.rs` -- `refine_errors_with_source` function

## Reproduction

```sh
cat > tmp/map-empty.avdl <<'EOF'
protocol Test {
  record Foo {
    map<> data;
  }
}
EOF
cargo run -- idl tmp/map-empty.avdl
```

Also test with `array<>`:

```sh
cat > tmp/array-empty.avdl <<'EOF'
protocol Test {
  record Foo {
    array<> items;
  }
}
EOF
cargo run -- idl tmp/array-empty.avdl
```

## Suggested fix

Add a new detection pattern in `refine_errors_with_source`: when the
offending token is `>` and the source text before it matches
`(map|array)\s*<\s*` (i.e., an empty type parameter), produce:

```
`map` (or `array`) requires a type parameter: use `map<string>` syntax
```

This is similar to how `detect_empty_union` already handles `union {}`.
