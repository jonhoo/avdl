# Inconsistent `SYSTEM_ERROR` handling in message error unions

## Symptom

The Java implementation internally adds `Protocol.SYSTEM_ERROR`
(`Schema.create(Schema.Type.STRING)`) as the first element of every
two-way message's error union. When serializing to JSON, it then elides
this element (serializes only the errors after `SYSTEM_ERROR`). When
parsing `.avpr` JSON back, it adds `SYSTEM_ERROR` back.

The Rust implementation does not add `SYSTEM_ERROR` at all -- neither
internally nor during serialization. This means the `Message.errors`
field in the Rust model differs from Java's internal representation.

## Detailed analysis

### IdlReader.java (building the error union)

Lines 718-725:

```java
List<Schema> errorSchemas = new ArrayList<>();
errorSchemas.add(Protocol.SYSTEM_ERROR);  // Always prepended
for (IdentifierContext errorContext : ctx.errors) {
    errorSchemas.add(namedSchemaOrUnresolved(...));
}
message = protocol.createMessage(..., Schema.createUnion(errorSchemas));
```

### Protocol.java TwoWayMessage.toJson1 (serializing)

Lines 231-236:

```java
List<Schema> errs = errors.getTypes(); // elide system error
if (errs.size() > 1) {
    Schema union = Schema.createUnion(errs.subList(1, errs.size()));
    gen.writeFieldName("errors");
    union.toJson(knownNames, namespace, gen);
}
```

So `SYSTEM_ERROR` is added internally but stripped during JSON output.

### Protocol.java MessageParsing (reading .avpr)

Lines 664-665:

```java
List<Schema> errs = new ArrayList<>();
errs.add(SYSTEM_ERROR); // every method can throw
```

When reading a `.avpr` file back, `SYSTEM_ERROR` is re-added.

## Impact on serialization

The Rust output **correctly matches the golden files** because Java
elides `SYSTEM_ERROR` during serialization. The Rust tool produces
`"errors": ["TestError"]` which matches the golden file.

## Impact on protocol import

When importing a `.avpr` file, Java re-adds `SYSTEM_ERROR` to the
error union. The Rust `import_protocol` function in `src/import.rs`
reads the errors array as-is from the JSON, without adding
`SYSTEM_ERROR`. This means:

1. If a message in an imported `.avpr` has `"errors": ["SomeError"]`,
   Java's internal model has `["string", "SomeError"]` but Rust's has
   `["SomeError"]`.

2. This difference is invisible in the final JSON output (both elide
   `SYSTEM_ERROR` during serialization).

3. However, if any future validation or analysis code inspects the
   `Message.errors` field expecting `SYSTEM_ERROR` to be present, it
   would find the wrong thing in Rust.

## Impact on non-throwing messages

The Java `TwoWayMessage.toJson1` only writes `"errors"` if
`errs.size() > 1`. For a non-throwing two-way message, the internal
model has `["string"]` (just `SYSTEM_ERROR`), so `errs.size() == 1`
and the `"errors"` key is omitted.

The Rust implementation omits `"errors"` when `errors` is `None`,
which produces the same result.

## Affected files

- `src/reader.rs` -- `walk_message` (model construction)
- `src/import.rs` -- `json_to_message` (protocol import)

## Suggested fix

This is a model-level concern, not a serialization bug. Two options:

1. **Do nothing for now**: The JSON output is correct. Add a TODO
   noting the internal model difference for future reference.

2. **Match Java's model**: Add `AvroSchema::String` as the first
   element of `Message.errors` for two-way messages, and strip it
   during JSON serialization. This would match Java's internal
   representation exactly.

Option 2 would also require updating `message_to_json` in
`model/json.rs` to skip the first error (if it is `"string"`) when
serializing.

## Priority

Low. The serialized output is correct. This is only relevant for
internal model fidelity and for future validation features that may
inspect the error union.
