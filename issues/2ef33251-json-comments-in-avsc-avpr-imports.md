# JSON comments in .avsc/.avpr imports rejected

## Symptom

Importing a `.avsc` or `.avpr` file that contains C-style comments
(`/* ... */` or `// ...`) before or within the JSON content fails with
"invalid JSON ... expected value at line 1 column 1". Java handles
these files successfully.

## Root cause

`import_schema` and `import_protocol` in `src/import.rs` both use
`serde_json::from_str` which requires strict JSON (no comments).
Java's `Schema.parse()` and `Protocol.parse()` use Jackson with
`ALLOW_COMMENTS` enabled, so C-style and C++-style comments are
silently stripped before parsing.

This commonly occurs in real-world `.avsc` files that have license
headers (e.g., Apache license block comments before the JSON body).

## Affected files

- `src/import.rs`: `import_schema` (line 349) and `import_protocol`
  (line 296-297)

## Reproduction

Regression test files in `tests/testdata/regressions/`:
- `avsc-with-comment.avdl` -- minimal .avdl importing a commented .avsc
- `imports/commented-report.avsc` -- .avsc with block comment before JSON
- `avsc-with-comment.avpr` -- expected Java output (golden file)

```sh
# Rust (fails):
cargo run -- idl tests/testdata/regressions/avsc-with-comment.avdl

# Java (succeeds):
java -jar avro-tools-1.12.1.jar idl tests/testdata/regressions/avsc-with-comment.avdl
```

## Suggested fix

Strip C-style (`/* ... */`) and C++-style (`// ...\n`) comments from
the file content before passing it to `serde_json::from_str`. This
should be applied in both `import_schema` and `import_protocol`.

Alternatively, use a JSON5 or comment-tolerant JSON parser, but a
simple comment-stripping preprocessor is likely sufficient and avoids
adding a dependency.

## Source

Discovered via `cloudera/flume` (`flumeconfig.avdl` importing
`avroflumereport.avsc` which has an Apache license block comment).
