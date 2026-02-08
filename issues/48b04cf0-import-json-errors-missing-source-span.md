# Import JSON parsing errors lack source spans

## Symptom

Errors in `import.rs` for malformed `.avpr`/`.avsc` JSON (missing
required fields, unknown schema types, etc.) use `miette::miette!()`
without source spans. The error message describes the problem but
doesn't point at the IDL source location of the import statement or
the JSON location of the malformed content.

## Root cause

Import resolution happens in `import.rs`, which parses JSON files
and constructs schemas. Errors here are about the *imported* file's
content, not the IDL source. There are two distinct span contexts:

1. The IDL source span of the `import` statement — available via
   `ImportEntry.span` (added in the rich-error-diagnostics work).
2. The JSON source span within the imported file — not tracked.

Currently neither span is used for these errors.

## Affected files

- `src/import.rs` — JSON parsing and schema construction
- `src/main.rs` — `resolve_single_import()` wraps import errors

## Suggested fix

For errors about the imported file's structure, wrap with the IDL
import statement's span so the user at least sees which import
triggered the failure. The `.wrap_err_with()` on `resolve_import()`
already uses `ParseDiagnostic` for path-not-found errors; extend
this to cover JSON parsing failures too.

For errors about specific JSON content (wrong field type, missing
key), adding JSON source spans would require a different approach
(e.g., `miette::NamedSource` pointing at the JSON file content).
This is a larger effort and lower priority.
