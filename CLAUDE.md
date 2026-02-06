# avdl — Avro IDL Compiler in Rust

A Rust implementation of Apache Avro's IDL (`.avdl`) compiler, porting
the Java `avro-tools idl` and `avro-tools idl2schemata` subcommands.
Parses `.avdl` files using an ANTLR4-generated parser and emits Avro
Protocol JSON (`.avpr`) or Schema JSON (`.avsc`).

## Specification references

- [Avro specification](https://avro.apache.org/docs/1.12.0/specification/)
  — full schema, protocol, and serialization format.
- [Avro IDL language](https://avro.apache.org/docs/1.12.0/idl-language/)
  — the `.avdl` surface syntax this tool parses.

## Build and test

```sh
cargo build
cargo test           # unit tests + integration tests
cargo insta test     # if snapshot tests are added later
```

The integration tests parse `.avdl` files from the Avro test suite and
compare the serialized JSON output against golden `.avpr`/`.avsc` files.
Known issues are tracked in `issues/`.

When comparing our JSON output against the Java tool's golden files,
note that key ordering now matches the Avro-specified order (matching
Java's output). The remaining formatting difference is whitespace:
Java uses spaces before colons and a different array/object
line-breaking style, so compare semantically (parse both as JSON and
compare values) rather than as raw strings.

### Comparing against the Java tool

```sh
INPUT_DIR=avro/lang/java/idl/src/test/idl/input
OUTPUT_DIR=avro/lang/java/idl/src/test/idl/output
CLASSPATH_DIR=avro/lang/java/idl/src/test/idl/putOnClassPath

# Rust:
cargo run -- idl --import-dir $INPUT_DIR --import-dir $CLASSPATH_DIR \
  $INPUT_DIR/foo.avdl tmp/foo.avpr
# Java:
java -jar ../avro-tools-1.12.1.jar idl $INPUT_DIR/foo.avdl tmp/foo-java.avpr
# Compare (semantic):
diff <(jq -S . tmp/foo.avpr) <(jq -S . $OUTPUT_DIR/foo.avpr)
```

For ad-hoc debugging, create a temporary Rust example in `examples/`
and run it with `cargo run --example <name>`. Remove the example
after use.

Use `tmp/` (project-local) for intermediate files and comparison
artifacts, not `/tmp`. This keeps outputs discoverable and
project-scoped. The `tmp/` directory is gitignored.

## CLI usage

```sh
# Compile .avdl to protocol (.avpr) or schema (.avsc) JSON:
cargo run -- idl input.avdl output.avpr
cargo run -- idl input.avdl               # stdout
cat input.avdl | cargo run -- idl         # stdin → stdout

# Extract individual .avsc files from a protocol:
cargo run -- idl2schemata input.avdl outdir/

# Add import search directories (replaces Java classpath):
cargo run -- idl --import-dir ./extra/ input.avdl
```

## Project layout

```
src/
  main.rs               CLI (clap): `idl` and `idl2schemata` subcommands
  lib.rs                 Module declarations
  reader.rs              Core ANTLR tree walker — the heart of the parser
  model/
    mod.rs               Re-exports schema, protocol, json modules
    schema.rs            AvroSchema enum, Field, LogicalType, FieldOrder, PrimitiveType
    protocol.rs          Protocol and Message structs
    json.rs              JSON serialization matching Java avro-tools output format
  doc_comments.rs        Extracts doc comments from the ANTLR token stream
  import.rs              Import resolution for .avdl, .avpr, .avsc files
  resolve.rs             SchemaRegistry: named type tracking and forward references
  error.rs               miette-based error types (ParseDiagnostic, IdlError)
  generated/             ANTLR-generated parser/lexer (do not edit by hand)
    mod.rs               #![allow(...)] wrappers for generated code
    idlparser.rs
    idllexer.rs
    idllistener.rs
    idlbaselistener.rs

tests/
  integration.rs         Parses all test .avdl files, compares against golden JSON

issues/                  Known bugs and improvements, one per file
```

## Key reference files in submodules

### Grammar

- `avro/share/idl_grammar/org/apache/avro/idl/Idl.g4` — the
  authoritative ANTLR grammar for Avro IDL. All literal parsing,
  keyword sets, and syntax rules should match what this grammar
  defines. When in doubt about what syntax is legal, consult this
  file rather than Java stdlib behaviour.

### Java reference implementation

These are the files we are porting from. Consult them when behaviour
is unclear or when the Avro specification is ambiguous about IDL
semantics.

- `avro/lang/java/idl/src/main/java/org/apache/avro/idl/IdlReader.java`
  — the primary source to port. Uses an ANTLR listener to walk the
  parse tree and build Avro Schema/Protocol objects (~1,072 lines).
- `avro/lang/java/tools/src/main/java/org/apache/avro/tool/IdlTool.java`
  — the `avro-tools idl` subcommand entry point. Simple wrapper that
  calls `IdlReader`, handles stdin/stdout, and writes JSON.
- `avro/lang/java/tools/src/main/java/org/apache/avro/tool/IdlToSchemataTool.java`
  — the `avro-tools idl2schemata` subcommand. Iterates named schemas
  and writes individual `.avsc` files.

### Test suite (golden files)

- `avro/lang/java/idl/src/test/idl/input/` — 18 `.avdl` test input
  files, plus `.avsc`/`.avpr` files used as import targets
  (`baz.avsc`, `foo.avsc`, `bar.avpr`, `player.avsc`, `position.avsc`).
- `avro/lang/java/idl/src/test/idl/output/` — expected `.avpr`/`.avsc`
  output for each test case.
- `avro/lang/java/idl/src/test/idl/putOnClassPath/` — files that Java
  resolves via classpath. In our tool, pass this directory via
  `--import-dir` instead. Contains `OnTheClasspath.avdl/avpr/avsc`
  and a `folder/` subdirectory with relative imports.
- `avro/lang/java/idl/src/test/idl/extra/` — additional test inputs
  (`protocolSyntax.avdl`, `schemaSyntax.avdl`).
- `avro/lang/java/tools/src/test/idl/` — additional golden-file
  pairs (`protocol.avdl`/`.avpr`, `schema.avdl`/`.avsc`) for the
  `idl` and `idl2schemata` CLI entry points.
- `avro/lang/java/tools/src/test/java/org/apache/avro/tool/` — Java
  test classes (`TestIdlTool.java`, `TestIdlToSchemataTool.java`)
  that exercise CLI behavior.

You can also validate against the Java tool directly:

```sh
java -jar avro-tools-1.12.1.jar idl input.avdl output.avpr
```

### antlr4rust runtime

- `antlr4rust/runtime/Rust/` — the Rust runtime for ANTLR4. Useful
  for understanding `CommonTokenStream`, `InputStream`, token access
  patterns, and the generated context types.
- `antlr4rust/runtime/Rust/tests/general_tests.rs` — usage examples
  for the ANTLR Rust runtime.
- `antlr4rust/runtime/Rust/src/common_token_stream.rs` — the
  `CommonTokenStream` API. Notably, `get(index)` is public and
  provides raw access to the token buffer including hidden-channel
  tokens, which we use for doc comment extraction (since
  `getHiddenTokensToLeft` is unimplemented in antlr4rust).

## Architecture decisions

### Recursive tree walk instead of ANTLR listener

The Java version implements `IdlBaseListener` with `enter`/`exit`
methods and maintains mutable stacks. In Rust, implementing the
`IdlListener` trait is awkward because listener methods receive
borrowed contexts that can't coexist with mutable state on
`&mut self`. Instead, we parse with `build_parse_tree = true` and
walk the tree with recursive functions (`walk_protocol`,
`walk_record`, `walk_full_type`, etc.) that return values. This is
simpler and more idiomatic Rust.

### Custom domain model instead of `apache-avro` crate

The `apache-avro` Rust crate lacks a `Protocol` type and its
`Schema` serialization does not match the expected output format of
the Java tools. We use a purpose-built domain model (`AvroSchema`
enum, `Protocol` struct) that serializes to `serde_json::Value` with
full control over JSON key ordering and formatting.

### Named type serialization

In Avro protocol JSON, named types (record, enum, fixed) appear
inline (full definition) on first occurrence, then as bare string
names in subsequent references. The `schema_to_json` function tracks
`known_names: &mut IndexSet<String>` to decide which form to use.
`Reference` nodes are resolved against a `SchemaLookup` table to
enable inlining at first use.

### Import search paths replace Java classpath

Java resolves `import` paths first relative to the current file, then
via the JVM classpath. We replace the classpath concept with explicit
`--import-dir` flags. The `ImportContext` struct handles path
resolution and cycle prevention via a `HashSet<PathBuf>` of already-
visited files.

### Doc comment extraction via raw token access

The Java code calls `tokenStream.getHiddenTokensToLeft()`, which is
unimplemented in antlr4rust. Instead, after parsing, we scan
backwards from a node's start token index via
`CommonTokenStream::get(index)`, looking for `DocComment` tokens
(type 2), skipping `WS` (type 6) and `EmptyComment` (type 3).
See `src/doc_comments.rs`.

## Tricky areas

These are areas where the implementation is non-obvious or where
bugs are likely to hide.

### Nullable type reordering

The `type?` syntax creates `union { null, T }`. But if the field's
default value is non-null, the union must be reordered to `[T, null]`
because Avro requires the first type in a union to match the default.
See `walk_nullable_type` in `reader.rs`.

### Namespace inheritance

Types without an explicit `@namespace` annotation inherit the
enclosing protocol's namespace. This affects the fully-qualified name
used for `SchemaRegistry` lookup keys and `SchemaLookup` keys during
JSON serialization. Getting this wrong causes reference resolution
failures.

### Schema mode vs protocol mode

Avro IDL files can define either a protocol (`protocol Foo { ... }`)
or a standalone schema (`schema int;` or bare named type
declarations). The two modes have different serialization paths and
different namespace/reference-resolution behaviour. Schema mode is
less thoroughly tested.

### Properties on primitives

Primitives with annotations (e.g., `@foo("bar") int`) are wrapped in
the `AnnotatedPrimitive` variant, which serializes as
`{"type": "int", "foo": "bar"}` instead of a bare `"int"` string.

## Issue tracking

Issues live in `issues/`, one file per issue. Filename format:
`<uuid>-short-description.md` (use `$(uuidgen)`). Some older issues
use numeric prefixes instead. Check existing issues before filing to
avoid duplicates.

Each issue file should include:
- **Symptom**: what's wrong or missing
- **Root cause**: why it happens (if known)
- **Affected files**: which source files are involved
- **Reproduction**: commands or test case to reproduce
- **Suggested fix**: approach sketch
