# Test boilerplate: `Field` and named-type construction

## Symptom

Unit tests in `model/json.rs` (52 occurrences) and `model/schema.rs`
(59 occurrences) repeat `aliases: vec![], properties: HashMap::new()`
when constructing `Field` and `AvroSchema::Record/Enum/Fixed` values.
This creates verbose, hard-to-read tests.

## Root cause

No test-only constructors exist for common model types with sensible
defaults.

## Affected files

- `src/model/json.rs` (unit tests)
- `src/model/schema.rs` (unit tests)

## Reproduction

Search for `aliases: vec![], properties: HashMap::new()` in unit tests.

## Suggested fix

Add `#[cfg(test)]` helper constructors:

- `Field::test("name", schema)` — sets `aliases: vec![]`,
  `properties: HashMap::new()`, `doc: None`, `default: None`,
  `order: None`
- Similar helpers for `AvroSchema::Record`, `AvroSchema::Enum`,
  `AvroSchema::Fixed` with sensible test defaults

This reduces boilerplate without affecting production code. A
`Default`-like approach could also work but the named constructors
are more explicit.
