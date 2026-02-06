# Missing validation: nested unions are not rejected

## Symptom

The Rust tool silently accepts union types nested inside other union
types. The Avro specification explicitly forbids this: "Unions may not
immediately contain other unions." Java throws
`AvroRuntimeException("Nested union: ...")` at schema construction
time.

## Reproduction

Create a file `tmp/nested_union.avdl`:

```avdl
@namespace("test.nested")
protocol NestedUnion {
  record Inner {
    string value;
  }
  record Outer {
    union {
      null,
      union { string, int },
      Inner
    } deepField;
  }
}
```

```sh
cargo run -- idl tmp/nested_union.avdl tmp/nested_union.avpr
echo $?
# Actual: 0 (success)
# Expected: non-zero (error)
```

The Rust tool produces valid-looking JSON with a nested array:

```json
"type": [
  "null",
  [
    "string",
    "int"
  ],
  "Inner"
]
```

This JSON is semantically invalid according to the Avro specification,
and any downstream Avro tool that attempts to parse it would reject
the schema.

## Root cause

In `src/reader.rs`, the `walk_union_type` function collects the
constituent types of a union but does not check whether any of them
is itself a union. The Java implementation catches this at schema
construction time in `Schema.UnionSchema` (line 1258-1259 of
`Schema.java`):

```java
if (type.getType() == Type.UNION) {
  throw new AvroRuntimeException("Nested union: " + this);
}
```

The ANTLR grammar allows `fullType` (which includes `unionType`)
inside a union's type list, so the parser accepts the syntax. The
validation must happen at the semantic level.

## Affected files

- `src/reader.rs` -- `walk_union_type` function

## Suggested fix

After collecting all types in a union, check that none of them is
`AvroSchema::Union`:

```rust
for t in &types {
    if matches!(t, AvroSchema::Union { .. }) {
        return Err(make_diagnostic(
            src,
            &*ctx,
            "Unions may not immediately contain other unions",
        )
        .into());
    }
}
```

## Priority

Medium. While this is a spec violation, it only affects malformed
input files that no well-formed Avro IDL would produce. The existing
test suite does not exercise nested unions.
