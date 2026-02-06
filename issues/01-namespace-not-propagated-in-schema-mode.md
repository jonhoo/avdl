# Namespace not propagated to named schemas in schema mode

## Symptom

`test_status_schema` fails: the enum `Status` in `status_schema.avdl`
has `namespace: None` in the parsed model, but the expected output has
`"namespace": "system"`.

## Root cause

`status_schema.avdl` declares `namespace system;` followed by
`enum Status { ... }`. The reader sets `namespace = Some("system")` on
the walker context, but `walk_enum` (and likely `walk_record`,
`walk_fixed`) only sets the schema's namespace field when the schema
declaration has its own `@namespace` annotation — it does not inherit
the enclosing namespace when none is explicitly declared on the schema
itself.

## Location

- `src/reader.rs`: `walk_enum`, `walk_record`, `walk_fixed` —
  `compute_namespace` logic
- `status_schema.avdl` triggers this because it uses bare `namespace`
  syntax without per-schema `@namespace`

## Expected behavior

Named schemas should inherit the enclosing namespace (from `namespace`
declarations or `@namespace` on the protocol) when they don't have
their own explicit `@namespace` annotation. The `namespace` field on
the schema should be set to the inherited value.

## Difficulty

Easy — pass the enclosing namespace into walk_enum/walk_record/walk_fixed
and use it as the default when `compute_namespace` returns `None`.
