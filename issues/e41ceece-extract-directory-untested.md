# `Idl2Schemata::extract_directory` has zero test coverage

- **Symptom**: The directory-walking code path in `Idl2Schemata::extract()`
  (lines 478-505 of `compiler.rs`) is never exercised by any test. When
  `extract()` receives a directory path, it delegates to `extract_directory`,
  which uses `walkdir` to recursively find `.avdl` files and compile each
  independently. This entire branch is at 0% coverage.

- **Root cause**: No test passes a directory to `Idl2Schemata::extract()`.
  All existing tests either use `extract_str()` (inline source) or pass a
  single `.avdl` file path.

- **Affected files**: `src/compiler.rs` (lines 448-505)

- **Reproduction**: Run `cargo llvm-cov --text` and observe that lines
  452, 478-505 all show hit count 0.

- **Suggested fix**: Add a test that creates a temp directory with 2-3
  `.avdl` files, passes the directory to `Idl2Schemata::new().extract()`,
  and verifies that schemas from all files are returned in sorted
  filename order. Should also test the edge case of an empty directory
  (no `.avdl` files) returning an empty `SchemataOutput`.
