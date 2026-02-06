# Messages from `import protocol` not merged

## Symptom

When a `.avdl` file uses `import protocol "foo.avpr"`, the imported
protocol's types are registered in the schema registry but its
messages are discarded.

## Root cause

In `src/main.rs`, the `ImportKind::Protocol` handler calls
`import_protocol` which returns the imported messages, but they are
stored in `_messages` (unused):

    let _messages = import_protocol(&resolved_path, registry)...
    // TODO: merge imported messages into the current protocol.

## Location

- `src/main.rs:270-279` — protocol import handling
- `tests/integration.rs:72-85` — similar TODO in test infrastructure

## Expected behavior

Imported protocol messages should be merged into the current
protocol's message map, matching Java's behavior.

## Difficulty

Moderate — requires threading messages back to the protocol builder.
The `import_protocol` function already returns the messages; they just
need to be collected and merged.
