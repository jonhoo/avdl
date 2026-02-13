# Fixed with non-integer size produces confusing error

- **Symptom**: When `fixed` is declared with a non-integer size (e.g.,
  `fixed MD5(abc)`), the error message says "unexpected token `)`" with a
  large expected-token list:

  ```
  Error:   × line 2:15 unexpected token `)`
    help: expected one of: protocol, namespace, import, idl, schema, enum,
          fixed, error, record, array, map, union, boolean, int, long, ...
  ```

  This doesn't explain that `fixed` requires an integer size parameter.

- **Root cause**: The ANTLR grammar expects an integer literal after
  `fixed Name(`, but sees an identifier (`abc`). ANTLR treats this as a
  type reference attempt and continues parsing. When it hits `)`, it
  reports an unexpected token. The error enrichment logic doesn't recognize
  this as a "fixed size must be an integer" scenario.

- **Affected files**: `src/reader.rs` (error enrichment logic)

- **Reproduction**:
  ```sh
  cat > tmp/bad-fixed.avdl <<'EOF'
  protocol Test {
    fixed MD5(abc);
  }
  EOF
  cargo run -- idl tmp/bad-fixed.avdl
  ```

- **Suggested fix**: Pattern-match on errors involving `fixed Name(` where
  the next token is not an integer, and produce a clearer message:
  ```
  fixed type requires an integer size -- use `fixed MD5(16)`
  ```

  Alternatively, add a custom error action in the ANTLR grammar for the
  `fixedDeclaration` rule to catch this case and produce a semantic error.
