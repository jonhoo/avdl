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
cat input.avdl | avdl idl

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

## Intentional divergences from Java

This tool aims for semantic correctness against the Java `avro-tools`
output, but deliberately differs in a few ways:

- **JSON formatting is not byte-identical.** Whitespace, key ordering
  within objects, and array line-breaking style may differ. The output
  parses to the same logical structure.

<!-- TODO: populate with a full review of intentional divergences -->

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

## Origin

The initial version of this crate was [implemented live on
YouTube](https://youtu.be/vmKvw73V394).
The stream ends at commit
[`fac28dd`](https://github.com/apache/avro/commit/fac28dd2dcbdf6f6a12abd5a99a2f290fcf29ae6).
