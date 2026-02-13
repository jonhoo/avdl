# Undefined type in imported .avsc file lacks source location

- **Symptom**: When an imported `.avsc` file references an undefined type,
  the error message lacks any source location information:

  ```
  Error: × Undefined name: UnknownType
  ```

  No file path, no line number, no source excerpt. The user has no way to
  know which file or import caused the problem.

- **Root cause**: JSON imports create `Reference` nodes without source spans
  (since they come from JSON, not IDL source). When `validate_all_references`
  reports unresolved references, references from JSON imports have no span
  and fall through to the spanless error path.

  The current code path (lines 976-977 in `compiler.rs`) produces a plain
  `miette::bail!` message with just the names. While this is technically
  correct, it provides no actionable information about where the undefined
  reference came from.

- **Affected files**: `src/compiler.rs` (`validate_all_references`),
  `src/import.rs` (JSON import parsing)

- **Reproduction**:
  ```sh
  cat > tmp/bad-avsc.avsc <<'EOF'
  {"type":"record","name":"Foo","fields":[{"name":"x","type":"UnknownType"}]}
  EOF
  cat > tmp/test.avdl <<'EOF'
  protocol Test {
    import schema "bad-avsc.avsc";
  }
  EOF
  cargo run -- idl tmp/test.avdl
  ```

- **Suggested fix**: When reporting undefined references from JSON imports,
  include the file path of the `.avsc`/`.avpr` file that defined the
  problematic schema. This could be done by:
  1. Tracking the source file path in `Reference` nodes created during JSON
     import, or
  2. When the spanless path is taken, generating a diagnostic that at least
     mentions the import statement span from the IDL file (since we know
     which import brought in the schema).
