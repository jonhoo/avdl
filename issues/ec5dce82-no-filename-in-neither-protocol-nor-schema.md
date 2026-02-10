# "Neither protocol nor schema" error lacks filename and source location

## Symptom

When an IDL file contains named types but no `protocol` or `schema`
keyword (schema-mode with bare declarations), the `idl` subcommand
produces a generic error with no filename or source location:

```
Error:   x IDL file contains neither a protocol nor a schema declaration
```

This is unhelpful when processing multiple files or when the user
doesn't immediately realize which file triggered the error. The same
error appears for empty or nearly-empty files.

The error also provides no guidance on what the user should do to fix
it. Adding "did you mean to use `protocol MyProto { ... }` or
`schema <type>;`?" would help.

## Root cause

`compiler.rs` line 205 uses `miette::bail!()` which produces a plain
text error without any `NamedSource` or `SourceSpan` attached. The
source text and filename are available in the `CompileContext` at that
point but are not threaded into the error.

## Affected files

- `src/compiler.rs`: `compile_idl()` method, around line 203-205

## Reproduction

```sh
# File with only namespace and named types, no protocol keyword
cat > tmp/err-schema-mode-no-schema.avdl <<'EOF'
namespace org.test;
record Foo { string name; }
EOF
cargo run -- idl tmp/err-schema-mode-no-schema.avdl 2>&1

# Empty file
echo "" > tmp/err-empty-file.avdl
cargo run -- idl tmp/err-empty-file.avdl 2>&1
```

Both produce the same generic error.

## Suggested fix

Replace the `miette::bail!()` with a `ParseDiagnostic` that includes:

1. The filename (`ctx.source_name`)
2. A source span pointing to the beginning of the file (or the first
   named type declaration)
3. A help message suggesting `protocol` or `schema` syntax

For empty files, even just including the filename in the error message
(e.g., `"tmp/err-empty-file.avdl: IDL file contains neither..."` or
using miette's `NamedSource`) would be a significant improvement for
agentic (programmatic) consumers that parse filenames from error
output.
