# Misspelled `protocol` keyword does not get a "did you mean" suggestion

## Symptom

When the user misspells `protocol` as `protocl`, the error says
`unexpected token 'protocl'` with a long `expected one of` list, but does
not suggest `protocol`. In contrast, misspelling `record` as `recrd` or
`enum` as `enm` within a protocol body produces a clear
`did you mean 'record'?` / `did you mean 'enum'?` suggestion.

## Reproduction

```avdl
protocl Test {
  record Foo {
    string name;
  }
}
```

```
Error:
  x parse IDL source
  |-> line 1:0 unexpected token `protocl`
   ,----[tmp/mut02c-protocl.avdl:1:1]
 1 | protocl Test {
   . ---+---
   .    '-- unexpected `protocl`
 2 |   record Foo {
   `----
  help: expected one of: protocol, namespace, import, schema, enum, fixed,
        error, record, @
```

## Root cause

The "did you mean" logic in `reader.rs` (`suggest_keyword`) appears to
run only for ANTLR errors inside the protocol body (where tokens are
parsed as identifiers by the grammar rule). At the top level, ANTLR
produces a different error path and the misspelled keyword does not get
routed through `suggest_keyword`.

## Suggested fix

Apply the same Levenshtein-based keyword suggestion logic to top-level
parse errors. The `expected one of` list already contains the candidate
keywords, so the fix is to scan that list for close matches when the
error token is an unrecognized identifier.
