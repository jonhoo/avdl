# `idl2schemata` emits `"namespace": ""` for empty-namespace types; Java omits it

## Symptom

When a type has an explicitly empty namespace (`@namespace("")`) and is
serialized as a standalone `.avsc` file via `idl2schemata`, the Rust
tool emits `"namespace": ""` while Java omits the `"namespace"` key
entirely.

In protocol mode (`idl`), both tools correctly emit `"namespace": ""`
because the enclosing protocol has a non-empty namespace and the empty
string is needed to override it. The discrepancy only appears in
standalone `.avsc` output where there is no enclosing namespace context.

## Root cause

Java's `Schema.Name` constructor normalizes empty-string namespaces
to `null`:

```java
if ("".equals(space))
    space = null;
```

Then `Name.writeName()` decides whether to emit the `"namespace"` key:

```java
if (space != null) {
    if (!space.equals(currentNamespace))
        gen.writeStringField("namespace", space);
} else if (currentNamespace != null) {
    gen.writeStringField("namespace", "");
}
```

When `space` is null (empty namespace) and `currentNamespace` is also
null (standalone `.avsc` with no enclosing context), neither branch
writes the namespace key.

In the Rust code (`src/model/json.rs`), the namespace is stored as
`Some("")` and the condition is:

```rust
if namespace.as_deref() != enclosing_namespace
    && let Some(ns) = namespace
{
    obj.insert("namespace".to_string(), Value::String(ns.clone()));
}
```

`Some("") != None` is `true`, so it writes `"namespace": ""`.

## Affected files

- `src/model/json.rs` -- the `schema_to_json` function for Record,
  Enum, and Fixed variants

## Reproduction

```sh
cat > tmp/edge-namespace-empty.avdl <<'EOF'
@namespace("org.example")
protocol NamespaceEmpty {
  @namespace("")
  record NoNamespace {
    string name;
  }
}
EOF

mkdir -p tmp/rust-out tmp/java-out
cargo run -- idl2schemata tmp/edge-namespace-empty.avdl tmp/rust-out/
java -jar ../avro-tools-1.12.1.jar idl2schemata \
  tmp/edge-namespace-empty.avdl tmp/java-out/

diff <(jq -S . tmp/rust-out/NoNamespace.avsc) \
     <(jq -S . tmp/java-out/NoNamespace.avsc)
```

Rust produces:
```json
{
  "type": "record",
  "name": "NoNamespace",
  "namespace": "",
  "fields": [{"name": "name", "type": "string"}]
}
```

Java produces:
```json
{
  "type": "record",
  "name": "NoNamespace",
  "fields": [{"name": "name", "type": "string"}]
}
```

## Suggested fix

In `schema_to_json`, when the enclosing namespace is `None`, treat an
empty-string namespace the same as `None` -- i.e., do not emit the
`"namespace"` key. The condition should be updated to something like:

```rust
let should_emit_namespace = match (namespace.as_deref(), enclosing_namespace) {
    (Some(ns), Some(enc)) if ns != enc => true,
    (Some(""), None) => false,   // <-- new: empty ns with no enclosing context
    (None, Some(_)) => true,     // null within non-null (emit "namespace": "")
    (Some(_), None) => true,
    _ => false,
};
```

Or more concisely, mirror Java's normalization: treat `Some("")` as
`None` when `enclosing_namespace` is also `None`.

Note: the protocol-mode behavior (`"namespace": ""` within a protocol
that has a non-null namespace) must remain unchanged, as both tools
agree on that.
