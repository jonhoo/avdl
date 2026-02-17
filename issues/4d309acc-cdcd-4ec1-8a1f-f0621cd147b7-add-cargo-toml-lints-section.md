# Add `[lints]` section to `Cargo.toml` for centralized lint configuration

## Symptom

The project has no `[lints]`, `[lints.rust]`, or `[lints.clippy]` section in
`Cargo.toml`. The CI `clippy` step runs with default lint levels only. No
`clippy.toml` or `.clippy.toml` configuration file exists. The only lint
configuration is a single `#![allow(non_upper_case_globals)]` in `src/reader.rs`
(for generated token constant names) and the `#[allow(clippy::all, unused)]`
wrappers on the generated parser modules.

This means that several useful lints that are off by default are not enforced,
and the lint policy is invisible to contributors.

## Root cause

The project was started without a `[lints]` section and has not yet adopted
one. Rust 1.74+ supports `[lints]` in `Cargo.toml` as the recommended way to
configure workspace-wide lint levels.

## Affected files

- `Cargo.toml`

## Reproduction

Run `cargo clippy` -- it passes cleanly, but so does code that violates
`clippy::must_use_candidate`, `clippy::return_self_not_must_use`, and other
useful pedantic lints.

## Suggested fix

Add a `[lints]` section to `Cargo.toml` with an opinionated baseline. A
reasonable starting point:

```toml
[lints.rust]
unsafe_code = "forbid"
unused_must_use = "deny"

[lints.clippy]
all = "warn"
# Selectively enable useful pedantic lints rather than enabling the full group,
# which fires extensively on generated code. The generated modules already have
# `#[allow(clippy::all)]` so these won't affect them.
must_use_candidate = "warn"
return_self_not_must_use = "warn"
doc_markdown = "warn"
missing_errors_doc = "warn"
missing_panics_doc = "warn"
```

The `unsafe_code = "forbid"` lint is effectively free since the project already
contains zero `unsafe` blocks. Making it explicit prevents accidental
introduction.

The `unused_must_use = "deny"` catches silently discarded `Result` values,
which is especially important in a compiler where ignoring errors leads to
silent data loss.

The selective pedantic lints improve API quality without creating noise on the
generated ANTLR code (which is already blanketed with `#[allow(clippy::all)]`).
