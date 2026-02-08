# Duplicate `@namespace` annotations rejected instead of last-write-wins

## Symptom

When a named type declaration has two `@namespace` annotations (e.g.,
`@namespace("ns1") @namespace("ns2") record R { ... }`), the Rust tool
rejects with "duplicate @namespace annotation" while Java silently
uses the last value (last-write-wins via `LinkedHashMap.put`).

## Root cause

`walk_schema_properties` in `reader.rs` explicitly checks
`result.namespace.is_some()` and returns an error on the second
`@namespace`. Java's `SchemaProperties.addProperty` simply overwrites
`namespace = value.textValue()` without checking for a previous value.

The Rust behavior is also internally inconsistent: duplicate `@aliases`
uses last-write-wins (matching Java), while duplicate `@namespace`
rejects.

## Affected files

- `src/reader.rs` (around line 573)

## Reproduction

```sh
# File: tmp/edge-conflicting-namespace.avdl
@namespace("test.edge")
protocol ConflictingNs {
    @namespace("ns1")
    @namespace("ns2")
    record DualNs {
        string name;
    }
}
```

```sh
# Rust: REJECTS
cargo run -- idl tmp/edge-conflicting-namespace.avdl
# Error: duplicate @namespace annotation

# Java: ACCEPTS (uses ns2)
java -jar ../avro-tools-1.12.1.jar idl tmp/edge-conflicting-namespace.avdl
# Output: {"type":"record","name":"DualNs","namespace":"ns2",...}
```

## Suggested fix

Remove the `result.namespace.is_some()` check in `walk_schema_properties`
and simply overwrite with `result.namespace = Some(s.clone())`, matching
Java's last-write-wins semantics. Alternatively, keep the error but
document it as an intentional strictness improvement, and also add the
same check for `@aliases` for internal consistency.
