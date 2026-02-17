# `ImportContext` fields are `pub` but only accessed within `import.rs`

## Symptom

`ImportContext::read_locations` and `ImportContext::import_dirs` are declared
`pub` (effectively `pub(crate)` since the `import` module is `pub(crate)`),
but neither field is accessed from outside `import.rs`. All external code
uses the constructor (`ImportContext::new`) and methods (`resolve_import`,
`mark_imported`) instead.

## Root cause

The fields were likely declared `pub` early in development when the module
boundary was less clear. They were never narrowed after the API stabilized.

## Affected files

- `src/import.rs`: lines 57-59

## Reproduction

```
grep -n '\.read_locations\|\.import_dirs' src/*.rs src/**/*.rs
```

All hits are within `import.rs` itself.

## Suggested fix

Change the field visibility from `pub` to private (no visibility qualifier):

```rust
pub struct ImportContext {
    /// Files that have already been imported (canonical paths, for cycle prevention).
    read_locations: HashSet<PathBuf>,
    /// Additional directories to search for imports (replaces Java classpath).
    import_dirs: Vec<PathBuf>,
}
```

This is a safe change since no code outside the module accesses the fields.
Per the Rust API guidelines (C-STRUCT-PRIVATE), structs should avoid
exposing implementation details through public fields when methods provide
the needed access.
