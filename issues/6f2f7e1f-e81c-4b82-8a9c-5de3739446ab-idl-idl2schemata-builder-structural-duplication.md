# `Idl` and `Idl2Schemata` builders are structurally duplicated

## Symptom

The `Idl` and `Idl2Schemata` structs in `compiler.rs` have identical
structure and nearly identical method implementations:

- Same fields: `import_dirs: Vec<PathBuf>`, `accumulated_warnings: Vec<miette::Report>`
- Same `new()` implementation (lines 111-115 vs 311-314)
- Same `import_dir()` implementation (lines 120-123 vs 318-321)
- Same `drain_warnings()` implementation (lines 135-137 vs 333-335)
- Same file-reading and path-resolution logic in `convert` / `extract`
  (lines 140-155 vs 340-359): `fs::read_to_string`, parent dir,
  `canonicalize`, etc.
- Same `convert_str` / `extract_str` delegation pattern
  (lines 159-161 vs 363-365)
- Same `convert_str_named` / `extract_str_named` pattern
  (lines 165-170 vs 369-378)
- Same `convert_impl` / `extract_impl` preamble: clear warnings,
  create `CompileContext`, call `parse_and_resolve`, handle error by
  draining warnings (lines 177-195 vs 428-446)

The two implementations diverge only after `parse_and_resolve`
succeeds: `convert_impl` serializes a protocol/schema to a single
JSON value, while `extract_impl` serializes each named schema
independently.

## Root cause

The two builders were developed as separate types to present distinct
public APIs (`IdlOutput` vs `SchemataOutput`). The shared
infrastructure was not factored out.

## Affected files

- `src/compiler.rs`: `Idl` (lines 59-267) and `Idl2Schemata`
  (lines 269-479)

## Reproduction

Read the two struct implementations side by side. The first ~70 lines
of each are virtually identical.

## Suggested fix

Extract a shared `IdlCompiler` (or `CompilerBase`) struct that owns
`import_dirs` and `accumulated_warnings`, and provides `new()`,
`import_dir()`, `drain_warnings()`, and a `compile()` method that
handles file reading, path resolution, and `parse_and_resolve`. Both
`Idl` and `Idl2Schemata` would wrap `IdlCompiler` and add only their
type-specific serialization logic.

Estimated savings: ~80 lines of duplicated builder boilerplate, plus
any future changes to the shared infrastructure only need to be made
once.
