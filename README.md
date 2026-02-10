# avdl

[![codecov](https://codecov.io/gh/jonhoo/avdl/graph/badge.svg?token=WP4ZHFDYVO)](https://codecov.io/gh/jonhoo/avdl)

A Rust port of Apache Avro's `avro-tools idl` and `idl2schemata`
commands. Compiles [Avro IDL](https://avro.apache.org/docs/1.12.0/idl-language/)
(`.avdl`) files to Avro Protocol JSON (`.avpr`) or Schema JSON (`.avsc`).

## Usage

```sh
# .avdl → protocol JSON
avdl idl input.avdl output.avpr

# .avdl → individual schema files
avdl idl2schemata input.avdl outdir/

# stdin/stdout
avdl idl < input.avdl

# additional import search paths
avdl idl --import-dir ./extra/ input.avdl
```

## Install

```sh
cargo install avdl
```

Or build and run from a local checkout:

```sh
cargo build
cargo run -- idl input.avdl
```

## Testing

```sh
cargo test
```

The integration tests parse the same `.avdl` input files shipped in
the [Apache Avro](https://github.com/apache/avro/) test suite and
compare output semantically against the expected `.avpr`/`.avsc`
reference files. All test cases pass, alongside unit tests for
serialization, error reporting, and edge cases. The tool is functional
but young — bug reports are welcome.

## Intentional divergences from Java

This tool aims for semantic correctness against the Java `avro-tools`
output, but deliberately differs in a few ways:

- **JSON formatting is not byte-identical.** Whitespace and array
  line-breaking style may differ. JSON object keys are sorted
  alphabetically, whereas Java avro-tools preserves insertion order.
  The output parses to the same logical structure.

- **Import search paths replace Java classpath.** avro-tools resolves
  `import` paths via the JVM classpath; this tool uses explicit
  `--import-dir` flags instead, which serve the same purpose without
  requiring a JVM.

- **Schema-mode leniency.** The `idl` subcommand accepts `.avdl` files
  containing bare named type declarations (no `protocol` or `schema`
  keyword), returning them as a JSON array. Java's `IdlTool` CLI
  rejects such files, though Java's internal `IdlFile.outputString()`
  test harness supports the same output.

- **Better error diagnostics.** Errors include source context with the
  offending token underlined, powered by [miette](https://docs.rs/miette).
  The Rust tool also gives clear error messages in cases where
  avro-tools 1.12.1 crashes with unchecked exceptions (e.g.,
  `NoSuchElementException` for duplicate message parameters,
  `NullPointerException` for reserved type names via backtick escapes).

  For example, given a record with a duplicate field name:

  ```avro
  record User {
      string name;
      int name;
  }
  ```

  **avro-tools** produces a Java stack trace:

  ```
  Exception in thread "main" org.apache.avro.SchemaParseException:
    org.apache.avro.AvroRuntimeException: Field already used: name type:STRING pos:0
      at org.apache.avro.idl.IdlReader.parse(IdlReader.java:220)
      ... 5 more
  ```

  **avdl** points directly at the problem:

  ```
  Error:   × parse IDL source
    ├─▶ parse `dup-field.avdl`
    ╰─▶ duplicate field 'name' in record 'User'
     ╭─[dup-field.avdl:4:13]
   3 │         string name;
   4 │         int name;
     ·             ──┬─
     ·               ╰── duplicate field 'name' in record 'User'
   5 │     }
     ╰────
  ```

- **Nested unions are rejected per spec.** The Avro specification
  states "Unions may not immediately contain other unions." This tool
  enforces this rule, while Java avro-tools 1.12.1 incorrectly accepts
  nested unions and silently produces an empty union `[]` in the JSON
  output. If you have `.avdl` files that compiled with Java but fail
  here, the nested union is a spec violation that Java should have
  caught.

- **Float formatting uses serde_json defaults.** Java renders large
  or small float/double values in scientific notation (e.g.,
  `-1.0E12`); this tool uses `serde_json`'s decimal representation
  (e.g., `-1000000000000.0`). Both parse to the same numeric value.

- **Namespace validation covers all segments.** Rust validates every
  dot-separated segment of namespace names. Java's
  `IdlReader.namespace()` loop skips the last segment, so a namespace
  like `org.valid.0bad` passes Java's validation but is correctly
  rejected by the Rust tool.

- **Faster execution.** As a native binary, avdl avoids JVM startup
  overhead. On real-world `.avdl` files it completes in single-digit
  milliseconds — roughly 50× faster than `avro-tools idl` for
  typical inputs, narrowing to ~6× on unusually large (1 MB) files
  where JVM startup is a smaller fraction of the total.

## Attribution

This is a "powercoded" port of the Java Avro tools — meaning it was
built with LLM assistance, incremental human guidance and review, and
strong testing against the upstream test suite.

All credit for Avro itself, the original tooling, and the underlying
test suite belongs to the [Apache Avro](https://github.com/apache/avro/)
project.

Key upstream sources this port is based on:

- [IdlTool.java](https://github.com/apache/avro/blob/c499eefb48aa2db906c7bca14a047223806f36db/lang/java/tools/src/main/java/org/apache/avro/tool/IdlTool.java) — the Java `avro-tools idl` entry point
- [IdlReader.java](https://github.com/apache/avro/blob/c499eefb48aa2db906c7bca14a047223806f36db/lang/java/idl/src/main/java/org/apache/avro/idl/IdlReader.java) — the Java IDL transformer
- [Idl.g4](https://github.com/apache/avro/blob/c499eefb48aa2db906c7bca14a047223806f36db/share/idl_grammar/org/apache/avro/idl/Idl.g4) — the ANTLR grammar for Avro IDL

## Background

The initial version of this crate was built during a [live stream on YouTube](https://youtu.be/vmKvw73V394).
The live stream ends at commit [`fac28dd`](https://github.com/jonhoo/avdl/commit/fac28dd).
Post-stream development is documented in [post-stream-summary.md](post-stream-summary.md).

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
