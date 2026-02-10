# Test suite cannot run from `.crate` file

## Symptom

`cargo package` produces a `.crate` that does not include the test
suite. Running `cargo test` against the unpacked crate fails because
test sources, snapshot files, test data, and avro submodule fixtures
are all excluded.

## Root cause

The `include` field in `Cargo.toml` is too narrow:

```toml
include = ["/src/**/*.rs", "/LICENSE", "/README.md"]
```

This excludes everything the test suite needs beyond the library source.

## Missing files

### Test source files

- `tests/integration.rs`
- `tests/cli.rs`
- `tests/error_reporting.rs`

### Insta snapshot files

- `tests/snapshots/*.snap` — integration and error reporting snapshots
- `src/snapshots/*.snap` — unit test snapshots (`.snap` does not match
  the current `/src/**/*.rs` pattern)

### Project-local test data

- `tests/testdata/**` — cycle detection `.avdl` fixtures, regression
  test files, and `idl2schemata-golden/` directories with `.avsc`
  golden files

### Avro submodule test fixtures

The `avro/` git submodule contains the upstream Avro test suite that
the integration tests compare against. Only specific subdirectories
are needed — not the entire submodule. The required paths are:

- `avro/lang/java/idl/src/test/idl/input/` — `.avdl` test inputs
  plus `.avsc`/`.avpr` files used as import targets
- `avro/lang/java/idl/src/test/idl/output/` — golden `.avpr`/`.avsc`
  expected output
- `avro/lang/java/idl/src/test/idl/putOnClassPath/` — classpath
  import test files (including `folder/` subdirectory)
- `avro/lang/java/idl/src/test/idl/extra/` — `protocolSyntax.avdl`
  and `schemaSyntax.avdl`
- `avro/lang/java/idl/src/test/idl/cycle.avdl`
- `avro/lang/java/idl/src/test/idl/logicalTypes.avdl`
- `avro/lang/java/idl/src/test/idl/AnnotationOnTypeReference.avdl`
- `avro/lang/java/tools/src/test/idl/` — `protocol.avdl`,
  `protocol.avpr`, `schema.avdl`, `schema.avsc`
- `avro/lang/java/compiler/src/test/idl/` — `work space/root.avdl`
  and `work space/root.avpr` (workspace path test)

## Suggested fix

Expand the `include` array in `Cargo.toml` to cover all of the above.
Cargo supports git submodule contents in `include` patterns. Use
targeted globs to avoid pulling in the entire `avro/` tree.

Ref: <https://doc.rust-lang.org/cargo/reference/manifest.html#the-exclude-and-include-fields>

Sketch:

```toml
include = [
    "/src/**",
    "/tests/**",
    "/LICENSE",
    "/README.md",
    # Avro submodule test fixtures
    "/avro/lang/java/idl/src/test/idl/input/**",
    "/avro/lang/java/idl/src/test/idl/output/**",
    "/avro/lang/java/idl/src/test/idl/putOnClassPath/**",
    "/avro/lang/java/idl/src/test/idl/extra/**",
    "/avro/lang/java/idl/src/test/idl/cycle.avdl",
    "/avro/lang/java/idl/src/test/idl/logicalTypes.avdl",
    "/avro/lang/java/idl/src/test/idl/AnnotationOnTypeReference.avdl",
    "/avro/lang/java/tools/src/test/idl/**",
    "/avro/lang/java/compiler/src/test/idl/**",
]
```

Note `/src/**` instead of `/src/**/*.rs` so that `src/snapshots/*.snap`
is included.

## Reproduction

```sh
cargo package --list
# Observe that tests/, src/snapshots/, and avro/ paths are absent.
```

## Affected files

- `Cargo.toml` (the `include` field)
