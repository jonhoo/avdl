# `validate_all_references` edge cases untested

- **Symptom**: Several branches in `validate_all_references` (lines
  912-1024 of `compiler.rs`) are at 0% coverage:
  1. **Spanless-only references** (lines 976-977): When all unresolved
     references lack source spans (e.g., from JSON `.avsc`/`.avpr`
     imports), the code falls back to a plain `miette::bail!` message.
     Never reached.
  2. **Multiple unresolved references** (lines 988-998): The `span_iter`
     loop that builds `related` diagnostics from the 2nd, 3rd, ...
     unresolved references. Never reached because all current tests
     trigger exactly one undefined name.
  3. **Mixed span/spanless related** (lines 1004-1013): Appending
     spanless references to the `related` list. Never reached.

- **Root cause**: All tests that trigger unresolved references define
  exactly one undefined type, and all references come from IDL source
  (which always carries spans). No test triggers multiple undefined
  names or references imported from JSON files (which have no spans).

- **Affected files**: `src/compiler.rs` (lines 962-1013)

- **Reproduction**: Run `cargo llvm-cov --text` and observe hit counts
  of 0 for lines 976-977, 988-998, 1004-1013.

- **Suggested fix**:
  1. Add a test with two or more undefined types in the same protocol
     to exercise the `related` diagnostics (lines 988-998).
  2. Add a test that imports a `.avsc` file containing an unresolved
     Reference (no span), then references it, to exercise the spanless
     path (lines 976-977) and mixed path (lines 1004-1013).
