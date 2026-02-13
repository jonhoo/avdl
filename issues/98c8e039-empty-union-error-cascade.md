# Empty union produces cascade of 4+ confusing errors

- **Symptom**: An empty union (`union {}`) triggers ANTLR's error recovery
  and produces a cascade of 4 errors, none of which clearly state that
  unions cannot be empty:

  ```
  Error:   × line 3:11 unexpected token `}`
    help: expected one of: protocol, namespace, import, ...

  Error:
    × line 3:17 extraneous input ';' expecting {'}', ','}

  Error:
    × line 5:0 unexpected token `}`
    help: expected one of: ...

  Error:
    × line 6:0 unexpected end of file
    help: expected one of: ...
  ```

  The first error points at the `}` in `union {}` but doesn't explain that
  a union requires at least one type.

- **Root cause**: When ANTLR sees `union {` followed immediately by `}`, it
  tries to recover by treating `union` as a type reference and continues
  parsing. This cascades into downstream errors. ANTLR's error messages
  don't carry semantic knowledge about Avro's union rules.

- **Affected files**: `src/reader.rs` (error enrichment or ANTLR grammar)

- **Reproduction**:
  ```sh
  cat > tmp/empty-union.avdl <<'EOF'
  protocol Test {
    record User {
      union {} name;
    }
  }
  EOF
  cargo run -- idl tmp/empty-union.avdl
  ```

- **Suggested fix**: Either:
  1. Pattern-match on the error sequence (or first error location) to detect
     `union {}` and produce a single clear message: "union must contain at
     least one type member".
  2. Modify the ANTLR grammar to require at least one type in the union rule,
     which would produce a more direct error message from the parser.

  Option 2 is cleaner but requires regenerating the ANTLR parser.
