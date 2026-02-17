# ANTLR jargon leaks into several error messages

## Symptom

Several error messages expose raw ANTLR internal phrasing that is
confusing to users unfamiliar with parser internals:

1. **"no viable alternative at input"** -- appears for non-integer
   fixed sizes (`fixed Bad(3.5);`) and double `@@`:

   ```
   line 2:12 no viable alternative at input 'fixedBad(3.5'
   ```

   The concatenated `fixedBad(3.5` is particularly confusing as it
   merges the keyword, name, and literal without spaces.

2. **`{';', ','}` set notation** -- appears in missing-semicolon
   contexts:

   ```
   line 4:4 mismatched input 'Baz' expecting {';', ','}
   ```

   Should use natural language like `expected ';' or ','`.

3. **"extraneous input" / "mismatched input"** -- these are ANTLR
   error-strategy terms that do not mean much to end users.

## Reproduction

```avdl
protocol Test {
  fixed Bad(3.5);
}
```

```
line 2:12 no viable alternative at input 'fixedBad(3.5'
```

```avdl
protocol Test {
  record Foo {
    string name
    int age;
  }
}
```

```
line 4:4 mismatched input 'Baz' expecting {';', ','}
```

## Root cause

These messages come from ANTLR's `DefaultErrorStrategy`, which produces
structured error messages using parser-internal terminology. The
`reader.rs` error enrichment logic handles some common cases (e.g.,
misspelled keywords, unterminated strings) but does not rewrite all
ANTLR error patterns.

## Affected files

- `src/reader.rs` (ANTLR error listener and enrichment logic)

## Suggested fix

Add pattern matching for these additional ANTLR error message formats in
the error enrichment pipeline:

1. **"no viable alternative at input '...'"** -- extract the actual
   offending token and produce a message like "unexpected `3.5` in fixed
   size declaration (expected an integer)".

2. **`{...}` set notation** -- reformat `expecting {';', ','}` to
   `expected ';' or ','`.

3. **"extraneous input"** -- rewrite to "unexpected <token>" (which the
   code already does for some cases but not all).

4. **"mismatched input"** -- rewrite to "unexpected <token>, expected
   <list>".

The concatenated input strings like `fixedBad(3.5` should be split back
into their constituent tokens for display.
