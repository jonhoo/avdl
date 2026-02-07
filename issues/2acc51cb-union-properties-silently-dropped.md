# Properties silently dropped for non-nullable unions

## Symptom

`@foo("bar") union { string, int }` silently loses the `@foo`
annotation. No error or warning is produced.

## Root cause

In `apply_properties_to_schema`, the match arm for non-nullable unions
(`other => other`) discards properties without applying them. Nullable
unions (two-element unions with null) have special handling that applies
properties to the non-null branch, but plain unions do not.

Java also rejects annotations on unions at a different level, so the
behavior is consistent in practice. However, silently dropping
annotations is surprising — a warning or error would be better.

## Affected files

- `src/reader.rs` — `apply_properties_to_schema` function

## Reproduction

```sh
cat > tmp/union-props.avdl <<'EOF'
protocol Test {
  record Foo {
    @custom("value") union { string, int } field1;
  }
}
EOF
cargo run -- idl tmp/union-props.avdl
# Annotation is silently dropped from output
```

## Suggested fix

Add a warning when properties are present but cannot be applied to a
non-nullable union. Alternatively, add a TODO comment documenting the
intentional omission.

Low priority — annotations on non-nullable unions are very rare in
practice and Java also doesn't support them.
