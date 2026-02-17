# Missing protocol name produces cascade of unhelpful errors

## Symptom

When the user writes `protocol { ... }` (missing the protocol name),
the tool produces 4 separate error diagnostics, none of which say
"expected protocol name". The first error says `unexpected token '{'`
with a long "expected one of" list. Subsequent errors are cascading
consequences of the parser's recovery attempt.

## Reproduction

```avdl
protocol {
  record Foo {
    string name;
  }
}
```

```
Error:
  x parse IDL source
  |-> line 1:9 unexpected token `{`
   ...
  help: expected one of: protocol, namespace, import, idl, schema, enum,
        fixed, error, record, array, map, union, boolean, int, long, ...

Error:
  x line 2:9 extraneous input 'Foo' expecting '{'

Error:
  x line 3:15 unexpected token `;`

Error:
  x line 5:0 extraneous input '}' expecting end of file
```

## Root cause

The ANTLR parser sees `protocol {` and interprets `{` as the start of
the body (skipping the missing name), then everything downstream breaks.
The error enrichment code does not have a specific pattern for "missing
identifier after `protocol`".

## Suggested fix

In the error enrichment pipeline, detect the pattern where the previous
token is `protocol` (or `record`, `enum`, `fixed`, `error`) and the
error token is `{`. Produce a targeted message:

    expected name after `protocol`, found `{`
    hint: `protocol MyProtocol { ... }`

This same pattern could also apply to `record {` (missing record name),
`enum {`, etc.

Additionally, when the first parse error results in 3+ cascading errors,
consider showing only the first error with a note like "(N additional
errors omitted -- fix the first error and re-run)".
