# "Undefined name" error does not suggest similar type names

- **Symptom**: When a type name is misspelled, the error shows the
  undefined name but does not suggest similar types that exist:

  ```
  Error:   x Undefined name: test.stiring
  ```

  For common typos of primitive types (`stiring` for `string`, `Int`
  for `int`), the error could suggest the correct type. For typos of
  user-defined types, it could list types with similar names.

- **Root cause**: The `validate_all_references` function in
  `compiler.rs` only checks whether a name exists in the registry. It
  does not compute similarity to existing names.

- **Reproduction**:
  ```avdl
  @namespace("test")
  protocol Test {
    record Foo {
      stiring name;
    }
  }
  ```

  ```
  Error:   x Undefined name: test.stiring
  ```

- **Suggested fix**: When reporting an undefined name:
  1. Check if the unqualified name is within edit distance 1-2 of a
     primitive type (`string`, `int`, `long`, `float`, `double`,
     `boolean`, `bytes`, `null`). If so, suggest it.
  2. Check registered type names for similar matches using edit
     distance or prefix matching.
  3. For capitalization errors like `String` vs `string`, provide a
     specific hint: "Avro primitive types are lowercase".

  Example improved error:
  ```
  Error: Undefined name: test.stiring
    help: did you mean 'string'? (note: Avro primitives are lowercase)
  ```
