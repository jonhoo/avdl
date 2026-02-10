# Schema mode files with only imports are rejected too early

## Symptom

An IDL file in schema mode that contains only `namespace` and `import`
statements (no `schema` declaration, no `protocol`, no bare named type
declarations) is rejected with "IDL file contains neither a protocol
nor a schema declaration". This prevents `idl2schemata` from extracting
the imported named schemas.

Java's `idl2schemata` accepts such files and correctly extracts the
imported schemas. Java's `idl` tool also rejects them, but that
rejection happens at the CLI level (after parsing completes), not at
parse time.

## Root cause

In `walk_idl_file` (reader.rs, around line 1100-1113), the function
checks whether there are local `namedSchemaDeclaration` nodes OR a
`mainSchemaDeclaration`. If neither exists, it returns an error. This
check happens in the parser, before imports are resolved, so it does
not account for named schemas that would be provided by imports.

The Java architecture separates parsing from output: `IdlReader`
always produces an `IdlFile` with a `ParseContext` containing all
registered schemas (including from imports). The "no schema nor
protocol" check is only in `IdlTool.run()` (the `idl` CLI), not in
`IdlToSchemataTool` (the `idl2schemata` CLI).

## Affected files

- `src/reader.rs` — `walk_idl_file` function
- `src/compiler.rs` — `parse_and_resolve` and `extract_impl` which
  call `walk_idl_file` and propagate the error

## Reproduction

```sh
# Create a minimal import-only schema-mode file:
cat > tmp/import-only.avdl << 'EOF'
namespace org.example;
import schema "tmp/imported.avsc";
EOF

cat > tmp/imported.avsc << 'EOF'
{"type":"record","name":"Foo","namespace":"org.example","fields":[{"name":"x","type":"string"}]}
EOF

# Rust rejects (both idl and idl2schemata):
cargo run -- idl2schemata tmp/import-only.avdl tmp/out/
# Error: IDL file contains neither a protocol nor a schema declaration

# Java idl2schemata accepts and extracts Foo.avsc:
java -jar avro-tools-1.12.1.jar idl2schemata tmp/import-only.avdl tmp/out-java/
ls tmp/out-java/  # Foo.avsc
```

## Suggested fix

Change `walk_idl_file` to return an `IdlFile` variant that indicates
"no main schema, no protocol, but there may be imports" instead of
returning an error. One approach:

1. When there are no local named schemas AND no main schema AND no
   protocol, but there ARE import statements in the children, return
   `Ok(IdlFile::NamedSchemas(vec![]))` instead of `Err(...)`. The
   empty vec signals "no local schemas" but allows import resolution
   to proceed.

2. Move the "no schema nor protocol" check from the parser to the
   `Idl::convert_impl` method (the `idl` CLI path). The
   `Idl2Schemata::extract_impl` path should NOT perform this check,
   matching Java's behavior where `IdlToSchemataTool` extracts all
   named schemas regardless.

Option 2 is cleaner and matches the Java architecture more closely.
The parser should not enforce a CLI-level policy.
