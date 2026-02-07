# Unqualified type names resolve across namespaces (Java requires qualified names)

## Symptom

When a type is defined with an explicit `@namespace` annotation that
differs from the protocol's namespace, the Rust tool allows
referencing it by its unqualified (short) name. Java requires the
fully-qualified name in this scenario.

For example, with protocol namespace `com.example` and a record
`@namespace("com.other") record DiffNs { ... }`, Java requires
`com.other.DiffNs` to reference it, while Rust accepts just `DiffNs`.

## Root cause

The Rust tool's type resolution (`SchemaRegistry` / `resolve.rs`)
searches all registered types by short name regardless of namespace.
The Java implementation qualifies unqualified names with the current
namespace (protocol namespace for top-level references) and then looks
up the fully-qualified name.

Specifically, Java's `IdlReader.name()` method calls
`fullName(namespace, typeName)` which prepends the current namespace
to unqualified names, then looks up the result. If the type exists
under a different namespace, the lookup fails with "Undefined schema".

## Affected files

- `src/resolve.rs`: `SchemaRegistry` type lookup logic
- `src/reader.rs`: where type references are resolved

## Reproduction

```sh
cat > tmp/test-unqual-cross-ns.avdl <<'EOF'
@namespace("com.example")
protocol UnqualCrossNsProto {
  @namespace("com.other")
  record DiffNs { string name; }

  record Container {
    DiffNs diff;
  }
}
EOF

# Rust: succeeds (resolves DiffNs by short name)
cargo run -- idl tmp/test-unqual-cross-ns.avdl

# Java: fails with "Undefined schema: com.example.DiffNs"
java -jar ../avro-tools-1.12.1.jar idl tmp/test-unqual-cross-ns.avdl
```

The fix is to use `com.other.DiffNs` as the field type:
```avdl
  record Container {
    com.other.DiffNs diff;
  }
```

## Suggested fix

When resolving an unqualified type reference, the resolver should
first qualify it with the current namespace and look up the
fully-qualified name. Only if the current namespace is empty should
it fall back to searching by short name. This matches Java's
`fullName()` → lookup behavior.

Care is needed to not break the common case where types share the
protocol's namespace (which should continue to resolve by short name).
The key change is: when the protocol has a namespace, unqualified
names should be qualified before lookup, not searched by short name
across all namespaces.
