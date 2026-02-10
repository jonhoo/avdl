# Java NPE on forward reference in message return type

## Symptom

When a protocol message's return type references a named type that is
defined later in the same file (forward reference), Java avro-tools
crashes with a `NullPointerException` during schema resolution. The
Rust tool handles this correctly.

This only affects bare named type references as return types. Forward
references in parameters, throws clauses, union-wrapped return types,
and array/map-wrapped return types all work fine.

## Reproduction

```avdl
@namespace("test")
protocol P {
  Foo getFoo();
  record Foo { string name; }
}
```

```sh
java -jar avro-tools-1.12.1.jar idl test-fwd-ref-msg.avdl
# Exception in thread "main" java.lang.NullPointerException:
#   Unknown schema: org.apache.avro.compiler.UnresolvedSchema_0
```

The following variations do NOT trigger the bug:

```avdl
# Forward ref in params -- works fine
void doStuff(Foo x);
record Foo { string name; }

# Forward ref in throws -- works fine
void doStuff() throws MyError;
error MyError { string msg; }

# Forward ref in union return -- works fine
union { null, Foo } getFoo();
record Foo { string name; }
```

## Root cause

Java's `IdlFile.ensureSchemasAreResolved()` resolves the protocol's
message schemas through `ParseContext.resolve()`. For bare return type
references, the `UnresolvedSchema` placeholder created during parsing
is not properly resolved when the target type appears later in the
source. The `ParseContext.resolve()` call then throws an NPE with
`"Unknown schema: org.apache.avro.compiler.UnresolvedSchema_0"`.

Union, array, and map wrappers apparently go through a different
resolution path that handles forward references correctly.

## Impact on this project

None. The Rust tool correctly handles forward references in all
message positions (return, params, throws) because it parses all
types before building messages, so forward references are always
resolvable. This is noted for documentation: if a user reports that
a file works in Rust but not Java, this is a known Java limitation.
