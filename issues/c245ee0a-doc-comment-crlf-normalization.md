# Document that `normalize_crlf` mirrors Java's test-level normalization

Add a comment to the `normalize_crlf` helpers in `tests/integration.rs`
and `tests/cli.rs` noting that the Java test suite applies the same
normalization before comparing output:

- `TestIdlReader.java:232` — `output.replace("\r", "")`
- `TestIdlTool.readFileAsString` (lines 102–104) —
  `BufferedReader.lines().collect(joining("\n"))`

This reassures readers that we're not papering over a bug — Java's
tests do the exact same thing.

## Affected files

- `tests/integration.rs` — `normalize_crlf`
- `tests/cli.rs` — `normalize_crlf`
