# Error message shows confusing '\u001A' control character

- **Symptom**: When there is extra content after the closing brace of a
  protocol, the error message shows `expecting {<EOF>, '\u001A'}`. The
  `\u001A` is the ASCII SUB (substitute) character, which ANTLR uses as
  an alternate end-of-file marker. This is confusing ANTLR jargon that
  means nothing to users trying to fix their IDL.

- **Root cause**: The error message is passed through directly from
  ANTLR without sanitizing the token names. ANTLR's internal EOF token
  representation includes `\u001A` alongside `<EOF>`.

- **Reproduction**:
  ```avdl
  @namespace("test")
  protocol Test {
    record Foo {
      string name;
    }
  } extra
  ```

  ```
  Error:   x parse IDL source
    |-> line 7:2 extraneous input 'extra' expecting {<EOF>, '\u001A'}
  ```

- **Suggested fix**: Post-process ANTLR error messages to:
  1. Remove `'\u001A'` entirely from the expected token list
  2. Convert `<EOF>` to a more user-friendly phrase like "end of file"

  The fix could be applied in the error formatting layer that already
  translates ANTLR errors to `ParseDiagnostic`.
