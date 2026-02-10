# Full name computation pattern duplicated across 8+ call sites

## Symptom

The expression `match namespace { Some(ns) if !ns.is_empty() => format!("{ns}.{name}"), _ => name.clone() }` (or slight variations) is repeated in at least 8 locations across 4 modules. An `AvroSchema::full_name()` method already exists in `schema.rs` (line 179) but is not used by most of these sites.

## Root cause

The full name computation pattern was written inline in each module
before the `full_name()` method was added. Some call sites operate on
destructured fields (`name` and `namespace` as separate variables)
rather than on an `&AvroSchema`, so calling `full_name()` directly
would require restructuring.

## Affected files

- `src/model/json.rs`:
  - `collect_named_types` Record arm (line 133-136)
  - `collect_named_types` Enum/Fixed arm (line 152-155)
  - `schema_to_json` Record arm (line 222-225)
  - `schema_to_json` Enum arm (line 292-295)
  - `schema_to_json` Fixed arm (line 351-354)
  - `schema_to_json` Reference arm (line 499)
- `src/resolve.rs`:
  - `collect_unresolved_refs` Reference arm (line 229)
- `src/model/schema.rs`:
  - `resolve_for_validation` Reference arm (around line 517)

The canonical implementation is `AvroSchema::full_name()` at
`src/model/schema.rs` line 179.

## Reproduction

Search the codebase for `format!("{ns}.{name}")` -- every hit is this
pattern.

## Suggested fix

Introduce a free function `fn make_full_name(name: &str, namespace: Option<&str>) -> Cow<'_, str>` (or similar) at the `model/schema` level, then rewrite `AvroSchema::full_name()` to delegate to it. Each call site that currently operates on destructured `name`/`namespace` fields can call the free function directly.

This would reduce ~8 inline copies to 1, each ~3 lines, for a total of roughly 24 duplicated lines collapsed to a single definition.
