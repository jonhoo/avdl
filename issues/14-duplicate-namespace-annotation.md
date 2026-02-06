# Duplicate `@namespace` annotations silently overwrite

## Symptom

If multiple `@namespace` annotations appear on the same schema
declaration, the last one silently wins. No error is reported.

## Root cause

In `walk_schema_properties`, the `"namespace"` match arm
unconditionally sets `result.namespace = Some(s.clone())` without
checking if it was already set.

## Location

- `src/reader.rs:158-166` — namespace handling in
  `walk_schema_properties`

## Expected behavior

Return an error if `result.namespace` is already `Some` when a
second `@namespace` is encountered.

## Difficulty

Easy — add a guard check before the assignment.
