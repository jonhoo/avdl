# Import cycle detection does not cover the initial input file

## Symptom

When a file imports itself (directly or via a chain), the tool produces
a confusing "duplicate schema name" error instead of silently skipping
the cyclic import. For example:

```avdl
// self_import.avdl
@namespace("test")
protocol P {
    import idl "self_import.avdl";
    record R { string name; }
}
```

Running `avdl idl self_import.avdl` produces:

    Error: duplicate schema name: test.R

The expected behavior is that the self-import is detected as a cycle and
silently skipped, producing normal output with just `R` in the types
array.

The same issue occurs with indirect cycles: if `a.avdl` imports
`b.avdl` and `b.avdl` imports `a.avdl`, the types from `a.avdl` are
registered twice (once from the recursive import and once from the
original processing), causing a duplicate name error.

## Root cause

The `parse_and_resolve` function in `src/main.rs` creates an
`ImportContext` but does not mark the initial input file as "imported"
before processing its declaration items. The cycle prevention mechanism
(`ImportContext::mark_imported`) only tracks files when they are
encountered as import targets in `resolve_single_import`.

When a file imports itself:

1. `parse_and_resolve("a.avdl")` starts -- `a.avdl` is NOT in
   `read_locations`
2. Processing `a.avdl`'s decl_items encounters
   `import idl "a.avdl"`
3. `resolve_single_import` calls `mark_imported("a.avdl")` -- first
   time, returns `false`, so it proceeds to parse `a.avdl` again
4. The second parse of `a.avdl` registers `R` in the registry
5. Back in step 2, the original `a.avdl`'s `DeclItem::Type(R)` tries
   to register -- duplicate error

The Java implementation handles this differently: `IdlReader` uses a
`readLocations` HashSet that includes the initial file before processing
begins.

## Affected files

- `src/main.rs` -- `parse_and_resolve` function (around line 256)

## Reproduction

```sh
# Self-import:
cat > /tmp/self_import.avdl << 'AVDL'
@namespace("test")
protocol P {
    import idl "self_import.avdl";
    record R { string name; }
}
AVDL
cargo run -- idl /tmp/self_import.avdl
# Error: duplicate schema name: test.R

# Mutual cycle:
cat > /tmp/cycle_a.avdl << 'AVDL'
@namespace("test")
protocol A {
    import idl "cycle_b.avdl";
    record TypeA { string name; }
}
AVDL
cat > /tmp/cycle_b.avdl << 'AVDL'
@namespace("test")
protocol B {
    import idl "cycle_a.avdl";
    record TypeB { string name; }
}
AVDL
cargo run -- idl /tmp/cycle_a.avdl
# Error: duplicate schema name: test.TypeA
```

## Suggested fix

In `parse_and_resolve`, after resolving the input file's canonical path,
add it to `import_ctx.read_locations` before calling
`process_decl_items`. This requires canonicalizing the input path, which
is already done in `read_input` (line 245: `dir.canonicalize()`). The
fix would be:

```rust
// After creating import_ctx, mark the input file as already imported
// to prevent cyclic self-imports.
if let Some(input_path) = input.as_ref() {
    if let Ok(canonical) = PathBuf::from(input_path).canonicalize() {
        import_ctx.mark_imported(&canonical);
    }
}
```

Alternatively, `read_input` could return the canonical path of the input
file alongside the source text and directory, and `parse_and_resolve`
could mark it immediately.

## Priority

Medium. Import cycles with the initial file cause confusing error
messages. While uncommon in practice, they are a known edge case that
the Java implementation handles correctly.
