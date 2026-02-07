# CLI panics on broken pipe (SIGPIPE)

## Symptom

When the tool's stdout is connected to a pipe that closes early (e.g.,
`avdl idl file.avdl | head -1`), the process panics with:

    thread 'main' panicked at library/std/src/io/stdio.rs:
    failed printing to stdout: Broken pipe (os error 32)

This produces a visible panic message on stderr instead of exiting
silently, which is the expected behavior for Unix CLI tools.

## Root cause

Rust programs by default install a SIGPIPE handler that ignores the
signal, causing `write` calls to return `ErrorKind::BrokenPipe` errors
instead of terminating the process. The `print!` macro in `write_output`
panics on write failure because it uses `write!` to stdout, which calls
`panic!` on error.

The standard fix for Rust CLI tools is to either:
1. Restore the default SIGPIPE behavior using
   `#[unix_sigpipe = "sig_dfl"]` on the main function (requires nightly
   or Rust 2024 edition), or
2. Catch `BrokenPipe` errors in `write_output` and exit with code 0, or
3. Use `writeln!(io::stdout(), ...)` instead of `print!` and handle the
   `BrokenPipe` error explicitly.

## Affected files

- `src/main.rs` -- `write_output` function (line 421, `print!` macro)

## Reproduction

```sh
echo '@namespace("test") protocol P { record R { string a; } }' \
  | cargo run -- idl 2>/dev/null | true
# Observe panic message on stderr (may be intermittent depending on
# timing; a larger output makes it more reproducible):
echo '@namespace("test") protocol P {
    record R1 { string a; string b; string c; }
    record R2 { string a; string b; string c; }
    record R3 { string a; string b; string c; }
    record R4 { string a; string b; string c; }
    record R5 { string a; string b; string c; }
}' | cargo run -- idl 2>/tmp/stderr.txt | true
grep "panicked" /tmp/stderr.txt
```

## Suggested fix

Since the project uses Rust edition 2024, the simplest fix is to add
`#[unix_sigpipe = "sig_dfl"]` to the `main` function. This restores the
default Unix behavior where SIGPIPE terminates the process silently:

```rust
#[unix_sigpipe = "sig_dfl"]
fn main() -> miette::Result<()> {
```

Alternatively, replace `print!("{content}")` in `write_output` with an
explicit write that catches `BrokenPipe`:

```rust
use std::io::Write;
if let Err(e) = write!(io::stdout(), "{content}") {
    if e.kind() == io::ErrorKind::BrokenPipe {
        return Ok(());
    }
    return Err(/* wrap error */);
}
```

## Priority

Medium. The panic produces confusing output for users piping avdl
output through other commands, and it violates Unix conventions for
CLI tool behavior.
