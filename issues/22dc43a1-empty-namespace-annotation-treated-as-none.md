# `@namespace("")` treated as no namespace instead of explicitly empty

## Symptom

When a named type (record, enum, fixed) has `@namespace("")`, the Rust
tool treats this as `None` (no explicit namespace) and falls back to
the enclosing protocol/scope namespace. Java preserves the empty string
and emits `"namespace": ""` in the JSON output.

This causes two semantic problems:

1. **Wrong namespace in output**: A type declared with `@namespace("")`
   inside `@namespace("org.example") protocol P { ... }` gets serialized
   without a `"namespace"` key, which means a consumer reading the JSON
   will interpret it as inheriting `org.example`. Java emits
   `"namespace": ""` to explicitly indicate the type has no namespace.

2. **Over-lenient reference resolution**: Because the Rust tool assigns
   the enclosing namespace to `@namespace("")` types, unqualified
   references like `Status` resolve correctly even when they shouldn't
   (Java requires the correct name for the actual namespace).

## Root cause

Two locations in `reader.rs` explicitly filter out empty strings:

1. `compute_namespace` (line ~1570):
   ```rust
   explicit_namespace
       .as_ref()
       .filter(|s| !s.is_empty())
       .cloned()
   ```
   This converts `Some("")` to `None`, which callers then replace with
   the enclosing namespace via `.or_else(|| namespace.clone())`.

2. Schema-mode namespace handling (line ~455):
   ```rust
   *namespace = if id.is_empty() { None } else { Some(id) };
   ```

In Java, the equivalent `namespace()` method (IdlReader.java line 938)
returns `""` as-is (it only filters `null`, not empty strings), and
`Schema.createRecord(name, doc, "")` stores the empty string.

## Affected files

- `src/reader.rs` -- `compute_namespace` and schema-mode namespace
- `src/model/json.rs` -- namespace serialization logic (the
  `namespace.as_deref() != enclosing_namespace` check)
- `src/model/schema.rs` -- `AvroSchema` namespace field representation
- `src/resolve.rs` -- `SchemaRegistry` key computation

## Reproduction

```sh
cat > tmp/namespace-empty.avdl << 'EOF'
@namespace("org.example")
protocol P {
  @namespace("")
  record NoNamespace { string name; }
}
EOF

# Rust: omits "namespace" key entirely
cargo run -- idl tmp/namespace-empty.avdl
# Output: {"type":"record","name":"NoNamespace","fields":[...]}
# Missing: "namespace": ""

# Java: includes "namespace": ""
java -jar ../avro-tools-1.12.1.jar idl tmp/namespace-empty.avdl
# Output: {"type":"record","name":"NoNamespace","namespace":"","fields":[...]}

# Semantic diff:
diff <(cargo run --quiet -- idl tmp/namespace-empty.avdl | jq -S .) \
     <(java -jar ../avro-tools-1.12.1.jar idl tmp/namespace-empty.avdl | jq -S .)
```

The difference also surfaces when the `@namespace("")` type is
referenced from a different scope. Java rejects the unqualified
reference; Rust silently resolves it:

```sh
cat > tmp/namespace-empty-ref.avdl << 'EOF'
@namespace("org.example")
protocol P {
  @namespace("") record Unnamespaced { string name; }
  record Main { Unnamespaced item; }
}
EOF

# Rust: succeeds (wrong -- should fail or require FQN)
cargo run -- idl tmp/namespace-empty-ref.avdl

# Java: fails with "Undefined schema: org.example.Unnamespaced"
java -jar ../avro-tools-1.12.1.jar idl tmp/namespace-empty-ref.avdl
```

## Suggested fix

The fix needs changes in multiple layers:

### 1. Distinguish `None` from `Some("")` in `AvroSchema`

The `namespace: Option<String>` field currently conflates "no explicit
namespace" with "explicitly empty namespace". Either:
- Allow `Some("")` to be stored and propagated (simplest)
- Add a third variant (e.g., `ExplicitlyEmpty`)

### 2. Update `compute_namespace` in `reader.rs`

Remove the `.filter(|s| !s.is_empty())` so that `@namespace("")`
produces `Some("")` instead of `None`. The callers that do
`.or_else(|| namespace.clone())` will then correctly NOT fall back to
the enclosing namespace when the annotation explicitly said `""`.

### 3. Update `schema_to_json` in `json.rs`

The current condition:
```rust
if namespace.as_deref() != enclosing_namespace
    && let Some(ns) = namespace
```
already handles `Some("")` correctly: when enclosing is `Some("org.example")`
and type namespace is `Some("")`, the condition is true and `"namespace": ""`
is emitted. No change needed here.

### 4. Update `SchemaRegistry` key computation

The registry key for a type with `@namespace("")` should be the bare
name (no dot prefix), not `org.example.TypeName`. Verify that
`resolve.rs` handles this correctly after the `compute_namespace` change.

### 5. Optional: reject unqualified cross-namespace references

Java requires fully-qualified names for cross-namespace references.
The Rust tool currently resolves unqualified names by searching all
registered types, which is more lenient. This is a separate issue but
is a consequence of the namespace handling being wrong.

## Priority

Medium. This is a semantic correctness bug that produces wrong JSON
output. However, `@namespace("")` is rare in practice -- none of the
18 test `.avdl` files use it. The bug only affects types that
explicitly opt out of namespace inheritance.
