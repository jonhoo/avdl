# Import warnings are not propagated to the user

## Symptom

When Java processes an `import idl` statement, it recursively parses
the imported file and collects any warnings from that file. These
warnings are propagated back to the top-level `IdlFile` with the
import filename prepended. The Rust tool does not propagate warnings
from imported files.

## Root cause

Java's `exitImportStatement` (line 430 of `IdlReader.java`) calls:

```java
warnings.addAll(idlFile.getWarnings(importFile));
```

where `getWarnings(importFile)` prepends the import filename to each
warning. This means if an imported file has a stray doc comment, the
warning appears in the top-level output as:

```
Warning: nestedimport.avdl line 1, char 1: Ignoring out-of-place
documentation comment.
```

The Rust `resolve_single_import` in `main.rs` has no warning
collection mechanism at all. The `parse_idl` function does not
return warnings, and the import resolution code does not attempt to
collect them.

This is partially dependent on the out-of-place doc comment warning
system (issue 7f3435cb): since the Rust code does not generate doc
comment warnings at all, there are no warnings to propagate. However,
even if that issue is fixed, the propagation path would still need to
be added.

## Impact on output

Affects stderr only. Users importing IDL files with stray doc comments
will not see warnings about those issues. This is a user experience
gap rather than a correctness issue.

## Affected files

- `src/reader.rs` -- `parse_idl` does not return warnings
- `src/main.rs` -- `resolve_single_import` does not collect warnings

## Suggested fix

1. Add a warnings collection to `parse_idl`'s return value.
2. In `resolve_single_import` for IDL imports, collect warnings from
   the recursively parsed file and propagate them upward.
3. In `run_idl` and `run_idl2schemata`, print collected warnings to
   stderr.

This depends on implementing the out-of-place doc comment warning
system first (issue 7f3435cb).
