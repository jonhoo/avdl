# Enum default with bare identifier produces confusing ANTLR error

## Symptom

Writing an enum default as a bare identifier:

```avro
enum Color { RED, GREEN, BLUE }
record Palette { Color primary = YELLOW; }
```

produces a confusing ANTLR error about expecting `'null', 'true',
'false', '{', '[', StringLiteral, ...` instead of a clear message
like "expected a quoted string for enum default".

The correct syntax is `Color primary = "YELLOW";` (a JSON string
literal), but the error gives no hint that quoting is needed.

## Root cause

The ANTLR grammar's `jsonValue` rule (used for field defaults) expects
a JSON literal, not a bare identifier. When the parser encounters
`YELLOW` (an `IdentifierToken`), it doesn't match any `jsonValue`
alternative, so ANTLR emits a generic "mismatched input" error with
the full expected-token set.

The grammar has no special-case production for bare enum symbols in
default positions, so this is fundamentally a grammar design choice
rather than a bug in ANTLR itself.

## Impact

Low — users learn the correct syntax quickly, and the Avro IDL
documentation shows quoted defaults. But the error is unhelpful for
newcomers.

## Possible upstream fix

Add a parser rule or error recovery strategy that recognizes a bare
identifier in default position and suggests quoting it. Alternatively,
the grammar could accept bare identifiers as enum defaults (matching
what users intuitively expect) and validate them semantically.
