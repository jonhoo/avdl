# Change `#[allow(deprecated)]` to `#[expect(deprecated)]` in `tests/cli.rs`

## Symptom

`tests/cli.rs:25` uses `#[allow(deprecated)]` on the `avdl_cmd()` helper to
suppress the deprecation warning from `Command::cargo_bin()`. This uses `allow`
instead of `expect`, so if `cargo_bin()` is ever un-deprecated (or the call is
replaced), the suppression silently becomes dead code.

## Root cause

The annotation was added before the team adopted `#[expect]`.

## Affected files

- `tests/cli.rs` (line 25)

## Reproduction

```rust
// Current:
#[allow(deprecated)] // cargo_bin() warns about custom build-dir; acceptable here
fn avdl_cmd() -> Command {
```

## Suggested fix

Change to:

```rust
#[expect(deprecated, reason = "cargo_bin() warns about custom build-dir")]
fn avdl_cmd() -> Command {
```

This ensures the compiler warns via `unfulfilled_lint_expectations` if the
deprecation warning ever goes away.
