# Java avro-tools crashes on out-of-range int default values

## Symptom

Java `avro-tools idl` crashes with `NoSuchElementException` when an
`.avdl` file contains an integer default value that overflows `int`
(32-bit signed):

```
Exception in thread "main" org.apache.avro.SchemaParseException:
    java.util.NoSuchElementException
  at org.apache.avro.idl.IdlReader.parse(IdlReader.java:220)
  ...
Caused by: java.util.NoSuchElementException
  at java.base/java.util.ArrayDeque.removeFirst(ArrayDeque.java:361)
  at java.base/java.util.ArrayDeque.pop(ArrayDeque.java:592)
  at org.apache.avro.idl.IdlReader$IdlParserListener
      .exitVariableDeclaration(IdlReader.java:623)
```

## Reproduction

```avro
@namespace("test")
protocol P {
  record R {
    int z = 2147483648;
  }
}
```

```sh
java -jar avro-tools-1.12.1.jar idl test.avdl
# Crashes with NoSuchElementException
```

The value `2147483648` is one past `Integer.MAX_VALUE` (2,147,483,647).

## Root cause

The Java `IdlReader` likely fails to push a value onto its internal
stack when the integer literal overflows Java's `int` type, causing
`exitVariableDeclaration` to pop from an empty `ArrayDeque`.

## Version

Observed in avro-tools 1.12.1.
