# Protocol name not validated against INVALID_TYPE_NAMES

## Symptom

The Rust tool accepts reserved type names (like `null`, `int`, `string`,
`date`, etc.) as protocol names. Java rejects them with
`SchemaParseException: Illegal name: <name>`.

## Root cause

Java's `enterProtocolDeclarationBody` calls `name(protocolIdentifier)`,
which in turn calls `validateName(name, true)`. The second parameter
(`isTypeName=true`) triggers the `INVALID_TYPE_NAMES` check:

```java
private String validateName(String name, boolean isTypeName) {
    if (name == null) {
        throw new SchemaParseException("Null name");
    } else if (!VALID_NAME.test(name)) {
        throw new SchemaParseException("Illegal name: " + name);
    }
    if (isTypeName && INVALID_TYPE_NAMES.contains(name)) {
        throw new SchemaParseException("Illegal name: " + name);
    }
    return name;
}
```

The Rust `walk_protocol` in `reader.rs` calls `extract_name` on the
protocol identifier but never checks against `INVALID_TYPE_NAMES`. The
check is present for records (line 767), enums (line 926), and fixed
types (line 994), but not for protocols.

## Affected files

- `src/reader.rs` -- `walk_protocol` function (around line 667)

## Reproduction

```sh
cat > tmp/test-invalid-protocol-name.avdl <<'EOF'
protocol `null` {
}
EOF
cargo run -- idl tmp/test-invalid-protocol-name.avdl
# Produces: {"protocol": "null", "types": [], "messages": {}}
# Java throws: SchemaParseException: Illegal name: null
```

Other reserved names that are accepted:

```sh
# All of these should be rejected but are accepted:
protocol `int` { }
protocol `string` { }
protocol `date` { }
protocol `uuid` { }
```

## Suggested fix

Add the same `INVALID_TYPE_NAMES` check used for records/enums/fixed
to the protocol name extraction in `walk_protocol`:

```rust
let protocol_name = extract_name(&raw_identifier);
if INVALID_TYPE_NAMES.contains(&protocol_name.as_str()) {
    return Err(make_diagnostic(
        src,
        &*name_ctx,
        format!("Illegal name: {protocol_name}"),
    ));
}
```

Priority: **Low**. Protocol names don't appear in the schema type
system, so a reserved protocol name doesn't cause downstream issues.
However, it is a parity gap with Java and could confuse users who
accidentally use a reserved word.
