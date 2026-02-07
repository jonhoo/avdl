# `collect_named_types` passes wrong namespace when recursing into record fields

## Symptom

In `build_lookup`, named types nested inside a record's fields are
registered with a lookup key based on the protocol-level
`default_namespace` instead of the record's own effective namespace.
This can cause reference resolution failures during JSON serialization
when a record has a different namespace from the protocol and contains
inline named type definitions in its fields.

## Root cause

In `src/model/json.rs`, `collect_named_types` correctly computes
`effective_ns` for the record itself (combining the record's
`namespace` with the `default_namespace`), but then passes the
original `default_namespace` when recursing into the record's fields:

```rust
let effective_ns = namespace.as_deref().or(default_namespace);
let full_name = match effective_ns {
    Some(ns) => format!("{ns}.{name}"),
    None => name.clone(),
};
lookup.insert(full_name, schema.clone());
for field in fields {
    collect_named_types(&field.schema, default_namespace, lookup);
    //                                 ^^^^^^^^^^^^^^^^^ should be effective_ns
}
```

Per the Avro spec, types nested inside a record inherit the record's
namespace when they don't have an explicit namespace of their own.

## Affected files

- `src/model/json.rs:116` -- `collect_named_types` field recursion

## Reproduction

This bug requires a specific pattern: a record with a different
namespace from the protocol, containing an inline named type definition
in one of its fields. Example:

```avdl
@namespace("org.example")
protocol P {
  @namespace("com.other")
  record Outer {
    enum InnerEnum { A, B } inner;
  }
}
```

Here `InnerEnum` should have full name `com.other.InnerEnum` (inheriting
from `Outer`), but `collect_named_types` registers it as
`org.example.InnerEnum`. A subsequent reference to `com.other.InnerEnum`
would fail to resolve in the lookup table.

No existing test file exercises this pattern. The `namespaces.avdl` test
has records in different namespaces but they have empty field lists or
reference types by FQN.

## Suggested fix

Pass `effective_ns` instead of `default_namespace` when recursing into
fields:

```rust
for field in fields {
    collect_named_types(&field.schema, effective_ns, lookup);
}
```

The same change should also be applied to the `Union` and `Array`/`Map`
recursive calls if they can appear inside a record with a non-default
namespace (they pass `default_namespace` too, which would have the same
issue).

## Priority

Medium. The bug is real but requires an unusual pattern (inline named
types inside records with non-default namespaces) that does not appear
in any existing test file.
