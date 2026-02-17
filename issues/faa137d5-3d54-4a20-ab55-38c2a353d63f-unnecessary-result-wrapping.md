# Remove unnecessary `Result`/`Option` wrapping from infallible functions

## Symptom

`cargo clippy --all-targets -- -W clippy::pedantic` reports `clippy::unnecessary_wraps`
for functions that always return `Ok(...)` or `Some(...)` and never produce an error
or `None`.

## Root cause

These functions were likely written with error handling in mind for future
extensibility, but currently they never fail.

## Affected files

- `src/compiler.rs:519` — `Idl2Schemata::extract_impl` returns
  `miette::Result<SchemataOutput>` but always returns `Ok(...)`. The `Result`
  wrapper serves no purpose since the body is infallible.

- `src/import.rs:411` — `string_to_schema` returns `Result<AvroSchema>` but
  every code path returns `Ok(...)`. The function dispatches between primitive
  parsing and reference construction, both of which are infallible.

- `src/reader.rs:826` — `append_quoting_hint` returns `Option<String>` but
  always returns `Some(...)`. This function always produces a hint string,
  never `None`.

## Reproduction

```sh
cargo clippy --all-targets -- -W clippy::unnecessary_wraps
```

## Suggested fix

For each function:

1. **`extract_impl`**: Change return type to `SchemataOutput`, remove `Ok()`
   wrapper. Update caller to not use `?`.

2. **`string_to_schema`**: Change return type to `AvroSchema`, remove all `Ok()`
   wrappers. Update callers to not use `?`.

3. **`append_quoting_hint`**: Change return type from `Option<String>` to
   `String`, remove `Some()` wrapper. Update callers that currently unwrap or
   match on the return value.

If the `Result` wrapping is intentional for future error handling, add
`#[allow(clippy::unnecessary_wraps)]` with a comment explaining the design
intent.
