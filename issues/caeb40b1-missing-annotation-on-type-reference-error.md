# Missing error: annotations on type references are not rejected

## Summary

The Rust tool silently accepts annotations placed on type references
(e.g., `@foo("bar") MD5 hash = ...` where `MD5` is a previously
defined fixed type). The Java implementation throws
`SchemaParseException` with the message "Type references may not be
annotated, at line 29, column 16". Our tool should produce an
equivalent error.

## Reproduction

```sh
cargo run -- idl avro/lang/java/idl/src/test/idl/AnnotationOnTypeReference.avdl tmp/annotref.avpr
```

**Expected:** The tool exits with a non-zero status and an error
message indicating that type references may not be annotated.

**Actual:** The tool succeeds (exit code 0) and produces valid JSON
output. The `@foo("bar")` annotation is silently dropped -- the
`hash` field in the output has no `foo` property:

```json
{
  "doc": "A stripped down version of a previous `simple.avdl`...",
  "messages": {},
  "namespace": "org.apache.avro.test",
  "protocol": "Simple",
  "types": [
    {
      "doc": "An MD5 hash.",
      "name": "MD5",
      "size": 16,
      "type": "fixed"
    },
    {
      "doc": "A TestRecord.",
      "fields": [
        {
          "default": "0000000000000000",
          "name": "hash",
          "type": "MD5"
        }
      ],
      "name": "TestRecord",
      "type": "record"
    }
  ]
}
```

## Root cause

The Java implementation checks in `exitNullableType` (IdlReader.java
line 776-777) whether a type reference has accumulated any properties
on the `propertiesStack`. If it has, it throws an error because
annotations on type references are semantically invalid -- the
annotation is ambiguous (does it apply to the field or the type?).

In the Rust `walk_full_type` / `walk_nullable_type` code path, when
a type reference is encountered, the accumulated schema properties
from annotations like `@foo("bar")` are not checked. The reference
is resolved and the properties are silently discarded.

## Java reference

`IdlReader.java`, lines 768-787:

```java
@Override
public void exitNullableType(NullableTypeContext ctx) {
  Schema type;
  if (ctx.referenceName == null) {
    type = typeStack.pop();
  } else {
    if (propertiesStack.isEmpty() || propertiesStack.peek().hasProperties()) {
      throw error("Type references may not be annotated",
                  ctx.getParent().getStart());
    }
    type = namedSchemaOrUnresolved(...);
  }
  ...
}
```

The corresponding Java test is `TestReferenceAnnotationNotAllowed`
(`TestReferenceAnnotationNotAllowed.java`), which parses
`AnnotationOnTypeReference.avdl` and asserts that parsing throws
`AvroRuntimeException`.

## Affected file

`src/reader.rs` -- the `walk_nullable_type` or `walk_full_type`
function needs to check whether properties have been accumulated
before resolving a type reference, and return an error if they have.

## Test input

`avro/lang/java/idl/src/test/idl/AnnotationOnTypeReference.avdl`:

```avdl
@namespace("org.apache.avro.test")
protocol Simple {
  fixed MD5(16);
  record TestRecord {
    @foo("bar") MD5 hash = "0000000000000000";
  }
}
```

The `@foo("bar")` annotation on line 29 is applied to `MD5`, which
is a reference to the already-defined fixed type -- not a type
definition. This should be rejected.
