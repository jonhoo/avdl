# Unresolved type references in protocol messages are not validated

## Symptom

When a protocol message references an undefined type in its return
type, parameter types, or error (`throws`) types, the Rust tool
silently produces output containing the unresolved name as a bare
string. The Java avro-tools correctly rejects these with an
"Undefined schema" error.

For example:

```avdl
@namespace("test.edge")
protocol UndefReturn {
  DoesNotExist getUnknown();
}
```

Rust produces:

```json
{
  "messages": {
    "getUnknown": {
      "request": [],
      "response": "DoesNotExist"
    }
  },
  ...
}
```

Java rejects with: `AvroTypeException: Undefined schema: test.edge.DoesNotExist`

The same problem applies to:
- Undefined types in message parameters:
  `void process(DoesNotExist arg);`
- Undefined types in throws clauses:
  `void doThing() throws DoesNotExist;`

## Root cause

`validate_all_references` in `src/compiler.rs` (line 830) only checks
types registered in the `SchemaRegistry` via
`registry.validate_references()`. For `IdlFile::Protocol`, the match
arm at line 850 is empty -- it does not additionally validate
references in the `Protocol`'s `messages` field.

Message response types, request parameter types, and error types are
stored in the `Message` struct within the `Protocol` but are never
registered in the `SchemaRegistry`. Thus `collect_unresolved_refs`
never sees them.

## Affected files

- `src/compiler.rs` -- `validate_all_references` function (line 830)
- `src/resolve.rs` -- `validate_references` and `validate_schema`
  methods

## Reproduction

```sh
# Undefined return type:
cat > tmp/undef-return.avdl <<'EOF'
@namespace("test") protocol P { DoesNotExist get(); }
EOF
cargo run -- idl tmp/undef-return.avdl
# Succeeds silently (should error)

# Undefined parameter type:
cat > tmp/undef-param.avdl <<'EOF'
@namespace("test") protocol P { void f(DoesNotExist x); }
EOF
cargo run -- idl tmp/undef-param.avdl
# Succeeds silently (should error)

# Undefined error type:
cat > tmp/undef-error.avdl <<'EOF'
@namespace("test") protocol P { void f() throws DoesNotExist; }
EOF
cargo run -- idl tmp/undef-error.avdl
# Succeeds silently (should error)
```

Java rejects all three with `AvroTypeException: Undefined schema`.

## Suggested fix

In `validate_all_references`, add a `IdlFile::Protocol(protocol)`
match arm that iterates over `protocol.messages` and validates each
message's response, request field schemas, and error schemas using
`registry.validate_schema()`:

```rust
IdlFile::Protocol(protocol) => {
    for msg in protocol.messages.values() {
        unresolved.extend(registry.validate_schema(&msg.response));
        for field in &msg.request {
            unresolved.extend(registry.validate_schema(&field.schema));
        }
        if let Some(errors) = &msg.errors {
            for err_schema in errors {
                unresolved.extend(registry.validate_schema(err_schema));
            }
        }
    }
}
```

Additionally, add integration tests that verify undefined types in
messages produce an error rather than silently passing.
