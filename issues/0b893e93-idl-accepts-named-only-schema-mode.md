# `idl` subcommand accepts schema-mode files with only named types

## Symptom

The `idl` subcommand accepts schema-mode `.avdl` files that contain
only named type declarations (records, enums, fixed) without a
`schema` keyword or `protocol` declaration. Java's `idl` subcommand
rejects such files with "the IDL file does not contain a schema nor
a protocol."

For example, this file is accepted by Rust but rejected by Java:

```avro
namespace org.test;

record Foo {
  string name;
}

enum Color {
  RED, GREEN, BLUE
}
```

Rust produces a JSON array of the named types. Java rejects it.

## Root cause

The `Idl::convert_impl` method in `compiler.rs` only rejects
`NamedSchemas` when the list is empty (import-only files). It does
not check whether the file lacks both a `schema` keyword and a
`protocol` declaration.

Java's `IdlTool.run()` checks `if (m == null && p == null)`, where
`m` is the main schema (from the `schema` keyword) and `p` is the
protocol. A file with only named type declarations has both as null,
so Java rejects it.

## Affected files

- `src/compiler.rs` -- `Idl::convert_impl`, lines ~205-212

## Reproduction

```sh
cat > tmp/test-named-only.avdl << 'EOF'
namespace org.test;
record Foo { string name; }
enum Color { RED, GREEN, BLUE }
EOF

# Rust (succeeds -- should fail):
cargo run -- idl tmp/test-named-only.avdl

# Java (fails as expected):
java -jar avro-tools-1.12.1.jar idl tmp/test-named-only.avdl
# Error: the IDL file does not contain a schema nor a protocol.
```

Note: `idl2schemata` correctly accepts this file in both Rust and
Java -- the fix should only affect the `idl` subcommand.

## Suggested fix

In `Idl::convert_impl`, change the `NamedSchemas` rejection from
"only if empty" to "always reject for the `idl` subcommand". The
current check:

```rust
if let IdlFile::NamedSchemas(schemas) = &idl_file {
    if schemas.is_empty() {
        // ...reject...
    }
}
```

Should become:

```rust
if let IdlFile::NamedSchemas(_) = &idl_file {
    // ...reject...
}
```

This makes `idl` reject all `NamedSchemas` files (whether empty or
not), matching Java's `IdlTool.run()` behavior. The `idl2schemata`
path (`Idl2Schemata::extract_impl`) intentionally omits this check,
so it will continue to accept named-only files.
