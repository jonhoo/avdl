# Bare `@` sign merges following tokens into annotation name

## Symptom

When a user types `@ record Foo {` (bare `@` followed by whitespace),
the error message reports the annotation name as `@recordFoo`, merging
the `record` keyword and `Foo` identifier into a single annotation name.
This is confusing because the user did not intend to create an annotation
named `@recordFoo`.

## Reproduction

```avdl
protocol Test {
  @ record Foo {
    string name;
  }
}
```

```
Error:
  x parse IDL source
  |-> line 2:11 annotation `@recordFoo` is missing its value -- use
      `@recordFoo("value")` syntax
   ,----[tmp/mut10a-at-alone.avdl:2:12]
 1 | protocol Test {
 2 |   @ record Foo {
   .            -+-
   .             '-- line 2:11 annotation `@recordFoo` is missing its value -- use `@recordFoo("value")` syntax
 3 |     string name;
   `----
```

## Root cause

The ANTLR grammar defines annotation names as dotted identifiers after
`@`, and since `record` is also a valid identifier token, the parser
greedily consumes `record` and `Foo` as parts of the annotation name
when they follow `@` without parentheses.

The error-reporting code then uses this merged name in the diagnostic
without realizing the tokens were not intended as an annotation.

## Suggested fix

When reporting a "missing annotation value" error, check whether the
resulting annotation name contains an Avro keyword (`record`, `enum`,
`fixed`, `error`, `protocol`). If so, the error message should instead
say something like:

    unexpected `@` before `record` -- did you mean to add an annotation?
    annotations use the syntax `@name("value")`

This would help the user understand the `@` was out of place rather than
suggesting they complete a phantom annotation named `@recordFoo`.
