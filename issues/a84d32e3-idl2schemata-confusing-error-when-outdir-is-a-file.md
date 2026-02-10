# `idl2schemata` gives confusing error when output dir is an existing file

## Symptom

When the user passes a path to an existing file as the output
directory for `idl2schemata`, the error message is:

```
Error:   x File exists (os error 17): create output directory
```

This doesn't explain that the path points to a regular file, not a
directory. The user might think there's a permissions issue or a name
collision. A clearer message would be:

```
Error:   x cannot create output directory `path/to/file.txt`: path exists and is not a directory
```

## Root cause

`run_idl2schemata` in `main.rs` calls `fs::create_dir_all(&output_dir)`
which fails with `EEXIST` when the path is an existing file. The error
is wrapped with generic context "create output directory" without
checking whether the cause is "path is a file" vs "permissions" vs
another OS error.

## Affected files

- `src/main.rs` -- `run_idl2schemata`

## Reproduction

```sh
touch tmp/output.txt
cargo run -- idl2schemata tmp/good.avdl tmp/output.txt
```

## Suggested fix

Before calling `create_dir_all`, check if the path exists and is not a
directory. If so, produce a specific error message:

```rust
if output_dir.exists() && !output_dir.is_dir() {
    return Err(miette::miette!(
        "output path `{}` exists and is not a directory",
        output_dir.display()
    ));
}
```
