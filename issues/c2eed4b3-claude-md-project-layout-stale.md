# CLAUDE.md project layout section missing `compiler.rs` and `suggest.rs`

## Symptom

The "Project layout" section in `CLAUDE.md` lists the `src/` directory
contents but is missing two source files:

- `compiler.rs` -- the public API (`Idl`, `Idl2Schemata` builders) and
  shared compilation preamble
- `suggest.rs` -- Levenshtein edit distance and suggestion threshold helpers

Both are referenced by `lib.rs` as `pub(crate) mod` declarations and are
integral parts of the codebase.

## Root cause

The project layout documentation was written before `compiler.rs` and
`suggest.rs` were split out from other modules. It was not updated when
these files were added.

## Affected files

- `CLAUDE.md`: the "Project layout" section (around line 208)

## Reproduction

Compare:
```
ls src/*.rs | sort
```
against the file list in CLAUDE.md's "Project layout" section.

## Suggested fix

Add the missing entries to the project layout section:

```
src/
  main.rs               CLI (lexopt): `idl` and `idl2schemata` subcommands
  lib.rs                 Module declarations
  compiler.rs            Public API: Idl and Idl2Schemata builders, compilation pipeline
  reader.rs              Core ANTLR tree walker — the heart of the parser
  suggest.rs             Levenshtein edit distance for "did you mean?" suggestions
  model/
    ...
```
