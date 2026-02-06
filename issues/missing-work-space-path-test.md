# Missing test for paths with spaces (AVRO-3706)

## Symptom

The Java `TestIdl` class (compiler module) includes a specific test
case for AVRO-3706: parsing an `.avdl` file from a directory whose
path contains spaces (`work space/root.avdl`). This exercises import
resolution when the file path contains spaces.

The Rust tool handles this correctly (manually verified), but there
is no integration test to prevent regression.

## Java behavior tested

In `TestIdl.loadTests()` (line 87-90):

```java
// AVRO-3706 : test folder with space in name.
File inputWorkSpace = new File(TEST_DIR, "work space");
File root = new File(inputWorkSpace, "root.avdl");
File rootResult = new File(inputWorkSpace, "root.avpr");
tests.add(new GenTest(root, rootResult));
```

The test files are:
- `work space/root.avdl` -- `protocol Root { import idl "level1.avdl"; }`
- `work space/level1.avdl` -- `protocol Level1 { import idl "level2.avdl"; }`
- `work space/level2.avdl` -- `protocol Level2 { }`
- `work space/root.avpr` -- `{"protocol":"Root","types":[],"messages":{}}`

This exercises:
- File path resolution with spaces
- Chained IDL imports within a space-containing directory
- Correct protocol output (types and messages merge from imports)

## Affected files

- `tests/integration.rs` -- no test case for `work space/` directory

## Reproduction

```sh
cargo run -- idl \
    "avro/lang/java/compiler/src/test/idl/work space/root.avdl" \
    tmp/workspace.avpr
# Works correctly, producing {"protocol":"Root","types":[],"messages":{}}
```

Note: the `work space/` directory exists in the compiler module's
test directory (`avro/lang/java/compiler/src/test/idl/work space/`),
not in the IDL module's test directory.

## Suggested fix

Add an integration test that parses `work space/root.avdl` through
the full import pipeline and compares against `root.avpr`. The path
should use the compiler module's test directory.

## Priority

Low. The feature works; this is only about adding regression
coverage for an edge case that was a real bug in the Java
implementation (AVRO-3706).
