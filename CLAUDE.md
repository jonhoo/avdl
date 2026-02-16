# avdl — Avro IDL Compiler in Rust

A Rust implementation of Apache Avro's IDL (`.avdl`) compiler, porting
the Java `avro-tools idl` and `avro-tools idl2schemata` subcommands.
Parses `.avdl` files using an ANTLR4-generated parser and emits Avro
Protocol JSON (`.avpr`) or Schema JSON (`.avsc`).

## Specification references

- [Avro specification](https://avro.apache.org/docs/1.12.0/specification/)
  — full schema, protocol, and serialization format.
  Local source: `avro/doc/content/en/docs/1.12.0/Specification/_index.md`
- [Avro IDL language](https://avro.apache.org/docs/1.12.0/idl-language/)
  — the `.avdl` surface syntax this tool parses.
  Local source: `avro/doc/content/en/docs/1.12.0/IDL Language/_index.md`

These are the authoritative Hugo source files for the published docs
at avro.apache.org. Prefer reading the local copies over fetching the
website — they are always available and match the submodule version.

## Build and test

```sh
cargo build
cargo test                              # unit tests + integration tests
cargo insta test --unreferenced delete  # also cleans up orphaned .snap files
```

The integration tests parse `.avdl` files from the Avro test suite and
compare the serialized JSON output against golden `.avpr`/`.avsc` files.
Known issues are tracked in `issues/`.

**Non-goal: byte-identical output.** Producing output that is
byte-for-byte identical to the Java tool is explicitly a non-goal.
Whitespace differences (spaces before colons, array/object
line-breaking style, indentation) are expected and acceptable. The
goal is **semantic correctness**: the JSON output should parse to the
same logical structure as the Java tool's output. Always compare
semantically (parse both as JSON and compare values) rather than as
raw strings.

### Comparing against avro-tools

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

The canonical correctness check is `cargo test`, which runs semantic
JSON comparison against golden files for both `idl` (18 `.avpr` files)
and `idl2schemata` (61 `.avsc` files across 17 test inputs).

For ad-hoc debugging, create a temporary Rust example in `examples/`
and run it with `cargo run --example <name>`. Remove the example
after use.

Use `tmp/` (project-local) for intermediate files and comparison
artifacts, not `/tmp`. This keeps outputs discoverable and
project-scoped. The `tmp/` directory is gitignored.

### Helper scripts

`scripts/compare-adhoc.sh` compares Rust output against Java
avro-tools output for arbitrary `.avdl` files written to `tmp/`.
Use this for edge-case exploration beyond the golden test suite.

```sh
scripts/compare-adhoc.sh tmp/edge-case.avdl          # single file
scripts/compare-adhoc.sh tmp/edge-*.avdl              # glob multiple files
scripts/compare-adhoc.sh --idl2schemata tmp/test.avdl # idl2schemata mode
scripts/compare-adhoc.sh --show-output tmp/test.avdl  # print JSON on diff
scripts/compare-adhoc.sh --rust-only tmp/test.avdl    # skip Java comparison
scripts/compare-adhoc.sh --import-dir path/ tmp/t.avdl # with import dirs
```

### Ad-hoc testing with the CLI

When testing the CLI with ad-hoc `.avdl` input, **write the input to a
temp file in `tmp/`** and pass it by path, rather than piping via
`echo | cargo run` or `cat <<EOF | cargo run`. This avoids interactive
permission prompts for pipe commands in sub-agents.

```sh
# Good: write to temp file, pass by path
cat > tmp/test-$(uuidgen).avdl <<'EOF'
protocol Test { record Foo { string name; } }
EOF
cargo run -- idl tmp/test-*.avdl

# Avoid: piping requires interactive permission
echo 'protocol Test { ... }' | cargo run -- idl
```

### Sandbox pipe workaround

The Claude Code sandbox has a [known issue][cc-16305] where data is
silently dropped in shell pipes between commands. Appending a trailing
`;` to the command fixes this:

```sh
# Broken in sandbox (downstream receives no input):
diff <(jq -S . a.json) <(jq -S . b.json)

# Fixed — append `;`:
diff <(jq -S . a.json) <(jq -S . b.json);
echo "abc" | grep "abc";
```

This affects pipes (`|`), process substitution (`<(...)`), and any
command that connects stdout of one process to stdin of another.
Direct command invocations without piping (e.g., `cargo run`,
`java -jar`) work fine under sandbox.

[cc-16305]: https://github.com/anthropics/claude-code/issues/16305

### Sandbox `!` (negation) workaround

The Claude Code sandbox has a [separate bug][cc-24136] where the bash
`!` keyword (pipeline negation operator) is treated as a literal
command name instead of a shell reserved word. The command after `!`
**never executes** — no side effects occur, and stderr shows
`!: command not found`. This affects `if !`, `while !`, and bare `!`.

The trailing-semicolon workaround for the pipe bug above does **not**
fix this.

```sh
# Broken in sandbox (touch never runs):
if ! some_command; then handle_failure; fi

# Workaround — capture $? instead:
some_command; rc=$?
if [ "$rc" -ne 0 ]; then handle_failure; fi

# Broken in sandbox (touch never runs):
while ! some_command; do sleep 1; done

# Workaround — use `until` (semantic equivalent):
until some_command; do sleep 1; done
```

[cc-24136]: https://github.com/anthropics/claude-code/issues/24136

### Sandbox `gh` (GitHub CLI) workaround

The `gh` CLI requires access to auth tokens stored under `~/.config/gh/`
(or `$GH_CONFIG_DIR`). The sandbox blocks this access, so `gh` commands
(e.g., `gh pr view`, `gh api`) fail with authentication errors. Use
`dangerouslyDisableSandbox: true` for `gh` invocations.

### Sandbox discipline

Never use `dangerouslyDisableSandbox` preemptively. Always attempt
commands in the default sandbox first — both `cargo` and `java -jar`
work fine. Only bypass the sandbox after observing an actual
permission error, and document which error triggered the bypass.
The one standing exception is `gh` (see above).

### Regenerating the ANTLR parser

The generated parser/lexer in `src/generated/` is checked in so that
building the project only requires Rust. Regeneration is only needed
when the grammar (`Idl.g4`) changes.

```sh
scripts/regenerate-antlr.sh
```

**Prerequisites** (only for regeneration, not for normal builds):
- Java (tested with 21)

The script automatically downloads the ANTLR tool JAR from the
[antlr4rust fork](https://github.com/antlr4rust/antlr4) GitHub
release and caches it at `tmp/antlr4-tool.jar`. This fork adds
Rust target support — the upstream ANTLR4 project does not support
Rust.

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
  main.rs               CLI (lexopt): `idl` and `idl2schemata` subcommands
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
  error.rs               ParseDiagnostic: source-span error for rich miette rendering
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
  and a `folder/` subdirectory with relative imports. **Note:** the
  three `OnTheClasspath.*` files are intentionally different fixtures,
  not variants of the same file. Each defines a different type name
  (`FromAfar` in `.avdl`, `VeryFar` in `.avpr`, `FarAway` in `.avsc`)
  to test the three import mechanisms (`import idl`, `import protocol`,
  `import schema`). They are NOT expected to be in sync.
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

The antlr4rust runtime source is available at
<https://github.com/antlr4rust/antlr4/tree/master/runtime/Rust>.
Useful for understanding `CommonTokenStream`, `InputStream`, token
access patterns, and the generated context types. Notably,
`CommonTokenStream::get(index)` is public and provides raw access to
the token buffer including hidden-channel tokens, which we use for
doc comment extraction (since `getHiddenTokensToLeft` is
unimplemented in antlr4rust).

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

### Test paths and Windows backslashes

`PathBuf::join` inserts `\` on Windows, and `canonicalize()` adds
the `\\?\` prefix. When a test path flows through `path.display()`
into a miette diagnostic source name (the `╭─[path:line:col]`
header), these differences cause snapshot mismatches against the
Linux-generated `.snap` files.

Tests whose snapshots contain file paths are annotated with
`#[cfg_attr(windows, ignore)]` so they are skipped on Windows CI.
These tests still run on Linux CI and locally on Linux/macOS.

### Error test conventions

Tests that assert on error output use `insta::assert_snapshot!` with
`render_diagnostic(&err)` (from `crate::error` for unit tests, or
`common::render_diagnostic` for integration tests). This captures the
full rendered diagnostic — source context, labels, help text — and
catches regressions that a `.contains()` substring check would miss.

Do **not** use `format!("{err}")` + `.contains(...)` for new error
tests. If a test's error contains a tempdir path, normalize it before
snapshotting:

```rust
let rendered = crate::error::render_diagnostic(&err);
let stable = rendered
    .replace(&dir.path().display().to_string(), "<tmpdir>");
insta::assert_snapshot!(stable);
```

### Properties on primitives

Primitives with annotations (e.g., `@foo("bar") int`) are wrapped in
the `AnnotatedPrimitive` variant, which serializes as
`{"type": "int", "foo": "bar"}` instead of a bare `"int"` string.

## Important branches

The `last-audit` branch marks the last commit reviewed by the
workflow prompt that audits changes. Do not delete this branch when
pruning worktree branches.

## Issue tracking

Issues live in `issues/`, one file per issue. Filename format:
`<uuid>-short-description.md` (use `$(uuidgen)`). Some older issues
use numeric prefixes instead. Check existing issues before filing to
avoid duplicates.

Issues caused by bugs in the upstream Apache Avro project (e.g.,
stale golden files, Java tool crashes) live in `upstream-issues/`
instead. These are tracked separately since they cannot be fixed in
this project.

Each issue file should include:
- **Symptom**: what's wrong or missing
- **Root cause**: why it happens (if known)
- **Affected files**: which source files are involved
- **Reproduction**: commands or test case to reproduce
- **Suggested fix**: approach sketch

## Changelog

This project keeps a changelog following
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). When making
user-facing changes, add an entry to the `[Unreleased]` section of
`CHANGELOG.md` under the appropriate category (Added, Changed,
Deprecated, Removed, Fixed, Security). The `[Unreleased]` section
always retains headings for all six categories, even when empty. Omit
empty categories only in released version sections.

Every entry must earn its place: the changelog is for users of the
library and binary, not for contributors. Internal changes (test
infrastructure, CI, issue tracking, code formatting, dev-only
scripts) do not belong. When in doubt, ask: "would a user deciding
whether to upgrade care about this?"

Write entries in imperative mood ("Validate ...", "Reject ..."), not
declarative ("Validates ...", "now rejects ..."). Describe the
user-visible effect, not the implementation technique — e.g., "Render
all errors with colored, annotated source excerpts" rather than "Use
miette diagnostic rendering for all error output".

Categorize by what the change *is*, not what code it touches: new
capabilities (including new validations and warnings) go in **Added**,
changes to existing behavior in **Changed**, and actual bug
corrections in **Fixed**.

Entries need not map one-to-one to commits: several commits may be
unified under one entry when they contribute to a single user-visible
change. Each entry references a primary commit by its short hash at
the end of the line (GitHub auto-links these).
