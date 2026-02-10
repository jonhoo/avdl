# Java avro-tools crashes on `"NaN"` and `"Infinity"` as float/double defaults

## Symptom

When a float or double field has a string default value of `"NaN"`,
`"Infinity"`, or `"-Infinity"`, Java avro-tools 1.12.1 crashes with a
`NoSuchElementException` stack trace. Our Rust tool handles these
correctly by emitting the string value in JSON (since JSON has no
native NaN/Infinity representation).

## Reproduction

```avdl
@namespace("test")
protocol FloatNan {
  record R {
    float nan = "NaN";
  }
}
```

```sh
java -jar avro-tools-1.12.1.jar idl /tmp/test-float-nan.avdl
# Exception in thread "main" org.apache.avro.SchemaParseException: java.util.NoSuchElementException
```

The same crash occurs with `"Infinity"` and `"-Infinity"`.

## Root cause

This is the same stack corruption pattern as the out-of-range integer
default bug (tracked in `1ec6f2bf`). The ANTLR grammar's `jsonLiteral`
rule accepts `StringLiteral` tokens, so `"NaN"` is parsed as a valid
JSON literal (a string). However, Java's `Schema.Field` constructor
with `validate=true` rejects a string default value for a float field,
throwing an exception that corrupts the internal type/property stacks.

The Avro JSON specification allows float/double defaults to be JSON
numbers, but `NaN` and `Infinity` are not representable as JSON numbers.
The common convention is to encode them as strings, which is what our
tool does. The Java IDL parser does not support this convention.

## Impact

Users cannot express NaN/Infinity defaults for float/double fields in
Avro IDL when using Java avro-tools. Our Rust tool accepts these values
and produces semantically correct JSON output.
