# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Suggest similar type names in "undefined name" errors using edit
  distance, including a note that Avro primitives are lowercase when
  the user writes e.g. `String` instead of `string` (3b31889)
- Detect missing import kind specifier (`import "file.avdl"`) and
  suggest `import idl`, `import protocol`, or `import schema` (450d85f)
- Include the imported file path in undefined type errors originating
  from `.avsc`/`.avpr` imports (2fc4544)
- Suggest `protocol` when misspelled at the top level, matching the
  existing did-you-mean behavior inside protocol bodies (3c7afeb)
- Detect `protocol`/`record`/`enum`/`fixed` followed by `{` and report
  "expected name" instead of cascading unhelpful errors (3c7afeb)
- Detect trailing commas in enum declarations and point at the comma
  with a "trailing comma is not allowed" message (3c7afeb)
- Detect bare `@` before declaration keywords and explain that the
  annotation syntax is `@name("value")` (3c7afeb)

### Changed

### Deprecated

### Removed

### Fixed

- Report "value must be a non-negative integer" for negative fixed
  sizes like `fixed Hash(-5)` instead of leaking Rust's internal
  `IntErrorKind::InvalidDigit` message (9af17af)
- Collapse cascading errors for `map<>` and `array<>` (empty type
  parameter) into a single message explaining that a type parameter
  is required (9af17af)
- Detect missing closing `}` when another `record`/`enum`/`error`/
  `fixed` declaration follows, and point at the unclosed construct
  instead of reporting a confusing "unexpected `{` expected `;`"
  (9af17af)
- Validate `@logicalType` annotations on `Fixed` schemas (e.g.,
  `duration` requires `fixed(12)`, `decimal` precision must fit the
  fixed byte size) (b687d72)
- Reject record default values that omit required fields; previously
  the compiler silently produced invalid Avro JSON (aec527c)
- Filter ANTLR-internal `'\u001A'` (SUB) and `DocComment` tokens from
  error messages, and display `<EOF>` as "end of file" (35cfa50)
- Display `identifier` instead of `IdentifierToken` in expected-token
  lists, and similarly humanize `StringLiteral`, `IntegerLiteral`, and
  `FloatingPointLiteral` to plain-language equivalents (d6605a9)
- Collapse cascading ANTLR errors for empty unions, misspelled keywords,
  non-integer fixed sizes, and unclosed braces into single actionable
  messages (704ff7d)
- Rewrite ANTLR jargon in error messages: "extraneous input" and
  "mismatched input" become "unexpected", "no viable alternative"
  becomes "unexpected input", and token set notation `{';', ','}`
  becomes natural language "expected ';' or ','" (3c7afeb)
- Explain that `void` can only be used as a message return type instead
  of reporting a misleading "Undefined name: void" error (12cc7fd)
- Explain that `decimal` requires `(precision, scale)` parameters
  instead of reporting a misleading "Undefined name: decimal" error
  (12cc7fd)

### Security

## [0.1.5] - 2026-02-11

### Added

- Add `--version` / `-V` flag to print the version (305c592)
- Report all syntax errors, undefined type references, and invalid field
  defaults at once instead of stopping at the first error (8685b71)

### Fixed

- Correct `local_timestamp_ms` spelling in reserved-type-name validation
  (was `localtimestamp_ms`, silently accepting the reserved name as a
  type) (305c592)
- Report unterminated string literals at the opening quote instead of
  at the next downstream token, which produced a misleading "unexpected
  token" error (01ea32a)
- Include enclosing record name and point source span at the offending
  field declaration in default-validation errors (125e2fb)
- Highlight the default value expression instead of the field name in
  invalid-default diagnostics (7e73fa8)
- Report undefined type errors in source order instead of alphabetical
  order (10e9962)
- Include filename and source location in "neither protocol nor schema"
  error, matching the diagnostic format used for other errors since
  0.1.4 (10e9962)
- Remove duplicate keywords from expected-token help text (305c592)

## [0.1.4] - 2026-02-10

### Added

- Validate `int` and `long` default values are within range (1aa6f24)
- Validate type references in protocol messages resolve to known types
  (aa8b2a8)
- Expose warnings as `Vec<miette::Report>` in public API (509dddd)
- Suggest quoting bare identifiers used in default value positions
  (ab34c2d)
- Reject bare named type declarations in `idl` subcommand (only valid
  in `idl2schemata`) (c4c7c94)
- Warn when annotations on non-nullable unions are silently dropped
  (0c069dc)

### Changed

- List all searched directories in import error diagnostics (7fa1f2f)
- Render all errors with colored, annotated source excerpts (8cab18b)
- Simplify expected-token lists in parser error messages (330b915)

### Fixed

- Accept files containing SUB character (U+001A) (51a945f)
- Accept import-only files in `idl2schemata` schema mode (b375efd)
- Clarify error when `idl2schemata` output path is a file not a
  directory (b9583eb)
- Emit warnings to stderr even when compilation fails (bf69ea8)
- Preserve source location in errors from imported JSON files
  (824c571)
- Widen error spans to cover full parse rule context (808715a)

## [0.1.3] - 2026-02-09

### Added

- Public library API: `Idl` and `Idl2Schemata` builder types for
  programmatic compilation without shelling out to the CLI (c9ffbc6)
- Parse C-style comments (`//`, `/* */`) in imported `.avsc`/`.avpr`
  files, matching Java's `ALLOW_COMMENTS` behavior (5c21581)
- Source-span highlighting in undefined-type errors (d4882ee)
- Source-span highlighting in out-of-place doc comment warnings
  (d2663de)
- Enriched annotation syntax errors (e.g., "`@beta` is missing its
  value") (d18e00d)

### Changed

- Make `main.rs` a thin wrapper around the public library API
  (c9ffbc6)
- Narrowed visibility of internal types to `pub(crate)`; public
  surface is `Idl`, `Idl2Schemata`, `IdlOutput`, `SchemataOutput`,
  and `NamedSchema` (c9ffbc6)
- Use standard float formatting in JSON output instead of Java-style
  scientific notation for edge-case values (937ce33)

### Fixed

- Surface lexer errors as structured warnings instead of raw stderr
  (7848f90)
- Validate decimal precision/scale per Avro spec (d18be09)
- Reject invalid `null?` type (produces duplicate-null unions)
  (9655a25)
- Validate default values for `Reference`-typed fields (d1b6691)
- Reject annotations on messages with named return types (ambiguous)
  (da29c52)
- Omit empty namespace from JSON output (`@namespace("")`) (da29c52)
- Strip tabs after `*` in doc comments (da29c52)

## [0.1.2] - 2026-02-08

### Changed

- Shortened package description for Homebrew compatibility (89171de)

## [0.1.1] - 2026-02-08

### Added

- Source spans in syntax errors, registry errors, and import errors
  for precise error location (07d8d21)

### Changed

- Reduced binary size and compile time by replacing `clap`,
  `thiserror`, and `miette-derive` with lighter alternatives (26021d7)
- Drop `git clone --recursive` requirement for building from source
  (git submodule replaced with automatic JAR download) (24596d4)
- Trim published crate size via `include` whitelist in `Cargo.toml`
  (af48a3d)

### Fixed

- Display actual filename and highlight correct field name in error
  diagnostics (c744086)
- Preserve source location through nested error wrapping (69eb914)

## [0.1.0] - 2026-02-08

First published release, covering changes since the end of the
[live stream](https://github.com/jonhoo/avdl/compare/end-of-2026-02-06-stream...v0.1.0+1.12.1)
that created the initial crate. Rust port of Apache Avro's
`avro-tools idl` and `idl2schemata` subcommands.

### Added

- `idl` subcommand: compile `.avdl` to Avro Protocol JSON (`.avpr`)
  (f31c58c)
- `idl2schemata` subcommand: extract individual `.avsc` files from
  protocols (f31c58c)
- Import support (`import idl`, `import protocol`, `import schema`)
  with `--import-dir` search paths and cycle detection (af3db53)
- Forward references to types defined later in the file (2e78024)
- Nullable type (`type?`) with automatic union reordering for defaults
  (af3db53)
- Logical type support (date, timestamp-millis, decimal, uuid, etc.)
  (af3db53)
- Doc comment extraction and preservation (2e78024)
- Schema mode for bare type declarations (no protocol wrapper)
  (af3db53)
- Rich error diagnostics with source context via `miette` (2ff13b2)
- Comprehensive validation: duplicate fields/symbols, namespace
  naming, reserved type names, nested unions, default value types,
  one-way message return types, fixed-size bounds, decimal
  precision/scale (3ffc409)

[Unreleased]: https://github.com/jonhoo/avdl/compare/v0.1.5+1.12.1...HEAD
[0.1.5]: https://github.com/jonhoo/avdl/compare/v0.1.4+1.12.1...v0.1.5+1.12.1
[0.1.4]: https://github.com/jonhoo/avdl/compare/v0.1.3+1.12.1...v0.1.4+1.12.1
[0.1.3]: https://github.com/jonhoo/avdl/compare/v0.1.2+1.12.1...v0.1.3+1.12.1
[0.1.2]: https://github.com/jonhoo/avdl/compare/v0.1.1+1.12.1...v0.1.2+1.12.1
[0.1.1]: https://github.com/jonhoo/avdl/compare/v0.1.0+1.12.1...v0.1.1+1.12.1
[0.1.0]: https://github.com/jonhoo/avdl/compare/end-of-2026-02-06-stream...v0.1.0+1.12.1
