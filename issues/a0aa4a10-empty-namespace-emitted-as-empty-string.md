# Empty namespace annotation emits `"namespace": ""` instead of omitting the key

## Symptom

When `@namespace("")` is used (empty string), Rust emits
`"namespace": ""` in the JSON output, while Java omits the
`namespace` key entirely. This is a minor semantic difference in the
output.

```avdl
@namespace("")
protocol Simple {
  record TestRecord { string name; }
}
```

Rust output (excerpt):
```json
{
  "namespace": "",
  "protocol": "Simple",
  ...
}
```

Java output (excerpt):
```json
{
  "protocol": "Simple",
  ...
}
```

## Root cause

The JSON serialization code always emits the `namespace` field if it
is `Some(...)`, even when the value is the empty string. Java treats
an empty namespace as equivalent to no namespace and omits it.

## Affected files

- `src/model/json.rs` (protocol serialization)

## Reproduction

```sh
cat > tmp/empty-ns.avdl <<'EOF'
@namespace("")
protocol Simple { record TestRecord { string name; } }
EOF

cargo run -- idl tmp/empty-ns.avdl
# Output includes "namespace": ""

java -jar ../avro-tools-1.12.1.jar idl tmp/empty-ns.avdl
# Output omits namespace key
```

## Suggested fix

In the protocol-to-JSON serialization, treat an empty namespace the
same as no namespace and omit the `"namespace"` key from the output.
This matches Java's behavior.
