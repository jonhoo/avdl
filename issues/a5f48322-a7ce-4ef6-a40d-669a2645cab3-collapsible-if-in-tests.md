# Collapse nested `if` blocks in error_reporting tests

## Symptom

`cargo clippy --all-targets -- -W clippy::all` reports `clippy::collapsible_if`
at two locations in `tests/error_reporting.rs`. The nested `if` blocks check
two conditions that can be combined with `&&`.

## Root cause

The diagnostic rendering helpers check `diag.source_code().is_some()` and
`handler.render_report(&mut buf, diag).is_ok()` as separate nested `if`
statements instead of a single combined condition.

## Affected files

- `tests/error_reporting.rs:50-54` (in `compile_error`)
- `tests/error_reporting.rs:76-80` (in `compile_file_error`)

## Reproduction

```sh
cargo clippy --all-targets -- -W clippy::collapsible_if
```

## Suggested fix

Collapse each pair into a single `if` with `&&`:

```rust
// Before:
if diag.source_code().is_some() {
    if handler.render_report(&mut buf, diag).is_ok() {
        return Some(buf);
    }
}

// After:
if diag.source_code().is_some()
    && handler.render_report(&mut buf, diag).is_ok()
{
    return Some(buf);
}
```

This is a style-only change with no behavioral impact. The fix can be
auto-applied with `cargo clippy --fix --test error_reporting`.
