# [IDL] Nested union in record field silently produces empty type array instead of error

- **Component:** java / idl
- **Affects Version:** 1.12.1

## Description

When a union type directly contains another union type, `avro-tools idl`
silently accepts the input and produces invalid protocol JSON where the
field type is an empty array `[]`. The Avro specification explicitly
states: "Unions may not immediately contain other unions." The tool
should reject this input with an error, but instead it exits
successfully and writes corrupt output.

This is silent data corruption: the exit code is 0, no error or warning
is printed, and the resulting `.avpr` file contains `"type": []` — an
empty union that no Avro consumer can interpret. Downstream tools that
process the output will fail with confusing errors unrelated to the
actual cause.

### Minimal reproduction

```avdl
protocol P {
  record R {
    union { null, union { int, string } } x;
  }
}
```

```
$ java -jar avro-tools-1.12.1.jar idl nested-union.avdl output.avpr
$ echo $?
0
$ cat output.avpr
{
  "protocol" : "P",
  "types" : [ {
    "type" : "record",
    "name" : "R",
    "fields" : [ {
      "name" : "x",
      "type" : [ ]
    } ]
  } ],
  "messages" : { }
}
```

### Expected behavior

The tool should reject the input with an error indicating that unions
may not immediately contain other unions, per the Avro specification
([Schema Declaration > Unions](https://avro.apache.org/docs/1.12.0/specification/#unions)):

> Unions may not immediately contain other unions.

### Actual behavior

The tool exits with code 0 and produces a `.avpr` file where the field
`x` has `"type": []` — an empty union. All three declared member types
(`null`, `int`, `string`) are silently dropped.

## Root cause

The ANTLR grammar (`Idl.g4`) allows nested unions syntactically. The
`unionType` rule (line 137) accepts `fullType` members, and `fullType`
(line 123) delegates through `plainType` (line 125) back to
`unionType`. There is no grammar-level or semantic-level check that
prevents a union from appearing inside another union.

In `IdlReader.java`, the listener uses a `typeStack` with sentinel
values to delimit union boundaries. `enterUnionType` (line 850-853)
pushes an empty marker union onto the stack:

```java
typeStack.push(Schema.createUnion());
```

`exitUnionType` (line 856-865) pops types until it finds a schema
whose type is `UNION`, which it treats as the marker:

```java
List<Schema> types = new ArrayList<>();
Schema type;
while ((type = typeStack.pop()).getType() != Schema.Type.UNION) {
    types.add(type);
}
Collections.reverse(types);
typeStack.push(Schema.createUnion(types));
```

When a union is nested inside another union, the inner union's
`exitUnionType` runs first and pushes its result — a
`Schema.createUnion([int, string])` — onto the stack. This result has
type `UNION`. When the outer union's `exitUnionType` then runs, its
`while` loop immediately pops this inner union result, sees that its
type is `UNION`, and treats it as the sentinel marker. The loop
terminates with an empty `types` list, and the outer union is created
with zero members. The `null` type and the outer sentinel are left
orphaned on the stack.

No golden test in the test suite covers nested unions
(`avro/lang/java/idl/src/test/idl/input/` has no such case), which
explains why this went undetected.

## Suggested fix

Two options:

1. **Add a nesting guard in `enterUnionType`.** Before pushing the
   marker, check whether the grandparent parse context is also a
   `UnionTypeContext`. If so, throw an error immediately:

   ```java
   @Override
   public void enterUnionType(UnionTypeContext ctx) {
       // fullType -> plainType -> unionType: grandparent is fullType
       // If that fullType's parent is also a unionType, unions are nested.
       ParserRuleContext fullTypeCtx = ctx.getParent().getParent();
       if (fullTypeCtx != null
               && fullTypeCtx.getParent() instanceof UnionTypeContext) {
           throw error("Unions may not immediately contain other unions",
               ctx.getStart());
       }
       typeStack.push(Schema.createUnion());
   }
   ```

   This rejects the invalid input at parse time before the sentinel
   mechanism has a chance to malfunction.

2. **Replace the sentinel approach with a depth counter.** Instead of
   using a marker union on the type stack, record `typeStack.size()`
   in `enterUnionType` (e.g., on an auxiliary `Deque<Integer>`). In
   `exitUnionType`, pop types down to that recorded depth. This
   eliminates the ambiguity between sentinel unions and result unions.
   Combined with a check that none of the collected members has type
   `UNION`, it would both fix the stack corruption and produce a
   clear error message.
