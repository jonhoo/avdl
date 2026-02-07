# Unqualified type references resolve across namespaces incorrectly

## Symptom

When a type is declared with an explicit `@namespace` that differs from
the enclosing protocol namespace, the Rust tool accepts unqualified
(short name) references to that type from the protocol namespace. Java
rejects such references with "Undefined schema" and requires the
fully-qualified name.

Additionally, the Rust tool serializes the cross-namespace reference as
a short name in the JSON output, which is semantically wrong -- a
consumer reading the JSON would resolve the short name relative to the
enclosing namespace and fail to find the type.

Example:

```avdl
@namespace("org.example")
protocol P {
  @namespace("org.other")
  record OtherRecord { string name; }

  record MainRecord {
    OtherRecord other;  // Java: error; Rust: accepts
  }
}
```

Rust output (wrong):
```json
{
  "name": "MainRecord",
  "fields": [{
    "name": "other",
    "type": "OtherRecord"  // Should be "org.other.OtherRecord"
  }]
}
```

Java requires the IDL to use `org.other.OtherRecord` and produces:
```json
{
  "name": "MainRecord",
  "fields": [{
    "name": "other",
    "type": "org.other.OtherRecord"
  }]
}
```

## Root cause

The Rust tool's type reference resolution in `resolve.rs` (and the
`SchemaRegistry` lookup) does not scope lookups to the current
namespace. When it encounters an unqualified name like `OtherRecord`,
it:

1. First tries `{current_namespace}.OtherRecord` (i.e.,
   `org.example.OtherRecord`).
2. If that fails, it falls back to searching all registered types by
   short name, finding `org.other.OtherRecord`.

This fallback search is more lenient than Java, which strictly resolves
`OtherRecord` as `org.example.OtherRecord` and fails if that type
doesn't exist.

The serialization side has a related bug: `ref_name()` in `json.rs`
uses the short name when the reference's namespace matches the
enclosing namespace, but the fallback resolution means the reference's
namespace might NOT match -- the reference was resolved by short name
against a type in a different namespace.

## Affected files

- `src/resolve.rs` -- `SchemaRegistry` type lookup / resolution
- `src/model/json.rs` -- `ref_name()` and `schema_to_json` for
  `Reference` variants
- `src/reader.rs` -- type reference creation in `walk_nullable_type`

## Reproduction

```sh
cat > tmp/crossns.avdl << 'EOF'
@namespace("org.example")
protocol P {
  @namespace("org.other")
  record OtherRecord { string name; }
  record MainRecord { OtherRecord other; }
}
EOF

# Rust: succeeds but produces wrong reference in JSON
cargo run -- idl tmp/crossns.avdl
# "type": "OtherRecord" instead of "type": "org.other.OtherRecord"

# Java: rejects with "Undefined schema: org.example.OtherRecord"
java -jar ../avro-tools-1.12.1.jar idl tmp/crossns.avdl
```

## Suggested fix

### Option A: Match Java strictness (recommended)

Remove the fallback short-name search from the schema registry. When an
unqualified name is used, resolve it ONLY as
`{current_namespace}.{name}`. If the user wants to reference a type in
a different namespace, they must use the fully-qualified name.

This matches Java behavior and produces correct JSON output.

### Option B: Keep lenient resolution but fix serialization

If leniency is desired, keep the fallback search but ensure that the
JSON serialization uses the fully-qualified name for cross-namespace
references. This means `ref_name()` must compare the resolved type's
actual namespace against the enclosing namespace, not just the
reference's stored namespace.

### Interaction with issue 22dc43a1

This issue is partially caused by the same root cause as the
`@namespace("")` issue (22dc43a1). When `@namespace("")` is treated
as `None`, the type inherits the protocol namespace, so cross-namespace
resolution works "by accident." Fixing 22dc43a1 will cause some of
these cases to start failing (correctly), but the fallback short-name
search is a separate problem that also affects non-empty namespace
mismatches.

## Priority

Medium. The JSON output is semantically wrong for cross-namespace
references: a consumer parsing the JSON would fail to resolve the
short-name reference. The existing golden test files don't exercise
this pattern (the `namespaces.avdl` test uses fully-qualified names),
so no tests fail currently.
