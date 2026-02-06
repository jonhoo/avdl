# `primitive_from_str` panics instead of returning an error

## Symptom

The `primitive_from_str` function in `import.rs` uses
`unreachable!()` for unknown type names, which panics at runtime.

## Root cause

The function's return type is `AvroSchema` (not `Result`), so it
can't return errors. The comment says "this is only called with values
we've already matched as primitives" — but the invariant is not
enforced by the type system.

## Location

- `src/import.rs:493-505` — `primitive_from_str` function

## Expected behavior

Return `Result<AvroSchema, IdlError>` and propagate errors to
callers. Both call sites (lines 472 and 489) are already in
`Result`-returning functions.

## Difficulty

Easy.
