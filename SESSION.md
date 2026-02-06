Known issues are tracked in `issues/`.

- `ParseDiagnostic` fields (`src`, `span`, `message`) generate spurious
  "value assigned is never read" warnings because the thiserror/miette
  derive macros access them in ways rustc's analysis doesn't track.
- Numeric literal parse errors (`parse_integer_literal`,
  `parse_floating_point_literal`, `parse_integer_as_u32`) still use
  `IdlError::Other` because those functions receive only text strings
  and have no ANTLR context to extract source spans from.
- The `SchemaRegistry::register` error path in `walk_named_schema` also
  remains `IdlError::Other` since the error comes from the registry,
  not from a parse context.
- Issue #24 should be enriched with: (a) Java `TestIdlTool` and
  `TestIdlToSchemataTool` assert specific warning messages on stderr,
  including a warning for license-header doc comments -- our CLI tests
  (section 10) should verify stderr warning output too; (b) Java
  `TestIdlToSchemataTool.splitIdlIntoSchemata` asserts exactly 4 output
  files, a useful sanity check for our `idl2schemata` tests (section 3);
  (c) the `tools/src/test/idl/` directory contains `protocol.avdl` and
  `schema.avdl` which are additional test inputs not in `input/`.
- Discovery Agent A ran all 18 .avdl test files through `cargo run -- idl`:
  all 18 succeeded. 14/18 match golden output via `jq -S .` comparison.
  The 4 with diffs are import.avdl (#19, #21), nestedimport.avdl (#21),
  status_schema.avdl (#20), and interop.avdl (new float formatting issue,
  filed as `issues/39c7d498-float-scientific-notation-formatting.md`).
  Java tool comparison was blocked by sandbox restrictions.
- Discovery Agent C tested edge cases: extra files (protocolSyntax.avdl,
  schemaSyntax.avdl), classpath imports, error cases, and logical types.
  uuid.avdl, cycle.avdl, forward_ref.avdl, leading_underscore.avdl,
  union.avdl, unicode.avdl, namespaces.avdl, reservedwords.avdl, and
  schema_syntax_schema.avdl all match golden output via `jq -S .`.
  Two missing validations filed as issues caeb40b1 and 877f0e96. The golden
  `OnTheClasspath.avpr` file appears stale (contains `VeryFar` which
  does not appear in any source `.avdl` file); the Rust output for
  `OnTheClasspath.avdl` with `--import-dir` is `FromAfar` + `NestedType`
  which matches what the source files define. Java tool comparison was
  blocked by sandbox restrictions on `java -jar` commands.
