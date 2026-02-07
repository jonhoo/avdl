# `idl2schemata` does not warn about unresolved type references

## Symptom

The `idl` subcommand validates that all type references resolve to
registered schemas and prints a warning to stderr when they do not:

```
warning: unresolved type references: test.Missing
```

The `idl2schemata` subcommand does NOT perform this validation. When a
file with unresolved type references is processed by `idl2schemata`, the
unresolved names appear as bare strings in the output `.avsc` files
without any warning.

## Root cause

The `run_idl` function in `src/main.rs` (lines 132-138) calls
`registry.validate_references()` after writing output. The
`run_idl2schemata` function (lines 147-203) omits this check entirely.

## Affected files

- `src/main.rs` -- `run_idl2schemata` function (around line 200)

## Reproduction

```sh
cat > /tmp/unresolved.avdl << 'AVDL'
@namespace("test")
protocol P {
    record R { Missing m; }
}
AVDL

# idl warns:
cargo run -- idl /tmp/unresolved.avdl /tmp/out.avpr 2>&1 | grep warning
# Output: warning: unresolved type references: test.Missing

# idl2schemata does not warn:
cargo run -- idl2schemata /tmp/unresolved.avdl /tmp/outdir/ 2>&1 | grep warning
# No output (no warning)
```

## Suggested fix

Add the same `validate_references()` check to `run_idl2schemata`, after
the schema output loop:

```rust
let unresolved = registry.validate_references();
if !unresolved.is_empty() {
    eprintln!(
        "warning: unresolved type references: {}",
        unresolved.join(", ")
    );
}
```

This is a one-line addition that mirrors the existing logic in
`run_idl`.

## Priority

Low. The primary use case for `idl2schemata` is extracting schemas from
valid IDL files where all references resolve. However, consistency
between the two subcommands is desirable, and the missing warning could
silently produce invalid `.avsc` files.
