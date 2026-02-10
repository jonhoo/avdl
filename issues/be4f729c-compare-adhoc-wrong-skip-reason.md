# compare-adhoc.sh shows wrong skip reason in idl2schemata mode

## Symptom

When running `scripts/compare-adhoc.sh --idl2schemata` without the
Java avro-tools JAR available, the skip message incorrectly says
"Rust-only" instead of "Java unavailable":

```
  SKIP  myfile (idl2schemata) (Rust-only)
```

The actual skip logic is correct (it correctly skips the Java
comparison). Only the cosmetic reason string is wrong.

## Root cause

In `compare_idl2schemata` (line 281), the skip reason uses:

```bash
report_skip "$basename (idl2schemata)" "${RUST_ONLY:+Rust-only}${AVRO_JAR:+Java unavailable}"
```

The parameter expansion `${RUST_ONLY:+Rust-only}` tests whether
`RUST_ONLY` is non-empty. But `RUST_ONLY` is always either `"true"`
or `"false"` -- both are non-empty strings. So it always expands to
`"Rust-only"` regardless of its boolean value.

Similarly, `${AVRO_JAR:+Java unavailable}` expands to
`"Java unavailable"` when the JAR IS available (non-empty), which is
the opposite of the intended meaning.

The three cases:

| `RUST_ONLY` | `AVRO_JAR` | Expected message     | Actual message                 |
|-------------|------------|----------------------|--------------------------------|
| `true`      | set        | Rust-only            | Rust-onlyJava unavailable      |
| `true`      | empty      | Rust-only            | Rust-only                      |
| `false`     | empty      | Java unavailable     | Rust-only                      |

## Affected files

- `scripts/compare-adhoc.sh` (line 281)

## Reproduction

```sh
unset AVRO_TOOLS_JAR
# Remove or rename ../avro-tools-1.12.1.jar so it can't be found
scripts/compare-adhoc.sh --idl2schemata tmp/some-file.avdl
# Observe: "SKIP ... (Rust-only)" instead of "SKIP ... (Java unavailable)"
```

## Suggested fix

Replace the bash parameter expansions with explicit conditional logic:

```bash
local reason=""
if [ "$RUST_ONLY" = true ]; then
    reason="Rust-only"
else
    reason="Java unavailable"
fi
report_skip "$basename (idl2schemata)" "$reason"
```
