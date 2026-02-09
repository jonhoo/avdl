# Annotations on messages with named return types not rejected

## Symptom

Rust accepts `@annotation NamedType method(...)` (annotations before a
message whose return type is a named reference like a record, enum, or
fixed), while Java rejects it with "Type references may not be
annotated". This applies to all three named type kinds: record, enum,
and fixed.

Rust correctly rejects annotations on named type references in
**field** positions (via the check in `walk_full_type`), but does not
perform the same check for **message return type** positions.

## Root cause

In the ANTLR grammar, `messageDeclaration` collects
`schemaProperties` on the message level, and
`resultType` does **not** include its own `schemaProperty*` prefix
(unlike `fullType`, which does). This means annotations before the
return type are grammatically part of the message declaration, not the
return type.

However, Java's `exitNullableType` (IdlReader.java lines 776-777)
checks whether the `propertiesStack` is empty or has properties when
it encounters a named type reference. When inside a message, the
message's `enterMessageDeclaration` pushes its properties onto the
stack. If those properties are non-empty (i.e., the message has
annotations), Java conservatively rejects the combination because the
annotations are ambiguous: do they belong to the message or to the
return type?

In Rust, `walk_message` collects the annotations as message-level
properties (line 1562-1563), and `walk_result_type` walks the return
type independently. There is no corresponding check that prevents
annotations from being applied to the message when the return type
is a named reference.

## Affected files

- `src/reader.rs` — `walk_message` (around line 1555) and
  `walk_result_type` (around line 1672)

## Reproduction

```
# Write test file
cat > tmp/test-annot-ref.avdl <<'EOF'
@namespace("test")
protocol T {
  record Foo { string name; }
  @prop("x")
  Foo getFoo(string id);
}
EOF

# Rust accepts (bug):
cargo run -- idl tmp/test-annot-ref.avdl
# → outputs valid JSON with "prop": "x" on message

# Java rejects (correct):
java -jar ../avro-tools-1.12.1.jar idl tmp/test-annot-ref.avdl
# → "Type references may not be annotated, at line 5, column 2"
```

Note: without annotations, `Foo getFoo(string id)` works in both.

## Suggested fix

In `walk_message`, after collecting `props` and walking the return
type, check if the return type is a named type reference (using the
existing `is_type_reference` helper) and `props.properties` is
non-empty. If so, return an error matching Java's: "Type references
may not be annotated".

```rust
let response = walk_result_type(&result_ctx, token_stream, src, namespace)?;

if !props.properties.is_empty() && is_type_reference(&response) {
    return Err(make_diagnostic(
        src,
        &result_ctx,
        "Type references may not be annotated",
    ));
}
```

This mirrors the existing check in `walk_full_type` (around line
1313) and matches the Java behavior in `exitNullableType`.
