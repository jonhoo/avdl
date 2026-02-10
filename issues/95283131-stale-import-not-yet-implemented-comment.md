# Stale "(not yet implemented)" comment on `ImportEntry` in reader.rs

## Symptom

The doc comment on `ImportEntry` at `reader.rs` line 735 says:

> Import type discovered during parsing. The actual import resolution is
> deferred to the `import` module (not yet implemented).

The import module (`src/import.rs`) is fully implemented and has been in
active use for a long time. The `ImportContext`, `import_protocol`, and
`import_schema` functions are all complete, and `compiler.rs` calls into
them for every import resolution. The parenthetical "(not yet
implemented)" is stale and misleading.

## Root cause

The comment was written during the initial scaffolding phase when import
resolution had not yet been built. It was never updated after the
`import` module was completed.

## Affected files

- `src/reader.rs` line 735

## Reproduction

Read the comment on `ImportEntry` and compare it to the state of
`src/import.rs`, which exports `ImportContext`, `import_protocol`, and
`import_schema` -- all fully functional.

## Suggested fix

Remove the parenthetical "(not yet implemented)" from the comment.
The corrected text would be:

```
/// Import type discovered during parsing. The actual import resolution is
/// deferred to the `import` module.
```
