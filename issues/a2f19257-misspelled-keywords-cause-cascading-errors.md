# Misspelled keywords cause cascading parse errors

- **Symptom**: When a keyword like `record` is misspelled as `recrod`,
  the parser produces a cascade of confusing errors instead of
  suggesting the correct keyword:

  ```
  Error: line 4:13 unexpected token `{`
    help: expected one of: protocol, namespace, import, ...

  Error: line 5:15 unexpected token `;`
    help: expected one of: protocol, namespace, import, ...

  Error: line 7:0 extraneous input '}' expecting {<EOF>, '\u001A'}
  ```

  The first error points at `{` rather than at `recrod`. The help
  message lists many tokens but does not suggest that `recrod` looks
  like `record`.

- **Root cause**: The parser treats `recrod` as an identifier (a type
  reference), so it enters a different grammar rule. The subsequent
  tokens (`{`, field definitions, `}`) don't match that rule, causing
  cascading failures.

- **Reproduction**:
  ```avdl
  @namespace("test")
  protocol Test {
    recrod Foo {
      string name;
    }
  }
  ```

- **Suggested fix**: After a parse failure, check if any identifier
  token in the error region is within edit distance 1-2 of a keyword.
  If so, suggest the correction:

  ```
  Error: line 4:2 unrecognized 'recrod' -- did you mean 'record'?
  ```

  This could be implemented as a post-processing pass over parse errors
  that examines tokens near error locations.
