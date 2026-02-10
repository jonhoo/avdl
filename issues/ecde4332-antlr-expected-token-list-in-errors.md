# ANTLR parser errors show unhelpful expected-token lists

## Symptom

When ANTLR produces "mismatched input" or "extraneous input" errors,
the expected-token set is dumped verbatim into the error message. This
set often contains 20-30 token names (all the Avro keywords, type
names, `IdentifierToken`, etc.) and renders as a wall of text that
obscures the actual problem. The same long message also appears twice
in the rendered output: once as the top-level error text and again in
the source-underline label.

Example (missing closing brace):

```
Error:   x parse IDL source
  |-> line 6:0 extraneous input '<EOF>' expecting {DocComment, 'protocol',
      'namespace', 'import', 'idl', 'schema', 'enum', 'fixed', 'error',
      'record', 'array', 'map', 'union', 'boolean', 'int', 'long', 'float',
      'double', 'string', 'bytes', 'null', 'true', 'false', ...}
   ,----[file.avdl:6:1]
 5 | }
   . ^--- line 6:0 extraneous input '<EOF>' expecting {...same long list...}
   `----
```

Other cases that produce this pattern:
- `protocol { ... }` (missing protocol name)
- `recrod Foo { ... }` (misspelled `record` keyword)
- `} extra_token_here` (extraneous token after closing brace)

## Root cause

`ParseDiagnostic` uses the same `message` string for both its
`Display` impl (top-level error text) and its `labels()` impl
(source-underline label). Additionally, the `enrich_antlr_error`
function in `reader.rs` only rewrites two specific ANTLR error
patterns but does not handle the common "mismatched/extraneous input
... expecting {large set}" pattern.

## Affected files

- `src/error.rs` -- `ParseDiagnostic` uses `message` for both
  `Display` and label text
- `src/reader.rs` -- `enrich_antlr_error` does not handle the
  expected-token-set pattern; `CollectingErrorListener::syntax_error`
  formats the message

## Reproduction

```sh
echo 'protocol Test { record Foo { string name' > tmp/test.avdl
cargo run -- idl tmp/test.avdl
```

Any syntax error that triggers ANTLR's "expecting {...}" message will
reproduce this.

## Suggested fix

Two complementary improvements:

1. **Separate the label from the Display message in
   `ParseDiagnostic`.** The label should be a shorter summary (e.g.,
   just "unexpected token" or "expected `;`"), while the full message
   with the expected-token list goes in the top-level Display text.
   This eliminates the duplication.

2. **Extend `enrich_antlr_error` to truncate or summarize the
   expected-token set.** When the set contains more than ~5 tokens,
   replace it with a summary like "expected a type declaration, field
   definition, or closing `}`". This requires mapping ANTLR's rule
   contexts to human-readable descriptions, which can be done
   incrementally for the most common error contexts.
