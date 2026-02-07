# `idl2schemata` with no arguments hangs instead of showing usage

## Symptom

Running `avdl idl2schemata` with no arguments causes the tool to hang
waiting for input on stdin, matching the `idl` subcommand's behavior
of treating no-input as stdin mode.

Java's `avro-tools idl2schemata` with no arguments prints a usage
message and exits immediately with code 255 (i.e., -1 from the Java
`Tool.run()` return value).

## Root cause

The Rust `Command::Idl2schemata` variant in `main.rs` defines `input`
as `Option<String>`, and `run_idl2schemata` calls `read_input(&input)`
which treats `None` as stdin mode. This mirrors how `idl` works, but
Java treats the two subcommands differently.

Java's `IdlToSchemataTool.run()` (line 43) has an explicit check:

```java
if (args.isEmpty() || args.size() > (useJavaCC ? 3 : 2) || isRequestingHelp(args)) {
    err.println("Usage: idl2schemata [--useJavaCC] [idl [outdir]]");
    return -1;
}
```

The key difference is `args.isEmpty()` -- Java requires at least one
argument for `idl2schemata`, while `idl` allows zero arguments (stdin
mode).

## Reproduction

```sh
# Rust: hangs reading from stdin (Ctrl+C to exit)
cargo run -- idl2schemata

# Java: prints usage and exits immediately with code 255
java -jar ../avro-tools-1.12.1.jar idl2schemata
# => "Usage: idl2schemata [--useJavaCC] [idl [outdir]]"
# => exit 255
```

## Impact

Low severity. The `idl2schemata` command is typically used with an
explicit input file. However, the hanging behavior is confusing for
users who forget to specify an input file -- they see no prompt and
no error, just silence. Java's behavior of showing usage is more
user-friendly.

This also means the `idl` and `idl2schemata` subcommands have
inconsistent stdin handling: `idl` supports stdin (useful for piping),
but `idl2schemata` writing multiple `.avsc` files from stdin input
is less useful and more surprising.

## Suggested fix

Option A: Require at least one positional argument for `idl2schemata`.
With clap, this can be done by making `input` non-optional or adding
a custom validator.

Option B: Keep stdin support but add a warning or prompt when stdin
is a terminal (not a pipe), matching common Unix CLI conventions.

Option A better matches Java behavior. Option B is more flexible
but adds complexity.

## Affected files

- `src/main.rs` -- `Command::Idl2schemata` and `run_idl2schemata()`
