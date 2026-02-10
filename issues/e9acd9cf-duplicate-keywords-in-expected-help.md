# Expected-token help text contains duplicate keywords

## Symptom

When a missing-brace parse error triggers the `format_expected_help`
path, the help text lists several keywords twice:

```
help: expected one of: protocol, namespace, import, idl, schema, enum,
      fixed, error, record, array, map, union, boolean, int, long, float,
      double, string, bytes, null, true, false, decimal, date, time_ms,
      timestamp_ms, local_timestamp_ms, uuid, void, oneway, throws, decimal,
      date, time_ms, timestamp_ms, local_timestamp_ms, uuid, void,
```

`decimal`, `date`, `time_ms`, `timestamp_ms`, `local_timestamp_ms`,
`uuid`, and `void` each appear twice. The list also ends with a
trailing comma.

## Root cause

The ANTLR grammar defines these tokens in multiple alternatives
(e.g., both as primitive types and as result types), so the
expected-token set from the parser error message contains duplicates.
`format_expected_help` in `reader.rs` splits, filters, and joins
the token list without deduplicating it.

## Affected files

- `src/reader.rs` -- `format_expected_help` function (around line 420)

## Reproduction

```sh
cat > tmp/dup-keywords.avdl <<'EOF'
@namespace("org.test")
protocol Test {
  record Foo {
    string name;
    int age;

}
EOF
cargo run -- idl tmp/dup-keywords.avdl 2>&1
```

## Suggested fix

Deduplicate the `cleaned` vector in `format_expected_help` before
joining. An `IndexSet` or a simple `dedup()` after sorting would
remove the repeated entries while preserving the original order. Also
ensure the join does not produce a trailing comma (verify no empty
strings slip through the filter).
