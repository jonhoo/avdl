# Missing validation: one-way messages must return void

## Summary

The Rust tool silently accepts one-way messages that declare a
non-void return type. The Java implementation validates that one-way
messages return `void` and throws `SchemaParseException` with the
message "One-way message '...' must return void" if they do not.

## Reproduction

Given this input file:

```avdl
@namespace("test")
protocol OneWayTest {
  record Msg { string text; }
  Msg send(Msg m) oneway;
}
```

```sh
cargo run -- idl oneway_nonvoid.avdl tmp/out.avpr
```

**Expected:** The tool exits with a non-zero status and an error
message like "One-way message 'send' must return void".

**Actual:** The tool succeeds (exit code 0) and produces JSON with
both `"one-way": true` and `"response": "Msg"`, which is
semantically invalid:

```json
{
  "messages": {
    "send": {
      "one-way": true,
      "request": [
        {
          "name": "m",
          "type": "Msg"
        }
      ],
      "response": "Msg"
    }
  },
  "namespace": "test",
  "protocol": "OneWayTest",
  "types": [
    {
      "fields": [{ "name": "text", "type": "string" }],
      "name": "Msg",
      "type": "record"
    }
  ]
}
```

The Avro specification states that one-way messages must have a `null`
response and no errors. Producing a protocol JSON with `"one-way":
true` and a non-null response type would cause downstream Avro tools
to reject the protocol.

## Root cause

In `src/reader.rs`, the `walk_message` function (around line 972)
checks `ctx.oneway.is_some()` to set the `one_way` flag on the
`Message` struct, but it does not validate that the return type is
`void` (`AvroSchema::Null`). The Java implementation performs this
check in `exitMessageDeclaration` at `IdlReader.java` line 715:

```java
if (ctx.oneway != null) {
  if (returnType.getType() != Schema.Type.NULL) {
    throw error("One-way message'" + name + "' must return void",
                ctx.returnType.start);
  }
}
```

## Suggested fix

In `walk_message` in `src/reader.rs`, after extracting the return
type and checking `one_way`, add a validation:

```rust
let one_way = ctx.oneway.is_some();
if one_way && response != AvroSchema::Null {
    return Err(make_diagnostic(
        src,
        &*ctx,
        &format!("One-way message '{}' must return void", message_name),
    )
    .into());
}
```

## Affected files

- `src/reader.rs` -- `walk_message` function, around line 972-973
