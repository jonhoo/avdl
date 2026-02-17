# `void` as field type produces misleading "Undefined name: void" error

## Symptom

When a user writes `void nothing;` as a record field, the error says:

```
Undefined name: void
```

This is misleading because `void` is a recognized keyword (valid as a
message return type), not an unknown type name. The user may search for
a definition of `void` or try to import it, when the real issue is that
`void` can only be used as a message return type.

## Root cause

In the ANTLR grammar, `void` is only part of the `resultType` rule
(for message return types), not `nullableType` or `fullType`. When used
as a field type, the parser treats the identifier `void` as a named
type reference, which then fails resolution as "Undefined name".

The compiler's type resolution does not distinguish between keywords
used in the wrong context and genuinely unknown type names.

## Affected files

- `src/resolve.rs` or `src/compiler.rs` -- type resolution logic
- `src/reader.rs` -- could add special-case detection

## Reproduction

```sh
cat > tmp/void-field.avdl <<'EOF'
protocol Test {
  record Foo {
    void nothing;
  }
}
EOF
cargo run -- idl tmp/void-field.avdl
```

Produces: `Undefined name: void`

Java produces the same unhelpful error (`Undefined schema: void`), so
this is an opportunity for the Rust tool to provide a better experience.

## Suggested fix

In the "did you mean?" suggestion logic for undefined names, add a
special case: if the undefined name is `void`, produce:

```
`void` can only be used as a message return type, not as a field type
```

This could be a simple string check before the edit-distance
suggestion logic.
