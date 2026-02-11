# Import error paths partially untested

- **Symptom**: Several error-handling branches in import-related code
  are at 0% coverage:

  In `compiler.rs`:
  1. **Import resolution error without span** (line 784): The fallback
     path when an import statement has no source span. Always has a span
     in practice since imports come from IDL parsing.
  2. **`wrap_import_error` without span** (line 899): The fallback path
     in the error wrapper. Same reason -- imports always have spans.
  3. **IDL import read/parse failure** (lines 826-827, 831-832): When
     a resolved IDL import file can't be read or parsed. The
     `resolve_import` step catches missing files first, so read errors
     are rare (would require permission issues or a race condition).
  4. **Nested import resolution failure** (lines 861-862): The
     `.with_context` wrapper for recursive `process_decl_items`. Would
     require a nested import to fail after the outer file succeeds.

  In `import.rs`:
  5. **Canonicalize errors** (lines 78-82, 90-94): When a file exists
     but `canonicalize()` fails (filesystem error mid-resolution).
  6. **Protocol JSON parse errors** (lines 320-324, 333-337): When an
     `.avpr` file contains valid JSON but invalid protocol structure.
  7. **Invalid schema JSON** (line 404): The catch-all for JSON values
     that are not strings, arrays, or objects.
  8. **Unknown schema type** (line 456): A JSON object with an
     unrecognized `type` field value.

- **Root cause**: These are defensive error paths for scenarios that are
  hard to trigger through normal IDL compilation. The import resolution
  step validates file existence before the read step, and the test suite
  uses well-formed golden files.

- **Affected files**: `src/compiler.rs` (lines 784, 826-832, 861-862,
  899), `src/import.rs` (lines 78-94, 320-337, 404, 456)

- **Suggested fix**: For the `import.rs` paths, add unit tests that
  call `json_to_schema` / `import_protocol` directly with malformed
  JSON (invalid types, missing fields, etc.). For the `compiler.rs`
  spanless paths, consider whether they are reachable at all -- if not,
  they could be simplified or documented as unreachable.
