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
