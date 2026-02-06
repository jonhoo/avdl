# Properties target wrong union branch after nullable reorder

## Symptom

When a nullable type `T?` has properties, `apply_properties` always
targets index `[1]` of the union (the non-null branch in the initial
`[null, T]` layout). But `fix_optional_schema` may reorder the union
to `[T, null]` when the field default is non-null. After reordering,
index `[1]` is `null`, so properties end up on the wrong branch.

## Root cause

`apply_properties` (line ~1216 in `reader.rs`) targets a hardcoded
index `[1]` without checking which branch is the non-null type.
Meanwhile, `fix_optional_schema` (line ~1197) may swap the order
before `apply_properties` runs.

## Location

- `src/reader.rs:1216-1232` — `apply_properties`
- `src/reader.rs:1197-1204` — `fix_optional_schema`
- Call site at `src/reader.rs:483` — order of operations

## Expected behavior

Properties should always land on the non-null branch of a nullable
union, regardless of reordering. Either:
- Apply properties before reordering, or
- Find the non-null branch by type rather than by index

## Difficulty

Moderate — straightforward logic fix once the correct order of
operations is determined.
