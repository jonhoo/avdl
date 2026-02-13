# "Unexpected end of file" for unclosed brace lacks actionable context

- **Symptom**: When a record or protocol is missing a closing brace, the
  error message says "unexpected end of file" but doesn't indicate what
  construct is unclosed:

  ```
  Error:   × parse IDL source
    ╰─▶ line 6:0 unexpected end of file
     ╭─[tmp/unclosed-brace.avdl:6:1]
   5 │ }
     ╰────
    help: expected one of: protocol, namespace, import, idl, schema, enum,
          fixed, error, record, array, map, union, ...
  ```

  The help text lists ~20+ expected tokens, which is overwhelming and
  doesn't clarify that the real problem is a missing `}`.

- **Root cause**: When ANTLR reaches EOF while still inside a nested rule
  (e.g., inside a record definition), it reports that it expected tokens
  valid at that position. The error enrichment logic simplifies large
  expected-token sets but doesn't recognize the "unclosed construct" pattern.

- **Affected files**: `src/reader.rs` (error enrichment logic)

- **Reproduction**:
  ```sh
  cat > tmp/unclosed.avdl <<'EOF'
  protocol Test {
    record User {
      string name;

  }
  EOF
  cargo run -- idl tmp/unclosed.avdl
  ```

- **Suggested fix**: When the offending token is `<EOF>` and the expected
  set includes `}`, produce a more specific message like:
  ```
  unexpected end of file -- missing closing `}`
  ```

  More advanced: track the opening brace's location and suggest where the
  unclosed block started. This would require parser state tracking beyond
  what ANTLR's error listener provides, but even the simpler message above
  would significantly improve usability.
