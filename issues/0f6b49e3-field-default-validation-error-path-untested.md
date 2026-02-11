# Field default validation error reporting path is untested

- **Symptom**: The entire error-reporting block for invalid field defaults
  in `process_decl_items` (lines 705-749 of `compiler.rs`) is at 0%
  coverage -- 44 uncovered lines. This includes constructing the primary
  `ParseDiagnostic`, building `related` diagnostics for multiple errors,
  and both the span-based and spanless fallback error paths.

- **Root cause**: `validate_record_field_defaults` never returns errors in
  any test. The function validates that field defaults match
  Reference-typed fields, but no test defines a record with a default
  value that is invalid for a reference type (e.g., an enum field with a
  default value that is not one of its symbols).

- **Affected files**: `src/compiler.rs` (lines 700-749)

- **Reproduction**: Run `cargo llvm-cov --text` and observe that
  `errors.is_empty()` at line 703 always evaluates to true (hit count
  220), so the `continue` at line 704 is always taken and lines 705-749
  are never reached.

- **Suggested fix**: Add tests with records whose fields have invalid
  defaults for Reference-typed fields:
  1. A single field with a bad default (exercises the primary diagnostic)
  2. Multiple fields with bad defaults (exercises the `related`
     diagnostics loop at lines 712-728)
  3. A field whose schema has no source span (exercises the spanless
     fallback at line 749)
