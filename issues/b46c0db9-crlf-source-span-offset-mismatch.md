# CRLF line endings cause snapshot test failure on Windows

## Symptom

The `test_annotation_on_type_reference_file` snapshot test fails on
Windows CI. The `SourceSpan` offset is 1117 on Windows vs 1089 on
Unix — a difference of 28, exactly the number of extra `\r` bytes
from the 28 lines preceding the error location.

```
-                1089,
+                1117,
```

## Root cause

The avro submodule's `.gitattributes` sets `* text=auto`, so `.avdl`
files are checked out with `\r\n` line endings on Windows. The test
reads the file with `fs::read_to_string`, which preserves the
platform line endings verbatim. The ANTLR lexer then assigns token
byte offsets that include the `\r` bytes, producing a different
`SourceSpan::offset` than on Unix.

The test uses `insta::assert_debug_snapshot!(err)`, which captures
the full `Debug` representation of the error — including the byte
offset. The offset is correct for the file as it exists on disk on
each platform, but it differs across platforms because the file has
different line endings.

## Affected files

- `tests/integration.rs` — `test_annotation_on_type_reference_file`
- `tests/snapshots/integration__annotation_on_type_reference_file.snap`

## Reproduction

Only reproduces on Windows, or when feeding a CRLF-encoded file to
the parser.

## Suggested fix

Normalize the input in the test before feeding it to `parse_idl`:

```rust
// Normalize CRLF → LF so that byte offsets in error spans are
// consistent across platforms. The avro submodule's .gitattributes
// causes .avdl files to be checked out with \r\n on Windows.
let input = fs::read_to_string(&avdl_path)
    .unwrap()
    .replace("\r\n", "\n");
```

This keeps the existing `assert_debug_snapshot!` and its byte-offset
coverage intact. The other error tests in `integration.rs` don't hit
this because they use inline string literals (which always have
`\n`), but this test reads from a file in the avro submodule whose
line endings vary by platform.

## Related issues

- `issues/8aa7e8a4-doc-comment-crlf-normalization.md` — CRLF
  handling in doc comment extraction (different symptom, same root
  cause family)
