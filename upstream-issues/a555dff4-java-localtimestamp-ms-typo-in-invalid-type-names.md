# Java `INVALID_TYPE_NAMES` contains `localtimestamp_ms` instead of `local_timestamp_ms`

## Symptom

The Java `IdlReader.INVALID_TYPE_NAMES` set (line 146 of IdlReader.java)
contains `"localtimestamp_ms"` (without the underscore between `local`
and `timestamp`), but the ANTLR grammar keyword token text is
`local_timestamp_ms` (with an underscore):

```java
// IdlReader.java line 145-146
private static final Set<String> INVALID_TYPE_NAMES = new HashSet<>(Arrays.asList(
    "boolean", "int", "long", "float", "double", "bytes", "string",
    "null", "date", "time_ms", "timestamp_ms", "localtimestamp_ms", "uuid"));
```

```
// Idl.g4 line 201
LocalTimestamp: 'local_timestamp_ms';
```

The ANTLR parser tokenizes `` `local_timestamp_ms` `` as the keyword
`LocalTimestamp`, but when the name flows through `identifier()` it
becomes `"local_timestamp_ms"`. The `INVALID_TYPE_NAMES` check at line
956 compares this against `"localtimestamp_ms"`, which never matches.

## Effect

`` record `local_timestamp_ms` { int x; } `` is accepted by both Java
avro-tools 1.12.1 and our Rust tool, even though `local_timestamp_ms`
is a reserved keyword and should be rejected as a type name. The same
check correctly rejects other keyword-based names like `` `date` ``,
`` `timestamp_ms` ``, etc.

## Reproduction

```sh
echo '@namespace("test") protocol P { record `local_timestamp_ms` { int x; } }' > /tmp/test.avdl
java -jar avro-tools-1.12.1.jar idl /tmp/test.avdl
# Succeeds (should fail with "Illegal name: local_timestamp_ms")
```

## Note

Our Rust code has the same typo (`"localtimestamp_ms"` at reader.rs
lines 551 and 713), mirrored from the Java source. This is tracked
separately in `issues/fd00c8bb-local-timestamp-ms-typo-in-keyword-lists.md`.
