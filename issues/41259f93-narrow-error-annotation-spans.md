# Narrow error annotation spans in miette-rendered diagnostics

## Symptom

After commit 8cab18b switched error snapshots from `Debug` to
miette-rendered output, several diagnostics show source annotations
pointing at a single character (or too-narrow span) rather than the
full relevant syntax.

The most obvious case is annotation-related errors (reserved
property, type reference annotated, invalid alias, etc.), where the
span points only at the `@` character instead of the entire
annotation `@name(value)`:

```
 4 |                 record R { string @doc("field doc") name; }
   :                                   |
   :                                   `-- Can't set reserved property: doc
```

Should be:

```
 4 |                 record R { string @doc("field doc") name; }
   :                                   ^^^^^^^^^^^^^^^^
   :                                   `-- Can't set reserved property: doc
```

## Affected snapshots

All 9 annotation-related reader.rs tests and 1 integration test:

- `alias_with_leading_digit_is_rejected` — `@` instead of `@aliases(["123bad"])`
- `annotation_on_type_reference_is_rejected` — `@` instead of `@foo("bar")`
- `default_annotation_on_enum_is_rejected` — `@` instead of `@default("A")`
- `doc_annotation_on_field_variable_is_rejected` — `@` instead of `@doc("field doc")`
- `doc_annotation_on_message_is_rejected` — `@` instead of `@doc("message doc")`
- `doc_annotation_on_protocol_is_rejected` — `@` instead of `@doc("Protocol doc via annotation")`
- `doc_annotation_on_record_is_rejected` — `@` instead of `@doc("Record doc")`
- `response_annotation_on_message_is_rejected` — `@` instead of `@response("custom")`
- `type_annotation_on_field_type_is_rejected` — `@` instead of `@type("custom")`
- `integration__annotation_on_type_reference_file` — same `@`-only span

Additionally worth checking (lower priority):

- `nested_union_rejected` — spans `union` keyword but not the full
  nested `union { string, int }` expression

## Root cause

`make_diagnostic` computes the span from the ANTLR context's start
and stop tokens. For annotation errors, the diagnostic is constructed
from the `schemaProperty` context (the `&**prop` argument). The
grammar rule is:

```antlr
schemaProperty: At name=identifier LParen value=jsonValue RParen;
```

The stop token should be `RParen`, giving a span covering the whole
`@name(value)`. This likely means that `make_diagnostic` is only
using the start token (or start..start) rather than start..stop, or
that the ANTLR context's stop token isn't being populated correctly
for this rule.

## Affected files

- `src/reader.rs` — where `make_diagnostic` is called for these errors
- `src/error.rs` — `ParseDiagnostic` / `make_diagnostic` implementation

## Suggested fix

1. Verify that `make_diagnostic` uses both start and stop tokens to
   compute the source span (offset + length).
2. If the ANTLR context stop token is unreliable for `schemaProperty`,
   manually extend the span to cover through the closing `)`.
3. Update the affected snapshots after fixing.
