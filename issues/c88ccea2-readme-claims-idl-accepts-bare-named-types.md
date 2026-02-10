# README claims `idl` accepts bare named types, but it rejects them

## Symptom

The README.md states under "Known divergences from Java avro-tools":

> **Schema-mode leniency.** The `idl` subcommand accepts `.avdl` files
> containing bare named type declarations (no `protocol` or `schema`
> keyword), returning them as a JSON array.

This is inaccurate. Running `avdl idl` on a file with bare named types
(no `protocol` or `schema` keyword) results in:

```
Error: IDL file contains neither a protocol nor a schema declaration
```

## Root cause

The README text was written when the `idl` subcommand did accept bare
named types as a convenience extension. The CHANGELOG entry for v0.1.3
documents the fix:

> `avdl idl` now rejects IDL files that contain only bare named type
> declarations (no `protocol` or `schema` keyword), with a clear error
> message instead of silently producing an empty protocol.

The `compiler.rs` code explicitly rejects `IdlFile::NamedSchemas`:

```rust
if let IdlFile::NamedSchemas(_) = &idl_file {
    miette::bail!("IDL file contains neither a protocol nor a schema declaration");
}
```

The integration test `test_idl_rejects_bare_named_types` confirms this
behavior. The README's divergence bullet was never updated.

## Affected files

- `README.md` (the "Schema-mode leniency" bullet under "Known
  divergences from Java avro-tools")

## Reproduction

```sh
cat > tmp/bare.avdl <<'EOF'
namespace org.test;
record Foo { string name; }
enum Color { RED, GREEN, BLUE }
EOF
cargo run -- idl tmp/bare.avdl
# Error: IDL file contains neither a protocol nor a schema declaration
```

## Suggested fix

Remove or rewrite the "Schema-mode leniency" bullet in the README's
"Known divergences" section. The current behavior (rejecting bare named
types) matches Java avro-tools, so it is no longer a divergence.
