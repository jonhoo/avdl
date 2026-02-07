# Java erroneously rejects annotated messages that return named type references

## Symptom

In Java avro-tools 1.12.1, any message that has custom annotations AND
returns a named type reference fails with "Type references may not be
annotated", even when the annotation belongs to the message, not the
return type.

```avdl
@namespace("test")
protocol P {
  record Foo { string name; }
  @since("1.0") Foo getFoo();
}
```

```
Exception: Type references may not be annotated, at line 4, column 16
```

## Root cause

In `IdlReader.java`, `exitNullableType` (line 776-778) checks:

```java
if (propertiesStack.isEmpty() || propertiesStack.peek().hasProperties()) {
    throw error("Type references may not be annotated", ...);
}
```

When processing `resultType -> plainType -> nullableType` (which has no
`fullType` wrapper and thus no properties stack push), the top of the
`propertiesStack` is the *message*'s properties entry (pushed by
`enterMessageDeclaration`). If the message has any custom annotations
(like `@since`), `hasProperties()` returns true, and the check fails.

The bug is that the check conflates message-level annotations with
type-level annotations. It should only check for type-level annotations,
but the `resultType` grammar path doesn't push its own properties entry.

## Workaround

Remove annotations from messages that return named type references, or
place the annotations elsewhere.

## Impact on Rust implementation

The Rust tool correctly handles this case by checking properties at the
`fullType` level only, which is decoupled from the message's annotations.
The Rust behavior is intentionally NOT changed to match this Java bug.
