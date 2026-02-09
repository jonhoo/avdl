# Improve ANTLR error messages with semantic suggestions

## Symptom

Raw ANTLR parse errors pass through as-is without semantic enrichment.
For example, an unknown annotation `@beta` produces:

```
no viable alternative at input '@betarecord'
```

...which is technically correct but unhelpful because ANTLR merges
`@beta` and `record` into a single token (the grammar does not
recognize `@beta` as a standalone annotation).

## Current status

All Pattern 2 cases (default value validation errors) now include
source spans via `make_diagnostic` in `walk_variable` and
`ParseDiagnostic` in `process_decl_items`. Those are resolved.

Pattern 1 remains: ANTLR syntax errors have source spans (since
`07d8d21`), making Rust strictly better than Java in presentation.
The remaining work is semantic enrichment of the error *text* itself.

## Suggested fix

After collecting the raw ANTLR error, pattern-match on known error
shapes in `CollectingErrorListener::syntax_error()` or in the
post-parse error formatting:

- If the error text contains `@<word>` where `<word>` is not a
  recognized annotation name, suggest: "unknown annotation `@<word>`"
- If the error mentions `'?'` as extraneous input, explain nullable
  syntax limitations in the current context

Low priority: Rust already produces strictly better errors than Java
for all known cases. This is a polish item.

## Affected files

- `src/reader.rs` — `CollectingErrorListener::syntax_error()` and
  the post-parse error formatting
