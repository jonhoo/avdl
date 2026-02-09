# Hardcoded absolute paths in test snapshots (and related path fragility)

GitHub issue: https://github.com/jonhoo/avdl/issues/5

## Symptom

The snapshot file `tests/snapshots/error_reporting__error_import_nonexistent_file.snap`
contains the absolute path `/home/jon/dev/stream/avdl/main`, which is
baked into the snapshot from the machine where `cargo insta test
--accept` was last run. The test fails on any other machine, checkout
location, or git worktree because the `current_dir()` embedded in the
error message will differ.

## Root cause

When `Idl::convert_str()` is called (as in the `error_reporting.rs`
test `test_error_import_nonexistent_file`), it resolves the working
directory via `std::env::current_dir()` (see `src/compiler.rs:131`).
That absolute CWD path is passed to `ImportContext::resolve_import()`
as `current_dir`, and when the import fails, the error message at
`src/import.rs:99` embeds `current_dir.display()`:

```
"import not found: does_not_exist.avsc (searched relative to {} and {} import dir(s))"
```

The resulting error string is then snapshot-tested by `insta`, freezing
the absolute path into the `.snap` file.

## Affected files

### 1. Snapshot with hardcoded absolute path

- **File:** `tests/snapshots/error_reporting__error_import_nonexistent_file.snap`
  (lines 5 and 11)
- **Content:** Contains `/home/jon/dev/stream/avdl/main` twice in the
  rendered error message.
- **Breaks in:** Any worktree, CI runner, or different developer machine.

### 2. Test that produces the path-dependent snapshot

- **File:** `tests/error_reporting.rs`, function
  `test_error_import_nonexistent_file` (line 236)
- **Mechanism:** Calls `compile_error()` which calls
  `Idl::new().convert_str(input)`, which calls
  `std::env::current_dir()` at `src/compiler.rs:131`.

### 3. Source code that embeds the CWD in error messages

- **File:** `src/import.rs`, line 99 in `resolve_import()`
- **Mechanism:** `current_dir.display()` is interpolated into the
  miette error message. When the caller is `convert_str` (no file
  path), `current_dir` is the process CWD, which is an absolute path.

### 4. Unit tests with hardcoded `/tmp` paths (minor)

- **File:** `src/import.rs`, lines 926 and 936
  (inside `#[cfg(test)] mod tests`)
- **Content:** `PathBuf::from("/tmp/test.avdl")` used in
  `mark_imported_returns_false_on_first_call` and
  `mark_imported_returns_true_on_subsequent_calls`.
- **Impact:** These tests only check `HashSet` membership, so the
  path string is arbitrary and doesn't touch the filesystem. They
  work on any machine. This is a cosmetic/style issue, not a
  functional bug.

### 5. Relative paths that depend on CWD (not currently broken)

All integration tests in `tests/integration.rs` and `tests/cli.rs` use
relative paths like `"avro/lang/java/idl/src/test/idl/input"` which
rely on `cargo test` setting the CWD to `CARGO_MANIFEST_DIR`. This is
the documented Cargo behavior and is not currently broken, but it is
worth noting that these paths are not anchored to `CARGO_MANIFEST_DIR`
explicitly.

## Reproduction

```sh
# From a git worktree or different checkout location:
cd /some/other/path/avdl
cargo test test_error_import_nonexistent_file

# The snapshot will fail because the .snap file contains
# /home/jon/dev/stream/avdl/main but the test produces
# /some/other/path/avdl in the error message.
```

## Suggested fix

### For the snapshot (primary fix)

Use `insta`'s snapshot redaction or settings to scrub the CWD from
the snapshot before comparison. For example, in the test or via a
`Settings` block:

```rust
let mut settings = insta::Settings::clone_current();
settings.add_filter(r"(?m)searched relative to .+ and", "searched relative to [CWD] and");
settings.bind(|| {
    insta::assert_snapshot!(error);
});
```

Then re-accept the snapshot so it contains `[CWD]` instead of the
absolute path.

### Alternative: change the error message

Make `resolve_import()` emit only the relative or display-friendly
portion of `current_dir` (e.g., relative to `CARGO_MANIFEST_DIR` or
just `.`). This is a larger change and affects user-facing error
messages, so the snapshot redaction approach is simpler.

### For the `/tmp` paths in unit tests (low priority)

Replace `PathBuf::from("/tmp/test.avdl")` with
`PathBuf::from("dummy/test.avdl")` or similar to make it clear the
path is not real and avoid any implicit dependency on `/tmp` existing.

### For relative paths in integration tests (optional hardening)

Anchor test path constants using `env!("CARGO_MANIFEST_DIR")`:

```rust
fn input_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(INPUT_DIR)
        .join(filename)
}
```

This is not strictly necessary since `cargo test` already sets CWD
to the manifest directory, but it would make the tests resilient to
any future change in test runner behavior.
