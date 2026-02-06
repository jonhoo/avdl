# Error messages lack source location context

## Symptom

Parse and interpretation errors use `IdlError::Other(String)` which
provides no source file location information. When users have errors
in their `.avdl` files, the error messages don't point to the
offending line/column.

## Root cause

The `ParseDiagnostic` struct in `error.rs` supports miette source
spans (`NamedSource`, `SourceSpan`), but it is not used in most error
paths. Over 20 call sites in `reader.rs` use
`IdlError::Other("message".into())` instead.

## Location

- `src/error.rs:5-13` — `ParseDiagnostic` (exists but underused)
- `src/reader.rs` — 20+ sites using `IdlError::Other`:
  - "missing protocol name" (line ~289)
  - "missing protocol body" (line ~303)
  - "missing property name" (line ~148)
  - "missing property value" (line ~154)
  - etc.

## Expected behavior

Create a helper function that extracts token position/span from ANTLR
parse tree contexts and constructs `ParseDiagnostic` with proper
source spans. Convert error sites systematically.

## Difficulty

Hard — systematic work across 20+ sites, needs ANTLR token position
extraction and source code threading.
