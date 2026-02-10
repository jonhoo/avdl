# Only the first error is reported; subsequent errors are discarded

## Symptom

When an IDL file contains multiple independent errors, only the first
one is reported. The user must fix one error at a time, re-run, and
discover the next. This creates a frustrating fix-one-rerun cycle,
especially for agentic users that batch-process files.

Examples:

1. **Multiple missing semicolons**: Only the first missing semicolon is
   reported. The others are silently dropped.

2. **Multiple undefined type references**: `Missing1`, `Missing2`, and
   `Missing3` are all undefined, but only `Missing1` is reported.

3. **Multiple syntax errors**: ANTLR collects all syntax errors, but
   `parse_idl_named` only uses `collected_errors[0]`.

4. **Multiple invalid field defaults via compiler.rs**: The
   `validate_record_field_defaults` function returns a `Vec` of errors
   but only `errors.into_iter().next()` (the first one) is used.

## Root cause

Two distinct code paths each report only the first error:

1. **`parse_idl_named` in `reader.rs`** (line 866): Collects all ANTLR
   syntax errors into a `Vec`, but only extracts `collected_errors[0]`
   to build the `ParseDiagnostic`.

2. **`process_decl_items` in `compiler.rs`** (line 635): The
   `validate_record_field_defaults()` returns all errors but only the
   first is used.

3. **Tree-walk errors in `reader.rs`**: The recursive tree walk
   short-circuits on the first `Err(...)` via `?` propagation. This is
   harder to fix without significant architectural changes (would need
   error accumulation throughout the walk).

## Affected files

- `src/reader.rs`: `parse_idl_named()` around line 865-875
- `src/compiler.rs`: `process_decl_items()` around line 635

## Reproduction

```sh
# Multiple missing semicolons (only first reported)
cat > tmp/err-multiple-errors.avdl <<'EOF'
protocol Test {
  record Foo {
    string name
    int age
    boolean active
  }
}
EOF
cargo run -- idl tmp/err-multiple-errors.avdl 2>&1

# Multiple undefined types (only first reported)
cat > tmp/err-multiple-undefined.avdl <<'EOF'
protocol Test {
  record Foo {
    Missing1 a;
    Missing2 b;
    Missing3 c;
  }
}
EOF
cargo run -- idl tmp/err-multiple-undefined.avdl 2>&1
```

## Suggested fix

**Low-hanging fruit (ANTLR syntax errors):** In `parse_idl_named`,
build a multi-diagnostic report from all collected errors using
`miette::Report::new()` with `.related()` to attach additional errors.
miette supports related diagnostics that render below the primary:

```rust
let first = &collected_errors[0];
let primary = ParseDiagnostic { ... };
let related: Vec<ParseDiagnostic> = collected_errors[1..]
    .iter()
    .map(|e| ParseDiagnostic { ... })
    .collect();
// Attach related diagnostics to the primary
```

**Medium effort (compiler default validation):** Report all validation
errors from `validate_record_field_defaults` instead of only the first.

**Higher effort (tree-walk errors):** Would require changing the tree
walk to accumulate errors into a `Vec` rather than short-circuiting.
This is a larger refactor and may not be worth it for the marginal
benefit -- the fix-one-rerun cycle for semantic errors is more
tolerable than for syntax errors.
