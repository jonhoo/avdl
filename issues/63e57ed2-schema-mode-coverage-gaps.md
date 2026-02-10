# Schema-mode test coverage is minimal

## Symptom

The Avro IDL spec supports a schema mode where files define standalone
schemas instead of protocols. The spec examples include:

- `schema int;` (bare primitive)
- `schema Message;` (named type reference)
- `schema array<Message>;` (complex type as main schema)
- Bare named type declarations without `schema` keyword

The test coverage for schema mode is limited to:

1. **`schema_syntax_schema.avdl`** -- tests `schema array<StatusUpdate>;`
   with named types. Compared against the golden `.avsc` file.
2. **`schemaSyntax.avdl`** -- tests `schema array<Message>;` with a
   simple structural check (not golden-file comparison).
3. **`status_schema.avdl`** -- tests bare named types (no `schema`
   keyword), only checking that `Idl` rejects it and `Idl2Schemata`
   accepts it.
4. **`convert_str_schema_mode`** in `compiler.rs` -- tests `schema int;`
   produces `"int"`.
5. **`tools/src/test/idl/schema.avdl`** -- tests
   `schema TestRecord;` with forward-referenced types.

Missing coverage includes:

- **`schema` with primitives**: `schema boolean;`, `schema string;`,
  `schema long;`, `schema bytes;`, etc. Only `schema int;` is tested
  in a compiler unit test, not via integration tests.
- **`schema` with logical types**: `schema date;`, `schema uuid;`,
  `schema decimal(10, 2);`
- **`schema` with `map<T>`**: `schema map<string>;`
- **`schema` with `union { ... }`**: `schema union { null, string };`
- **`schema` with nullable**: `schema string?;`
- **Namespace interactions in schema mode**: the `namespace` directive
  (without `@`) affects named types differently than protocol mode.
- **Multiple named types in schema mode**: testing that all named types
  are registered and correctly cross-referenced.

## Root cause

The upstream Java test suite focuses on protocol mode. Schema mode was
added later and has fewer golden test files. The Rust port mirrors this
gap.

## Affected files

- `tests/integration.rs` (no schema-mode tests for primitives, logical
  types, maps, or unions)
- `src/reader.rs` (the schema-mode path in `walk_idl_file`)

## Reproduction

No test currently exercises these. Ad-hoc examples:

```avdl
schema boolean;
```

```avdl
namespace org.example;
schema date;
```

```avdl
namespace org.example;
schema map<string>;
```

## Suggested fix

Add unit tests in `reader.rs` or integration tests for a broader set
of schema-mode inputs:
- Primitives: `schema int;`, `schema string;`, `schema boolean;`
- Logical types: `schema date;`, `schema uuid;`
- Complex types: `schema map<string>;`, `schema union { null, int };`
- Nullable shorthand: `schema string?;`
- Namespace + named type combinations
