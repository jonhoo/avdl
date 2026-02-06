# `compare-golden.sh` cannot find Java JAR from worktrees

## Symptom

When `scripts/compare-golden.sh` is run from a git worktree (e.g.,
`avdl-worktrees/wt-a/`), it cannot find the `avro-tools-1.12.1.jar`
file because the script resolves paths relative to the worktree root,
not the main repository root.

This also affects the `idl2schemata` comparison for the `import` test
case: the script passes `--import-dir` flags to both tools, but the
Java tool ignores them (it uses the JVM classpath instead), causing a
parse failure on Java's side.

## Root cause

The script uses a relative path to locate the Java JAR, but worktrees
have a different filesystem root than the main checkout. The JAR lives
at `../avro-tools-1.12.1.jar` relative to the main repo root, which
doesn't resolve from a worktree directory.

## Affected files

- `scripts/compare-golden.sh`

## Reproduction

```sh
cd avdl-worktrees/wt-a
scripts/compare-golden.sh idl simple
# Fails: cannot find avro-tools JAR
```

## Suggested fix

Accept an `AVRO_TOOLS_JAR` environment variable to override the
default path. Fall back to searching common locations:
`../avro-tools-1.12.1.jar`, `../../avro-tools-1.12.1.jar`, and
the main repo root (via `git worktree list` to find the main
checkout).

## Priority

Low. The script works correctly from the main checkout. Worktree
usage is only needed by sub-agents during parallel development.
