# Messages from `import idl` not merged into enclosing protocol

## Symptom

When a `.avdl` file uses `import idl "other.avdl"`, the imported
protocol's types are merged into the schema registry, but its
messages are discarded. This is visible in `import.avdl` which
imports `reservedwords.avdl` (via `import idl`); the golden output
includes 7 messages from `reservedwords.avdl` (`error`, `void`,
`idl`, `import`, `oneway`, `null`, `local_timestamp_ms`), but the
actual output omits all of them.

Golden `import.avpr` messages: `error`, `void`, `idl`, `import`,
`oneway`, `null`, `local_timestamp_ms`, `bar`, `bazm`, `barf`.
Actual output messages: `bazm`, `barf` only.

## Root cause

In `src/main.rs`, the `ImportKind::Idl` handler (line ~288) calls
`parse_idl` which returns `(imported_idl, imported_registry,
nested_imports)`. The `imported_idl` is an `IdlFile::ProtocolFile`
containing the imported protocol's messages, but those messages are
never extracted or merged into the enclosing protocol:

    let (imported_idl, imported_registry, nested_imports) =
        parse_idl(&imported_source)...;
    registry.merge(imported_registry);
    // Messages from imported_idl are silently dropped.

Issue 09 covers the same problem for `import protocol` (where the
`import_protocol` function returns messages that are stored in
`_messages` and discarded). This issue specifically covers `import
idl`, which has a different code path.

## Affected files

- `src/main.rs:288-313` -- `ImportKind::Idl` handler in
  `resolve_imports`

## Reproduction

```sh
cargo run -- idl \
  --import-dir avro/lang/java/idl/src/test/idl/input/ \
  --import-dir avro/lang/java/idl/src/test/idl/putOnClassPath/ \
  avro/lang/java/idl/src/test/idl/input/import.avdl /dev/stdout \
  | python3 -c "import json,sys; d=json.load(sys.stdin); print(sorted(d['messages'].keys()))"
# Actual:   ['barf', 'bazm']
# Expected: ['bar', 'barf', 'bazm', 'error', 'idl', 'import',
#            'local_timestamp_ms', 'null', 'oneway', 'void']
```

## Suggested fix

When `imported_idl` is an `IdlFile::ProtocolFile`, extract its
messages and collect them for merging into the enclosing protocol.
This requires threading a mutable messages map through
`resolve_imports`, similar to how `registry` is threaded for type
merging. Both `ImportKind::Idl` and `ImportKind::Protocol` (issue 09)
should feed into the same message merge path.
