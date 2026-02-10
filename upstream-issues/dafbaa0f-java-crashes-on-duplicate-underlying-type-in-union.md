# [IDL] Java crashes with NoSuchElementException on union containing types with same underlying type

- **Component:** java / idl
- **Affects Version:** 1.12.1

## Description

When a union contains two types that resolve to the same underlying
Avro type (e.g., `int` and `date`, where `date` is backed by `int`),
`avro-tools idl` crashes with an unhandled `NoSuchElementException`
instead of producing a clear error message. The Avro specification
states that unions may not contain more than one schema with the same
type, so this input is invalid -- but the tool should reject it
gracefully rather than crashing.

### Minimal reproduction

```avdl
@namespace("org.test")
protocol P {
  record R {
    union { int, date } field;
  }
}
```

```
$ java -jar avro-tools-1.12.1.jar idl union-int-date.avdl
Exception in thread "main" org.apache.avro.SchemaParseException: java.util.NoSuchElementException
        at org.apache.avro.idl.IdlReader.parse(IdlReader.java:220)
        ...
Caused by: java.util.NoSuchElementException
        at java.base/java.util.ArrayDeque.removeFirst(ArrayDeque.java:361)
        at java.base/java.util.ArrayDeque.pop(ArrayDeque.java:592)
        at org.apache.avro.idl.IdlReader$IdlParserListener.exitUnionType(IdlReader.java:859)
```

Also crashes for `union { null, int, date }` and similar combinations
(e.g., `union { long, timestamp_ms }`, `union { string, uuid }`).

### Expected behavior

A clear error message such as:

```
Duplicate in union: int
```

### Actual behavior

An unhandled `NoSuchElementException` originating from
`exitUnionType` when it tries to pop from the type stack.

## Root cause

Java's `Schema.createUnion(types)` constructor calls
`new UnionSchema(types)`, which internally builds an `objectTypes`
map keyed by each schema's full name (or type name for anonymous
types). For both `int` and `date`, the key is `"int"` because
`date`'s underlying type is `int`. When the duplicate is detected,
the union constructor throws, but this exception is caught (or
causes stack corruption) somewhere in the listener flow, leaving the
type stack in an inconsistent state. The subsequent `pop()` call on
the now-depleted stack throws `NoSuchElementException`.

## Impact

Low -- this is an invalid IDL input that should be rejected. The
issue is that the rejection mechanism is a stack trace dump rather
than a user-friendly error message.

Our Rust avdl tool handles this correctly, producing:

```
Duplicate in union: int
```
