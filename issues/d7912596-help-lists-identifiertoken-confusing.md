# Help text shows "IdentifierToken" instead of user-friendly description

- **Symptom**: Many error messages include `IdentifierToken` in the
  "expected one of" list:

  ```
  help: expected one of: protocol, namespace, import, ..., void, oneway,
        throws, IdentifierToken
  ```

  `IdentifierToken` is internal ANTLR terminology. Users don't know
  what an "IdentifierToken" is -- they would understand "identifier" or
  "type name" or "field name".

- **Root cause**: The error message is generated from ANTLR token
  names, which include `IdentifierToken` as the lexer rule name.

- **Reproduction**:
  ```avdl
  @namespace("test")
  protocol Test {
    record Foo {
      array<> items;
    }
  }
  ```

  ```
  help: expected one of: protocol, namespace, import, idl, schema, enum,
        fixed, error, record, array, map, union, boolean, int, long, float,
        double, string, bytes, null, true, false, decimal, date, time_ms,
        timestamp_ms, local_timestamp_ms, uuid, void, oneway, throws, @,
        IdentifierToken
  ```

- **Suggested fix**: Post-process error messages to replace
  `IdentifierToken` with a context-appropriate term:
  - "identifier" for general cases
  - "type name" when a type is expected
  - "field name" when a field identifier is expected

  This could be done in the error formatting layer or by customizing
  the ANTLR vocabulary's display names.
