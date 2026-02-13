# "No viable alternative" error concatenates unrelated tokens

- **Symptom**: When a keyword is misspelled (e.g., `protocl` instead of
  `protocol`), the error message shows:

  ```
  line 3:0 no viable alternative at input '@namespace("test")protocl'
  ```

  The message concatenates the namespace annotation with the misspelled
  keyword, making it unclear what exactly is wrong. Users may think the
  namespace annotation is the problem.

- **Root cause**: ANTLR's "no viable alternative" error includes the
  entire input consumed up to the error point. This default formatting
  is unhelpful for Avro IDL because it mixes unrelated tokens.

- **Reproduction**:
  ```avdl
  @namespace("test")
  protocl Test {
    record Foo {
      string name;
    }
  }
  ```

  ```
  Error:   x parse IDL source
    |-> line 3:0 no viable alternative at input '@namespace("test")protocl'
  ```

- **Suggested fix**: Customize the ANTLR error listener to extract the
  actual problematic token from "no viable alternative" errors. The
  message could be rewritten as:

  ```
  line 3:0 unrecognized token 'protocl' -- did you mean 'protocol'?
  ```

  This requires adding a typo-detection heuristic that compares
  unrecognized identifiers against known keywords using edit distance.
