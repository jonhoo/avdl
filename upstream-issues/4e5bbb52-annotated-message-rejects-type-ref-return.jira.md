# [IDL] Annotated message declarations rejected when return type is a named type reference

- **Component:** java / idl
- **Affects Version:** 1.12.1

## Description

Message declarations with custom annotations (e.g., `@since("1.0")`)
are incorrectly rejected with "Type references may not be annotated"
when the return type is a named type reference (record, enum, fixed).
The same annotations work fine when the return type is a primitive.

### Minimal reproduction

```avdl
@namespace("test")
protocol AnnotatedMessageBug {
  record Foo { string name; }
  @since("1.0") Foo getFoo();   // ERROR: "Type references may not be annotated"
}
```

```
$ java -jar avro-tools-1.12.1.jar idl annotated-message-bug.avdl
Exception in thread "main" org.apache.avro.SchemaParseException:
  Type references may not be annotated, at line 4, column 16
    at org.apache.avro.idl.IdlReader.error(IdlReader.java:247)
    at org.apache.avro.idl.IdlReader$IdlParserListener.exitNullableType(IdlReader.java:777)
```

### Counterexample: annotations on messages with primitive returns work

The existing test file `simple.avdl` (line 85) uses:

```avdl
@specialProp("test") int add(int arg1, int arg2 = 0);
```

This compiles successfully and produces `"specialProp": "test"` in the
protocol JSON output (`simple.avpr` line 123). This proves that
message-level annotations are intentionally supported — the failure
only occurs when the return type is a named type reference.

### Expected behavior

The annotation `@since("1.0")` should be treated as a message-level
annotation (like `@specialProp("test")` on the `add` message) and
included in the message JSON output:

```json
"getFoo": {
  "since": "1.0",
  "request": [],
  "response": "Foo"
}
```

### Actual behavior

Compilation fails with `Type references may not be annotated` at the
position of the named return type.

## Root cause

In `IdlReader.java`, `exitNullableType` (line 776-778) checks whether
annotations exist on the current top of `propertiesStack`:

```java
if (propertiesStack.isEmpty() || propertiesStack.peek().hasProperties()) {
    throw error("Type references may not be annotated", ...);
}
```

The issue is a scoping difference between two grammar paths:

1. **Field types** (`fullType -> nullableType`): `enterFullType`
   (line 752) pushes a fresh `SchemaProperties` entry onto the stack.
   When `exitNullableType` checks `propertiesStack.peek()`, it sees
   this fresh entry (no properties), so named type references in field
   types pass the check correctly.

2. **Message return types** (`resultType -> plainType -> nullableType`):
   The `resultType` rule has no `fullType` wrapper, so no fresh
   properties entry is pushed. The top of `propertiesStack` is the
   *message*'s properties entry (pushed by `enterMessageDeclaration`).
   If the message has any custom annotations, `hasProperties()` returns
   true, causing the spurious error.

The code comment at line 774 acknowledges this state:

```java
// propertiesStack is empty within resultType->plainType->nullableType
```

But it doesn't account for the case where message-level annotations
are already on the stack.

## Suggested fix

Either:

1. **Push a fresh properties entry for `resultType`:** Add an
   `enterResultType`/`exitResultType` handler that pushes and pops a
   properties entry, mirroring what `fullType` does. This isolates
   type-level annotation checking from message-level annotations.

2. **Scope the check in `exitNullableType`:** Only check for
   type-level annotations when in a `fullType` context, not when in
   a `resultType` context. The parent context can be determined from
   the parse tree.
