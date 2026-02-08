# `full_name()` allocates a String on every call

## Symptom

`AvroSchema::full_name()` appears in multiple profiling stacks — it is
called from `SchemaRegistry::register`, `schema_to_json`,
`collect_named_types`, `collect_unresolved_refs`, and `union_type_key`.
Each call allocates a new `String` via `format!("{ns}.{name}")` or
`name.clone()`.

## Root cause

`full_name()` at `schema.rs:170` returns `Option<String>`, computing
the full name from scratch every time:

```rust
pub fn full_name(&self) -> Option<String> {
    // ...
    Some(match namespace {
        Some(ns) if !ns.is_empty() => format!("{ns}.{name}"),
        _ => name.clone(),
    })
}
```

This allocates on every call. For named types that are referenced
multiple times (looked up in the registry, serialized to JSON, checked
for duplicates in unions), the same string is computed and discarded
repeatedly.

## Affected files

- `src/model/schema.rs`: `AvroSchema::full_name()` (line 170–188)
- `src/resolve.rs`: callers in `register()`, `collect_unresolved_refs()`
- `src/model/json.rs`: callers in `schema_to_json()`,
  `collect_named_types()`

## Reproduction

No functional bug — observable under profiling. Search for
`AvroSchema::full_name` and `format!` in flamegraph stacks.

## Suggested fix

Several approaches, in order of preference:

1. **Cache the full name at construction.** Add a `full_name: String`
   field to `Record`, `Enum`, `Fixed`, and `Reference` variants,
   computed once when the schema is created. `full_name()` then
   returns `Option<&str>` (a borrow, zero allocation). This is the
   simplest change and eliminates all repeated allocation.

2. **Return `Cow<'_, str>`.** Change the return type to
   `Option<Cow<'_, str>>` — returning `Cow::Borrowed(name)` when
   there is no namespace, and `Cow::Owned(format!(...))` otherwise.
   This halves allocations (the no-namespace case is free) but still
   allocates for namespaced types.

3. **Store the full name as the canonical identifier.** Replace the
   separate `name` + `namespace` fields with a single `full_name`
   string and derive `name()`/`namespace()` by splitting on the last
   dot. This is a larger refactor but eliminates the duality entirely.

Option 1 is recommended as the lowest-risk, highest-impact change.
