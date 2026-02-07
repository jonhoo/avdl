# Nullable array/map syntax (`array<T>?`, `map<T>?`) silently drops `?`

## Symptom

When using `array<string>?` or `map<int>?` in a field declaration,
the Rust tool silently ignores the `?` suffix and produces the
un-nullable array/map type. The field has `"default": null` but the
type is NOT wrapped in a union, making the schema semantically
invalid: a non-union field cannot have `null` as its default value.

Example:

```avdl
@namespace("test")
protocol P {
  record R {
    array<string>? optionalList = null;
  }
}
```

Rust output:

```json
{
  "name": "optionalList",
  "type": {"type": "array", "items": "string"},
  "default": null
}
```

Expected (if nullable were supported):

```json
{
  "name": "optionalList",
  "type": ["null", {"type": "array", "items": "string"}],
  "default": null
}
```

Java 1.12.1 crashes on this input with a `NullPointerException` in
`exitVariableDeclaration`.

## Root cause

The ANTLR grammar (`Idl.g4`) only supports `?` on primitive types
and named type references, not on arrays or maps:

```
plainType: arrayType | mapType | unionType | nullableType;
nullableType: (primitiveType | referenceName=identifier) optional=QuestionMark?;
```

The `arrayType` and `mapType` alternatives in `plainType` do not
include an optional `QuestionMark`. When the parser encounters
`array<string>?`, ANTLR successfully parses `array<string>` as an
`arrayType`, then the `?` token does not match any rule and is
silently skipped via ANTLR's error recovery mechanism.

The Rust tool does not detect or report this ANTLR error recovery.
The `?` token is consumed but has no effect on the resulting schema.

## Affected files

- `src/reader.rs` -- `walk_plain_type`, `walk_full_type`
- The ANTLR grammar itself (`Idl.g4`) does not support this syntax

## Reproduction

```sh
cat > tmp/nullable-array.avdl << 'EOF'
@namespace("test")
protocol P {
  record R {
    array<string>? optionalList = null;
    map<int>? optionalMap = null;
  }
}
EOF

# Rust silently drops the `?`:
cargo run -- idl tmp/nullable-array.avdl
# Output: type is bare array/map, not union with null

# Java crashes:
java -jar ../avro-tools-1.12.1.jar idl tmp/nullable-array.avdl
# NullPointerException
```

## Suggested fix

Two options:

1. **Error detection**: After parsing, check for ANTLR error recovery
   tokens and report parse errors instead of silently proceeding.
   This would reject `array<T>?` with a clear error message.

2. **Extended support**: Handle `?` on any type (not just primitives
   and references) by adding `QuestionMark?` to the full type rule
   or to `plainType`. This would be a grammar extension beyond what
   Java supports.

Option 1 is more conservative and matches Java's behavior (both
reject the input, just Rust would do so gracefully instead of
silently producing wrong output).

## Priority

Low. This only affects input that is technically invalid IDL syntax.
However, the silent acceptance with semantically wrong output is
worse than crashing or erroring -- users may not realize their
`array<T>?` is not actually nullable.
