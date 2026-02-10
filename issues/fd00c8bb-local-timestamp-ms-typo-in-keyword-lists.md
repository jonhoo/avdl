# `local_timestamp_ms` misspelled as `localtimestamp_ms` in KEYWORDS and INVALID_TYPE_NAMES

## Symptom

The `KEYWORDS` and `INVALID_TYPE_NAMES` arrays in `reader.rs` contain
`"localtimestamp_ms"` (missing the underscore between `local` and
`timestamp`), but the ANTLR grammar token text is `local_timestamp_ms`.

This has two consequences:

1. **`INVALID_TYPE_NAMES` does not block `local_timestamp_ms` as a type
   name.** A user could write `` record `local_timestamp_ms` { ... } ``
   and the validator would accept it, whereas Java rejects it. The
   misspelled entry `localtimestamp_ms` would never match anything from
   the parser because the grammar lexer always produces
   `local_timestamp_ms` (with the underscore).

2. **`KEYWORDS` list in `split_trailing_keyword` cannot match
   `local_timestamp_ms` as a trailing keyword suffix.** When ANTLR
   merges a bare annotation `@foo` with the keyword
   `local_timestamp_ms` into `@foolocal_timestamp_ms`, the enrichment
   logic tries to split the trailing keyword. It looks for `KEYWORDS`
   entries at the end of the string, and `localtimestamp_ms` won't match
   the `local_timestamp_ms` suffix that actually appears.

## Root cause

Simple typo when transcribing the keyword list. The grammar defines
`LocalTimestamp: 'local_timestamp_ms';` (line 201 of `Idl.g4`), but
the Rust code uses `localtimestamp_ms` in two places.

## Affected files

- `src/reader.rs` lines 551 and 713

## Reproduction

```rust
// This should be rejected but will be accepted:
let idl = "protocol P { record `local_timestamp_ms` { int x; } }";
let result = parse_idl_for_test(idl);
assert!(result.is_err(), "local_timestamp_ms as type name should be rejected");
```

## Suggested fix

Replace `"localtimestamp_ms"` with `"local_timestamp_ms"` in both
`KEYWORDS` (line 551) and `INVALID_TYPE_NAMES` (line 713). Add a unit
test that verifies `` record `local_timestamp_ms` { ... } `` is
rejected.
