# `try_promote_logical_type` only handles `AnnotatedPrimitive`

## Symptom

`try_promote_logical_type` in `reader.rs` only promotes
`AnnotatedPrimitive` variants to `AvroSchema::Logical`. Java also
validates logical type compatibility on non-primitive schemas (e.g.,
`duration` on `fixed(12)`).

The JSON output is semantically correct — this is a validation gap,
not an output gap. Invalid logical type annotations on non-primitive
bases silently pass through as plain properties rather than being
promoted or warned about.

## Root cause

The promotion function only pattern-matches on `AnnotatedPrimitive`
and returns all other variants unchanged.

## Affected files

- `src/reader.rs` (`try_promote_logical_type`)

## Reproduction

```avdl
protocol Test {
  @logicalType("duration") fixed Duration(12);
}
```

Java promotes this to a logical type; Rust leaves it as a fixed with
a `logicalType` property.

## Suggested fix

Extend `try_promote_logical_type` to also handle `Fixed` (for
`duration`) and potentially other non-primitive bases. Add validation
that the base type matches what the logical type expects (e.g.,
`duration` requires `fixed(12)`, `decimal` requires `bytes` or
`fixed`).
