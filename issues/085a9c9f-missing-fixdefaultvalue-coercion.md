# Missing `fixDefaultValue` int-to-long coercion

## Symptom

When a field has type `long` (or a union containing `long` before
`int`), and its default value is an integer that fits in 32 bits, Java
coerces the JSON representation from an `IntNode` to a `LongNode`.
The Rust tool does not perform this coercion, which could produce
semantically different JSON output in edge cases.

## Root cause

Java's `IdlReader.fixDefaultValue()` (lines 641-662) checks whether
the parsed default value is an `IntNode` and the field type is `long`
or a union whose first `long` branch comes before any `int` branch.
If so, it replaces the `IntNode` with a `LongNode`.

The Rust code has no equivalent of this method. The `walk_variable`
function in `reader.rs` passes the default value through as-is from
`walk_json_value`.

## Impact on output

In practice, serde_json serializes both `i32` and `i64` values as
plain numbers (e.g., `0` vs `0`), so for values that fit in an `i32`
the JSON output is byte-identical. The difference only manifests if:

1. A downstream consumer distinguishes between 32-bit and 64-bit JSON
   numbers at the model level (not at the serialized string level).
2. The value is used in a context where the numeric type matters
   (e.g., Avro binary encoding, which uses different encodings for
   int vs long).

For the JSON output produced by `avdl idl` and `avdl idl2schemata`,
this is likely a no-op difference because both values serialize to
the same JSON text. But it represents a semantic gap in the domain
model that could matter if the model is used for other purposes.

## Affected files

- `src/reader.rs` -- `walk_variable` (no coercion step)

## Reproduction

```avdl
@namespace("test")
protocol P {
    record R { long x = 0; }
}
```

In Java, the default value `0` is an `IntNode` from the parser, then
coerced to `LongNode(0)` by `fixDefaultValue`. In Rust, it stays as
`serde_json::Value::Number(0)`.

The JSON output is identical (`"default": 0`) because both serialize
the same way, but the internal model differs.

## Suggested fix

Add a `fix_default_value` function in `reader.rs` that mirrors the
Java logic: if the default is an integer value and the field type is
`Long` or a union where `Long` appears before `Int`, convert the
`serde_json::Value::Number` to ensure it's stored as i64. This is
low priority since the JSON output is unaffected.
