# `Idl2Schemata::drain_warnings` untested

- **Symptom**: `Idl2Schemata::drain_warnings()` (lines 441-443 of
  `compiler.rs`) has 0 hits. The analogous `Idl::drain_warnings()` has
  2 hits from CLI integration tests.

- **Root cause**: No test calls `drain_warnings()` on an `Idl2Schemata`
  builder after a failed `extract*` call. The method is the only way to
  retrieve warnings that were accumulated before a compilation error.

- **Affected files**: `src/compiler.rs` (lines 441-443)

- **Reproduction**: Run `cargo llvm-cov --text` and observe that line
  441 shows hit count 0.

- **Suggested fix**: Add a test that:
  1. Creates an `Idl2Schemata` builder
  2. Calls `extract_str()` with IDL source that produces warnings
     (e.g., orphaned doc comments) followed by an error (e.g., an
     undefined type reference)
  3. Verifies the call returns `Err`
  4. Calls `drain_warnings()` and asserts the warnings are non-empty
