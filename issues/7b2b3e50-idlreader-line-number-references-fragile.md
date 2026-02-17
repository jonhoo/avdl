# Comments reference specific Java line numbers which are fragile

## Symptom

Two comments in `reader.rs` reference specific line numbers in the Java
`IdlReader.java` source file:

1. Line 3003: `exitNullableType (IdlReader.java lines 776-777)`
2. Line 3383: `exitMessageDeclaration (IdlReader.java line 715)`

These line numbers are correct as of the current `avro` submodule version
but will become stale when the submodule is updated.

## Root cause

The comments were written with exact line references for easy lookup. This
is helpful for a one-time port but becomes a maintenance burden when the
upstream file changes.

## Affected files

- `src/reader.rs`: lines 3003-3004 and 3383

## Reproduction

Update the `avro` submodule to a newer version and check whether lines
776-777 and 715 of `IdlReader.java` still correspond to the same code.

## Suggested fix

Replace line-number references with method name references, which are more
stable across upstream changes:

```rust
// Before:
// The Java implementation checks this in exitNullableType (IdlReader.java
// lines 776-777) and throws "Type references may not be annotated".

// After:
// Java checks this in exitNullableType() and throws
// "Type references may not be annotated".

// Before:
// implementation checks this in exitMessageDeclaration (IdlReader.java line 715).

// After:
// implementation checks this in exitMessageDeclaration().
```

This is a documentation-only change with zero risk.
