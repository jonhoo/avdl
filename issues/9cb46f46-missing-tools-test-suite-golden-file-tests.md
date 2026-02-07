# Missing integration tests for `tools/src/test/idl/` golden files

## Symptom

The Java `TestIdlTool` and `TestIdlToSchemataTool` test suites
exercise two additional golden-file pairs that have no corresponding
Rust integration tests:

1. `tools/src/test/idl/schema.avdl` -> `schema.avsc`
   (schema mode with `schema TestRecord;` and forward references)
2. `tools/src/test/idl/protocol.avdl` -> `protocol.avpr`
   (simplified protocol, similar to `input/simple.avdl`)

The Rust tool produces correct output for both (verified manually),
but there are no regression tests.

## Why these matter

The `schema.avdl` file exercises a pattern not covered by any
existing Rust test: **schema mode with a named type as the main
schema** (`schema TestRecord;`) where the main type references
forward-declared types (`Kind`, `MD5`). The existing schema-mode
tests use different patterns:

- `schema_syntax_schema.avdl`: `schema array<StatusUpdate>;`
  (main schema is an anonymous array type)
- `status_schema.avdl`: Bare named types without `schema` keyword
  (NamedSchemasFile mode)
- `extra/schemaSyntax.avdl`: `schema array<Message>;`
  (similar to schema_syntax_schema)

None of these test `schema <NamedType>;` where the named type itself
contains forward references to types defined later in the file. This
pattern exercises the reference resolution pipeline in schema mode
more thoroughly.

The `protocol.avdl` is less critical since it overlaps significantly
with `input/simple.avdl`, but it does exercise the `@aliases(["hash"])`
annotation on a non-null-default nullable field (`MD5?`), which
produces `["null", "MD5"]` ordering rather than the `["MD5", "null"]`
from the explicit `union { MD5, null}` in `simple.avdl`.

## Affected files

- `tests/integration.rs` -- no tests referencing `tools/src/test/idl/`

## Reproduction

```sh
# Both produce output matching the golden files:
cargo run -- idl avro/lang/java/tools/src/test/idl/schema.avdl \
    tmp/tools-schema.avsc
diff <(jq -S . tmp/tools-schema.avsc) \
     <(jq -S . avro/lang/java/tools/src/test/idl/schema.avsc)
# No diff

cargo run -- idl avro/lang/java/tools/src/test/idl/protocol.avdl \
    tmp/tools-protocol.avpr
diff <(jq -S . tmp/tools-protocol.avpr) \
     <(jq -S . avro/lang/java/tools/src/test/idl/protocol.avpr)
# No diff
```

## Suggested fix

Add two integration tests:

```rust
const TOOLS_IDL_DIR: &str = "avro/lang/java/tools/src/test/idl";

#[test]
fn test_tools_schema() {
    let avdl_path = PathBuf::from(TOOLS_IDL_DIR).join("schema.avdl");
    let actual = parse_and_serialize(&avdl_path, &[]);
    let expected = load_expected(
        &PathBuf::from(TOOLS_IDL_DIR).join("schema.avsc"),
    );
    assert_eq!(actual, expected);
}

#[test]
fn test_tools_protocol() {
    let avdl_path = PathBuf::from(TOOLS_IDL_DIR).join("protocol.avdl");
    let actual = parse_and_serialize(&avdl_path, &[]);
    let expected = load_expected(
        &PathBuf::from(TOOLS_IDL_DIR).join("protocol.avpr"),
    );
    assert_eq!(actual, expected);
}
```

## Priority

Low. The output is already correct; this is purely about regression
coverage for a schema-mode pattern not exercised by existing tests.
