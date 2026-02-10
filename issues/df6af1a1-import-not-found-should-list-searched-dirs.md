# Import-not-found error should list the directories that were searched

## Symptom

When an import file cannot be found, the error message says:

```
import not found: some-file.avsc (searched relative to /path/to/dir and 1 import dir(s))
```

The number of import dirs is shown, but the actual directory paths are
not. When a user has misconfigured `--import-dir` flags (e.g., a typo
in the path), the current message doesn't help them spot the mistake.
They need to know which directories were actually searched.

## Root cause

`ImportContext::resolve_import` in `import.rs` formats the error with
`self.import_dirs.len()` but does not include the directory paths.

## Affected files

- `src/import.rs` -- `resolve_import` method

## Reproduction

```sh
cargo run -- idl --import-dir /path/to/wrong/dir input.avdl
```

Where `input.avdl` imports a file that exists in a different directory.

## Suggested fix

Include the import directory paths in the error message. For a small
number of dirs (say, 1-3), list them inline:

```
import not found: some-file.avsc
  searched: /path/to/input/dir, /path/to/wrong/dir
```

For many dirs, use a bulleted list or truncate.
