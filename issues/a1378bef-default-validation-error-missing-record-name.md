# Default validation error from reader.rs omits enclosing record name

## Symptom

When a field with a primitive type has an invalid default value, the
error message from the reader.rs path does not include the enclosing
record name:

```
Invalid default for field `count`: expected int, got string
```

But when a field with a Reference type (enum, record, fixed) has an
invalid default, the error message from the compiler.rs path includes
the enclosing record name:

```
Invalid default for field `nested` in `Outer`: expected record Inner, got string
```

This inconsistency means users sometimes get the record name for
context and sometimes don't, depending on the field type. The record
name is especially useful when multiple records have fields with
similar names.

## Root cause

Two separate validation paths produce different message formats:

1. **`reader.rs` line 1722** (primitive defaults, validated during
   parsing):
   ```
   "Invalid default for field `{field_name}`: {reason}"
   ```
   The enclosing record name is not included because `walk_variable`
   doesn't receive the record name as a parameter.

2. **`compiler.rs` line 638** (Reference defaults, validated after type
   registration):
   ```
   "Invalid default for field `{field_name}` in `{type_name}`: {reason}"
   ```
   This path has access to the schema's `full_name()`.

## Affected files

- `src/reader.rs`: `walk_variable()`, around line 1718-1723
- `src/compiler.rs`: `process_decl_items()`, around line 636-638

## Reproduction

```sh
# Primitive default (reader.rs path -- no record name in error)
cat > tmp/err-invalid-default.avdl <<'EOF'
protocol Test {
  record Foo {
    int count = "not-a-number";
  }
}
EOF
cargo run -- idl tmp/err-invalid-default.avdl 2>&1

# Reference default (compiler.rs path -- includes record name)
cat > tmp/err-ref-default.avdl <<'EOF'
protocol Test {
  record Inner { string x; }
  record Outer {
    Inner nested = "bad";
  }
}
EOF
cargo run -- idl tmp/err-ref-default.avdl 2>&1
```

## Suggested fix

Thread the enclosing record name (or full name) into `walk_variable()`
so the error message can include "in `{record_name}`", matching the
compiler.rs format. The record name is available in the calling
`walk_record()` function and can be passed down as an additional
parameter.
