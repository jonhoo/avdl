# Import without kind specifier produces confusing merged error message

- **Symptom**: When an import statement omits the kind specifier (`idl`,
  `protocol`, or `schema`), the error message shows ANTLR's merged token
  representation:

  ```
  line 2:9 no viable alternative at input 'import"foo.avdl"'
  ```

  This `import"foo.avdl"` looks like a typo rather than a clear indication
  that the import kind is missing.

- **Root cause**: ANTLR merges consecutive tokens when reporting "no viable
  alternative" errors. The grammar expects `import` followed by one of
  `idl`/`protocol`/`schema`, but sees `StringLiteral` instead. ANTLR lumps
  `import` and the string together in its error message.

- **Affected files**: `src/reader.rs` (error enrichment logic in
  `enrich_antlr_error`)

- **Reproduction**:
  ```sh
  cat > tmp/bad-import.avdl <<'EOF'
  protocol Test {
    import "foo.avdl";
  }
  EOF
  cargo run -- idl tmp/bad-import.avdl
  ```

- **Suggested fix**: Add a pattern match in `enrich_antlr_error` for
  `no viable alternative at input 'import"...'` and produce a friendlier
  message like:
  ```
  import statement missing kind specifier -- use `import idl`, `import
  protocol`, or `import schema`
  ```
