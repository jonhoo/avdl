# Trailing comma in enum gives unhelpful error message

## Symptom

When an enum has a trailing comma after the last symbol (a common
mistake, especially for users coming from languages that allow trailing
commas), the error says `unexpected token '}'` with a long "expected one
of" list. It does not mention the trailing comma or suggest removing it.

## Reproduction

```avdl
protocol Test {
  enum Color {
    RED,
    GREEN,
    BLUE,
  }
}
```

```
Error:
  x parse IDL source
  |-> line 6:2 unexpected token `}`
   ,----[tmp/mut-extra8-extra-comma.avdl:6:3]
 5 |     BLUE,
 6 |   }
   .   +
   .   '-- unexpected `}`
 7 | }
   `----
  help: expected one of: protocol, namespace, import, idl, schema, enum,
        fixed, error, record, array, map, union, boolean, int, long, float,
        double, string, bytes, null, true, false, decimal, date, time_ms,
        timestamp_ms, local_timestamp_ms, uuid, void, oneway, throws, @,
        identifier
```

## Root cause

The ANTLR grammar does not allow trailing commas in enum symbol lists
(`enumBody: enumSymbol (',' enumSymbol)*`). When the parser encounters
`}` after a comma, it sees a missing enum symbol rather than a trailing
comma.

The error enrichment code does not handle this specific pattern.

## Suggested fix

In the error enrichment pipeline, detect the pattern where the error
token is `}` and the previous non-whitespace token is `,` inside an
enum declaration. Produce a targeted message:

    trailing comma not allowed in enum declaration
    hint: remove the comma after `BLUE`

Or simply:

    expected enum symbol after `,`, found `}`
    hint: remove the trailing comma
