# Warnings are downgraded to plain strings, losing source span info

## Symptom

The `Warning` struct in `reader.rs` carries rich diagnostic
information: a `miette::NamedSource` and `miette::SourceSpan` for
source-context rendering, plus a `miette::Diagnostic` trait impl.
However, this information is never used. By the time warnings reach
the caller, they have been converted to plain `String` values.

Specifically:
- `IdlOutput::warnings` is `Vec<String>` (compiler.rs line 77)
- `SchemataOutput::warnings` is `Vec<String>` (compiler.rs line 207)
- The conversion at compiler.rs line 174 calls `.to_string()` on each
  `Warning`, discarding the source/span fields
- `main.rs` prints `"Warning: {w}"` where `w` is a `String`

This means:
1. The `miette::Diagnostic` impl on `Warning` is dead code
2. Warnings cannot be rendered with source-context underlining
3. The `Debug` output of `Warning` (used in the
   `lexer_error_produces_warning` snapshot test) shows the raw
   struct internals (`NamedSource { source: "<redacted>" }`,
   `SourceSpan { offset: SourceOffset(0), length: 0 }`) which is
   not useful for diagnostics

If warnings were rendered through miette's `GraphicalReportHandler`
(the way errors are rendered in the `compile_error` test helper),
they would show the source text with the offending token underlined,
which would be much more helpful.

## Root cause

The public API types `IdlOutput` and `SchemataOutput` use
`Vec<String>` for warnings instead of `Vec<Warning>`. The `Warning`
type is `pub` in `reader.rs` but is not re-exported from the crate
root, and the compiler module converts warnings to strings before
returning them.

## Affected files

- `src/compiler.rs` — `IdlOutput::warnings` (line 77),
  `SchemataOutput::warnings` (line 207), conversion at lines 174 and
  374
- `src/reader.rs` — `Warning` struct and its `miette::Diagnostic`
  impl (lines 59-153)
- `src/main.rs` — warning printing (lines 210-212, 239-241)

## Reproduction

Compare the error and warning rendering:

```sh
# Errors get rich miette rendering with source underlining:
cargo run -- idl <(echo 'protocol P { record R { Foo x; } }')

# Warnings only get flat text:
cargo run -- idl avro/lang/java/tools/src/test/idl/protocol.avdl /dev/null
# Output: "Warning: Line 1, char 1: Ignoring out-of-place..."
# No source context, no underlining
```

## Suggested fix

1. Re-export `Warning` from the crate root (or `compiler` module) so
   it is part of the public API.
2. Change `IdlOutput::warnings` and `SchemataOutput::warnings` to
   `Vec<Warning>` instead of `Vec<String>`.
3. In `main.rs`, render each warning through miette's
   `GraphicalReportHandler` when source info is available, falling
   back to `Display` when it is not (e.g., for import-prefixed
   warnings where the source is cleared).

This would give warnings the same rich rendering that errors already
have, with the source text and an underlined span pointing to the
offending doc comment or lexer error.

Note: this is a breaking API change for library users who depend on
`IdlOutput::warnings` being `Vec<String>`. If backward compatibility
is needed, consider adding a separate `raw_warnings: Vec<Warning>`
field alongside the existing `warnings: Vec<String>`.

Additionally, the `labels()` method in the `Diagnostic` impl
hardcodes `"out-of-place doc comment"` as the label text for ALL
warnings, including lexer error warnings. When the Diagnostic impl
is actually used, the label should be derived from the warning type
or message rather than hardcoded.
