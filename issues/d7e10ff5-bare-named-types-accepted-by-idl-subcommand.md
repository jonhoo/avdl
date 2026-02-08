# `idl` subcommand accepts bare named types that Java CLI rejects

## Symptom

When an `.avdl` file contains bare named type declarations (record,
enum, fixed) without a `protocol` wrapper and without a `schema
<type>;` declaration, the Rust `idl` subcommand outputs a JSON array
of all named schemas and exits 0. The Java `IdlTool` CLI rejects
this with "Error: the IDL file does not contain a schema nor a
protocol." and exits 1.

## Root cause

The Rust code has an `IdlFile::NamedSchemasFile` variant that was
implemented to match Java's `IdlFile.outputString()` method (used by
the test harness). However, Java's actual `IdlTool` CLI only checks
`getProtocol()` and `getMainSchema()`, both of which are `null` for
this case:

```java
// IdlTool.java:83-85
if (m == null && p == null) {
    err.println("Error: the IDL file does not contain a schema nor a protocol.");
    return 1;
}
```

The `outputString()` method (used by `TestIdlReader`) does support
this case, outputting a JSON array of named schemas. But this is a
test-only path, not the actual CLI behavior.

Meanwhile, Java's `IdlToSchemataTool` (the `idl2schemata` command)
uses `getNamedSchemas()` which works correctly with bare named types,
so `idl2schemata` handles this case in both Java and Rust.

## Affected files

- `src/main.rs` -- `run_idl` outputs JSON array for
  `IdlFile::NamedSchemasFile`
- `src/reader.rs` -- `walk_idl_file` produces
  `IdlFile::NamedSchemasFile` variant

## Reproduction

```sh
cat > tmp/bare-types.avdl <<'EOF'
namespace test.bare;

record Alpha { string name; }
enum Beta { X, Y, Z }
fixed Gamma(8);
EOF

cargo run -- idl tmp/bare-types.avdl
# Output: JSON array of 3 schemas (exit 0)

java -jar avro-tools-1.12.1.jar idl tmp/bare-types.avdl
# Error: the IDL file does not contain a schema nor a protocol. (exit 1)

# idl2schemata works in both:
cargo run -- idl2schemata tmp/bare-types.avdl tmp/out/
java -jar avro-tools-1.12.1.jar idl2schemata tmp/bare-types.avdl tmp/out/
# Both produce Alpha.avsc, Beta.avsc, Gamma.avsc
```

## Design considerations

This is a behavioral discrepancy rather than a correctness bug. The
Rust behavior is arguably more useful (it produces valid output
instead of an error), and matches the `IdlFile.outputString()` test
harness behavior. However, it deviates from the actual Java CLI tool.

Two reasonable approaches:

1. **Match Java CLI**: Error on bare named types in the `idl`
   subcommand with a message like "IDL file does not contain a
   schema nor a protocol." Keep `idl2schemata` working (as Java does).

2. **Keep current behavior**: Document this as an intentional
   enhancement. The JSON array output is valid and useful for tools
   that consume schema definitions.

## Priority

Low -- this only affects the edge case of `.avdl` files with bare
named type declarations and no `schema`/`protocol` keyword. These
files work correctly with `idl2schemata`, and the `idl` output is
valid JSON. The discrepancy is unlikely to cause real-world problems.
