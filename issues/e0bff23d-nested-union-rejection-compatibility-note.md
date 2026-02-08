# Nested union rejection (compatibility note)

## Symptom

Rust rejects `.avdl` files containing nested unions such as
`union { union { int } }` with the error "Unions may not immediately
contain other unions". Java accepts these files and produces output
(albeit with a quirky empty union `[]` for the field).

This means `.avdl` files that happen to contain nested unions will
compile with Java's `avro-tools idl` but fail with our Rust
implementation.

## Root cause

Rust enforces the Avro specification literally. The spec states:

> Unions may not immediately contain other unions.

Java's IDL reader does not check for nested unions at the IDL parsing
stage. Its `UnionSchema` constructor is supposed to reject them, but
the IDL reader appears to handle the situation differently, resulting
in an empty union `[]` rather than an error.

## This is NOT a bug in Rust

Rust's behavior is **correct per the Avro specification**. Java is
technically violating the spec by accepting nested unions. However,
this is filed as a compatibility note because:

1. Real-world `.avdl` files may contain nested unions that Java
   silently accepts.
2. Users migrating from Java `avro-tools` to this Rust implementation
   may encounter unexpected errors.
3. The error message could be more helpful.

## Affected files

- `src/reader.rs` (union type walking logic)

## Reproduction

Regression test file:
`tests/testdata/regressions/nested-union-accepted-by-java.avdl`

```sh
# Rust (rejects):
cargo run -- idl tests/testdata/regressions/nested-union-accepted-by-java.avdl

# Java (accepts, produces empty union):
java -jar avro-tools-1.12.1.jar idl tests/testdata/regressions/nested-union-accepted-by-java.avdl
```

## Suggested fix

**Keep Rust's current behavior** (rejecting nested unions) since it
is spec-compliant. However, improve the error message to:

1. Cite the relevant part of the Avro specification.
2. Mention that Java `avro-tools` incorrectly accepts nested unions,
   so users migrating from Java know this is a known divergence.

Example improved error message:

> Unions may not immediately contain other unions (per the Avro
> specification). Note: Java avro-tools incorrectly accepts this
> syntax, producing an empty union.

## Spec reference

[Avro specification](https://avro.apache.org/docs/1.12.0/specification/):

> Unions may not immediately contain other unions.

## Upstream issue

The Java bug (silently producing empty unions instead of rejecting) is
documented in `upstream-issues/90644028-java-nested-union-produces-empty-array.jira.md`.

## Source

Discovered during fuzz testing of 229 real-world `.avdl` files.
