# Missing `fixDefaultValue` int-to-long coercion

## Symptom

When a field has type `long` (or a union containing `long` before
`int`), and its default value is an integer that fits in 32 bits, Java
coerces the JSON representation from an `IntNode` to a `LongNode`.
The Rust tool does not perform this coercion.

In practice, `serde_json` serializes both `i32` and `i64` values
identically as plain JSON numbers, so this has no effect on JSON
output. It is a semantic gap in the domain model only.

## Status

Re-opened during audit of deleted issues. Original issues:
`085a9c9f-missing-fixdefaultvalue-coercion.md` and
`missing-fix-default-value-int-to-long-promotion.md`.

## Evidence of partial fix (if any)

None. The TODO comment at `src/reader.rs:1000` still reads:
```
// TODO: implement fixDefaultValue — Java coerces IntNode to LongNode when
// the field type is `long`. In practice, serde_json serializes both the
// same way, so JSON output is unaffected.
```

## Remaining work

Add a `fix_default_value` function mirroring Java's `IdlReader.fixDefaultValue()`
(lines 641-662). When the parsed default is a `serde_json::Value::Number` that
fits in `i32` and the field type is `Long` (or a union where `Long` appears
before `Int`), promote the value to `i64`.

Priority is very low since JSON output is unaffected.

## Affected files

- `src/reader.rs` -- `walk_variable` function (around line 1000)

## Reproduction

```avdl
@namespace("test")
protocol P {
    record R {
        long count = 0;
        union { null, long } nullable_count = 0;
    }
}
```

JSON output is identical between Rust and Java. The difference is
only observable in the internal domain model (the `serde_json::Value`
is backed by `i32` instead of `i64`).
