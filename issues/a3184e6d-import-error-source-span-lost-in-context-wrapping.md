# Import error source span lost when wrapped as context

## Symptom

When a `.avsc` or `.avpr` import fails (e.g., invalid JSON), the error
message shows the imported file's JSON parse error but does NOT show
the source span of the `import` statement in the calling `.avdl` file.
The user sees:

```
Error:   x import schema /path/to/bad.avsc
  |-> invalid JSON in /path/to/bad.avsc: key must be a string at line 1 column 3
```

But does NOT see an underlined source span like:

```
   ,----[main.avdl:2:5]
 1 | protocol Test {
 2 |     import schema "bad.avsc";
   .     ------
   .        |-- import schema /path/to/bad.avsc
   `----
```

By contrast, when an import file is simply not found (resolve failure),
the source span IS shown correctly because the error is the root
`ParseDiagnostic`, not a context wrapper.

## Root cause

`wrap_import_error` in `compiler.rs` wraps the JSON parse error
(a `miette::Report`) with a `ParseDiagnostic` using
`error.context(diag)`. Miette's `GraphicalReportHandler` does not
render `source_code()` and `labels()` from context-layer diagnostics
-- only from the root diagnostic. Since the root here is the plain
`miette::miette!("invalid JSON in ...")` message (which has no source
code), the source span from the `ParseDiagnostic` context layer is
silently dropped.

## Affected files

- `src/compiler.rs` -- `wrap_import_error` and
  `resolve_single_import`

## Reproduction

```sh
echo '{ broken json' > tmp/bad.avsc
cat > tmp/test.avdl <<'EOF'
protocol Test {
    import schema "bad.avsc";
    record Foo { string name; }
}
EOF
cargo run -- idl tmp/test.avdl
```

The error output will not include the source span of line 2
(`import schema "bad.avsc";`) in the calling file.

## Suggested fix

Reverse the wrapping order so the `ParseDiagnostic` (which has
`source_code()` and `labels()`) is the **root** diagnostic, and the
downstream JSON error is attached as context:

```rust
let diag = ParseDiagnostic { src, span, message: ... };
miette::Report::new(diag).wrap_err(format!("{inner_error}"))
```

Or use `miette::Diagnostic::related()` to attach the inner error as
a related diagnostic, so both source spans are rendered.
