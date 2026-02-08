# avdl

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
cargo install --path .
```

Or just build and run locally:

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
  line-breaking style may differ. The output parses to the same
  logical structure.

- **Import search paths replace Java classpath.** avro-tools resolves
  `import` paths via the JVM classpath; this tool uses explicit
  `--import-dir` flags instead, which serve the same purpose without
  requiring a JVM.

- **Schema-mode leniency.** The `idl` subcommand accepts `.avdl` files
  containing bare named type declarations (no `protocol` or `schema`
  keyword), returning them as a JSON array. Java's `IdlTool` CLI
  rejects such files, though Java's internal `IdlFile.outputString()`
  test harness supports the same output.

- **Trailing commas in enums.** `enum E { A, B, C, }` is silently
  accepted via ANTLR error recovery. avro-tools 1.12.1 crashes on this
  input.

- **Better error diagnostics.** The Rust tool gives clear error
  messages in cases where avro-tools 1.12.1 crashes with unchecked
  exceptions: duplicate message parameter names
  (`NoSuchElementException` in avro-tools), and reserved type names via
  backtick escapes like `` record `int` {} `` (`NullPointerException`
  in avro-tools).

- **Namespace validation covers all segments.** Rust validates every
  dot-separated segment of namespace names. Java's
  `IdlReader.namespace()` loop skips the last segment, so a namespace
  like `org.valid.0bad` passes Java's validation but is correctly
  rejected by the Rust tool.

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

The initial version of this crate was built during a [live stream on
YouTube](https://youtu.be/vmKvw73V394). The live stream ends at
commit [`fac28dd`](https://github.com/jonhoo/avdl/commit/fac28dd).

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
