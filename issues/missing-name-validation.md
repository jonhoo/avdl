# Missing name validation: `validateName` and `INVALID_TYPE_NAMES`

## Symptom

The Rust tool accepts type names and namespace components that the Java
implementation would reject. Specifically:

1. **Invalid characters in names**: Java validates that identifiers match
   the pattern `[_\p{L}][_\p{LD}]*` (Unicode letters/digits) via
   `VALID_NAME` (IdlReader.java line 141-143). The Rust tool performs
   no such validation after extracting identifiers.

2. **Reserved type names**: Java rejects type names that collide with
   built-in Avro types via `INVALID_TYPE_NAMES` (IdlReader.java lines
   145-146):

   ```java
   private static final Set<String> INVALID_TYPE_NAMES = new HashSet<>(
       Arrays.asList("boolean", "int", "long", "float", "double",
           "bytes", "string", "null", "date", "time_ms",
           "timestamp_ms", "localtimestamp_ms", "uuid"));
   ```

   A user could define `record int { ... }` or `enum string { ... }`
   and the Rust tool would silently produce output that may confuse
   downstream Avro consumers.

3. **Namespace component validation**: Java validates each component
   of a dotted namespace (e.g., in `com.example.MyType`, it validates
   `com`, `example`, and `MyType` separately) via the `namespace()`
   method (IdlReader.java lines 938-948). The Rust tool does not
   validate namespace components.

## Root cause

The Java `IdlReader` has a `validateName` method (lines 950-960) that
is called from `name()` and `namespace()`:

```java
private String validateName(String name, boolean isTypeName) {
    if (name == null) {
        throw new SchemaParseException("Null name");
    } else if (!VALID_NAME.test(name)) {
        throw new SchemaParseException("Illegal name: " + name);
    }
    if (isTypeName && INVALID_TYPE_NAMES.contains(name)) {
        throw new SchemaParseException("Illegal name: " + name);
    }
    return name;
}
```

The Rust implementation's `extract_name` and `compute_namespace`
functions (reader.rs lines 1385-1418) only split the identifier at
dots; they perform no validation of the resulting name parts.

Note: The ANTLR grammar itself restricts identifiers to valid tokens,
so syntactically invalid characters are already caught by the parser.
The main gap is the `INVALID_TYPE_NAMES` check, which is a *semantic*
validation that the grammar does not enforce. A user can write
`record int { ... }` because `int` is a valid identifier token (it's
a keyword, but backtick-escaped identifiers bypass keyword checks).

## Affected files

- `src/reader.rs` -- `extract_name`, `compute_namespace`, and all
  callers that process type identifiers

## Reproduction

```avdl
@namespace("test")
protocol P {
    record `int` {
        string value;
    }
}
```

Java rejects this with `SchemaParseException: Illegal name: int`.
Rust accepts it and produces:

```json
{
  "types": [{"type": "record", "name": "int", "fields": [...]}]
}
```

## Suggested fix

1. Add an `INVALID_TYPE_NAMES` constant matching Java's list.
2. Add a `validate_name` function that checks the name against the
   invalid names set and (optionally) the Unicode pattern.
3. Call `validate_name` from `walk_record`, `walk_enum`, `walk_fixed`,
   and `walk_protocol` after extracting the type name.

## Priority

Medium. While the ANTLR grammar prevents most truly malformed names,
the reserved type name check prevents confusing schema output where a
type named `int` could shadow the built-in primitive.
