# Add `#[must_use]` to `Idl::new()` and `Idl2Schemata::new()`

## Symptom

`Idl::new()` and `Idl2Schemata::new()` are the primary public constructors for
the library's two builder types. Neither is annotated with `#[must_use]`. A
caller could write `Idl::new();` (constructing and immediately dropping the
builder) without any compiler warning.

## Root cause

The `#[must_use]` attribute was never added to these constructors. Clippy's
`must_use_candidate` lint (which is off by default, in the `pedantic` group)
flags both.

## Affected files

- `src/compiler.rs` (lines 248 and 425, the two `pub fn new()` methods)

## Reproduction

```sh
cargo clippy -- -W clippy::must-use-candidate 2>&1 | grep must_use
```

Output:

```
warning: this method could have a `#[must_use]` attribute
   --> src/compiler.rs:248:12
warning: this method could have a `#[must_use]` attribute
   --> src/compiler.rs:425:12
```

## Suggested fix

Add `#[must_use]` to both constructors:

```rust
#[must_use]
pub fn new() -> Self { ... }
```

Per the [Rust API guidelines](https://rust-lang.github.io/api-guidelines/checklist.html),
constructors and builder methods that return a new value should be `#[must_use]`.
The `IdlOutput`, `SchemataOutput`, and `NamedSchema` structs are returned from
fallible methods (`Result<T>`), so `#[must_use]` on the `Result` already covers
them. Only the infallible `new()` constructors need the annotation.
