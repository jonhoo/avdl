# Stale `IdlFile` variant names in comments (`SchemaFile`, `NamedSchemasFile`)

## Symptom

Two comments reference `IdlFile` variant names that do not exist:

1. **`resolve.rs` line 196** says `IdlFile::SchemaFile`, but the actual
   variant is `IdlFile::Schema`.

2. **`compiler.rs` line 820** says "`SchemaFile` and `NamedSchemasFile`
   store their top-level schemas outside the registry", but the actual
   variants are `IdlFile::Schema` and `IdlFile::NamedSchemas`.

In both cases, the code immediately below the comment uses the correct
variant names (`IdlFile::Schema`, `IdlFile::NamedSchemas`), creating a
visible contradiction between comment and code.

## Root cause

The `IdlFile` enum variants were likely renamed at some point (perhaps
from `SchemaFile`/`NamedSchemasFile` to the shorter `Schema`/
`NamedSchemas`), and these two comments were not updated to match.

## Affected files

- `src/resolve.rs` line 196
- `src/compiler.rs` line 820

## Reproduction

```sh
grep -n 'SchemaFile\|NamedSchemasFile' src/resolve.rs src/compiler.rs
```

Shows the stale names. Compare with:

```sh
grep -n 'IdlFile::Schema\|IdlFile::NamedSchemas' src/reader.rs src/compiler.rs
```

which shows the actual variant names used in code.

## Suggested fix

In `resolve.rs` line 196, change `IdlFile::SchemaFile` to
`IdlFile::Schema`.

In `compiler.rs` line 820, change `SchemaFile` to `Schema` and
`NamedSchemasFile` to `NamedSchemas`.
