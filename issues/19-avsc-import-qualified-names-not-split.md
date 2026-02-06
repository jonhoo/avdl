# Qualified names in `.avsc` imports not split into name + namespace

## Symptom

When importing `.avsc` files whose `name` field contains a
fully-qualified name (e.g., `"name": "ns.other.schema.Baz"`), the
entire qualified string is stored as the schema's `name` field with
`namespace: None`. This produces incorrect JSON output:

```json
{"type": "record", "name": "ns.other.schema.Baz", ...}
```

The expected output is:

```json
{"type": "record", "name": "Baz", "namespace": "ns.other.schema", ...}
```

This also causes the `idl2schemata` subcommand to write files named
`ns.other.schema.Baz.avsc` instead of `Baz.avsc`, since the
`schema.name()` method returns the unsplit qualified name.

## Root cause

In `src/import.rs`, the `parse_record` function (line ~268) reads the
`name` field from the JSON object directly:

    let name = obj.get("name").and_then(|n| n.as_str())...;

It does not check whether the name contains dots. In Avro's JSON
format, the `name` field may contain a fully-qualified name (e.g.,
`"ns.other.schema.Baz"`), in which case the portion before the last
dot is the namespace and the portion after is the simple name. The
Java Avro library's `Schema.parse()` handles this by calling
`Schema.parseNameContext()` which splits qualified names.

The same issue applies to `parse_enum` and `parse_fixed`.

## Affected files

- `src/import.rs:263-312` -- `parse_record`
- `src/import.rs:314-361` -- `parse_enum`
- `src/import.rs:363-401` -- `parse_fixed`

## Reproduction

Test files `baz.avsc` and `foo.avsc` in the Avro test suite use
qualified names:

```sh
# baz.avsc contains: {"type": "record", "name": "ns.other.schema.Baz", ...}
# foo.avsc contains: {"type": "record", "name": "org.foo.Foo", ...}

cargo run -- idl \
  --import-dir avro/lang/java/idl/src/test/idl/input/ \
  --import-dir avro/lang/java/idl/src/test/idl/putOnClassPath/ \
  avro/lang/java/idl/src/test/idl/input/import.avdl /dev/stdout \
  | python3 -c "
import json, sys
d = json.load(sys.stdin)
for t in d['types']:
    if 'Baz' in t.get('name','') or 'Foo' in t.get('name',''):
        print(t['name'], t.get('namespace'))
"
# Actual:   ns.other.schema.Baz None / org.foo.Foo None
# Expected: Baz ns.other.schema  / Foo org.foo
```

## Suggested fix

In each of `parse_record`, `parse_enum`, and `parse_fixed`, after
extracting the `name` field, check if it contains a dot. If so, split
at the last dot: the prefix becomes the namespace (if no explicit
`"namespace"` key exists in the JSON) and the suffix becomes the
simple name. Something like:

```rust
let (name, inferred_ns) = if raw_name.contains('.') {
    let pos = raw_name.rfind('.').unwrap();
    (raw_name[pos+1..].to_string(), Some(raw_name[..pos].to_string()))
} else {
    (raw_name.to_string(), None)
};
let namespace = obj.get("namespace")
    .and_then(|n| n.as_str())
    .map(|s| s.to_string())
    .or(inferred_ns)
    .or_else(|| default_namespace.map(|s| s.to_string()));
```

This mirrors the Java `Schema.Name` constructor which splits
qualified names.
