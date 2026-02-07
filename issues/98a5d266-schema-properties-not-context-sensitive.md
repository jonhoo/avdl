# `walk_schema_properties` always extracts `@namespace`, `@aliases`, `@order` regardless of context

## Symptom

Annotations `@namespace`, `@aliases`, and `@order` are silently consumed
and dropped in contexts where Java would preserve them as custom
properties. For example:

- `@aliases(["foo"]) int myField;` on a field type (`fullType` context):
  Java preserves `aliases` as a custom property on the schema; Rust
  extracts it into `SchemaProperties.aliases` and then ignores it.
- `@order("DESCENDING") record Foo { ... }`: Java preserves `order` as
  a custom property on the record; Rust extracts it and drops it.
- `@namespace("x") void foo()` on a message declaration: Java preserves
  `namespace` as a custom property; Rust extracts and drops it.

## Root cause

Java's `SchemaProperties` class has three boolean flags --
`withNamespace`, `withAliases`, `withOrder` -- that control whether
`@namespace`, `@aliases`, and `@order` are intercepted as special
annotations or treated as generic custom properties. The flags are set
per context:

| Context                    | `withNS` | `withAliases` | `withOrder` |
|----------------------------|----------|---------------|-------------|
| `enterProtocolDeclaration` | true     | false         | false       |
| `enterFixedDeclaration`    | true     | true          | false       |
| `enterEnumDeclaration`     | true     | true          | false       |
| `enterRecordDeclaration`   | true     | true          | false       |
| `enterEnumSymbol`          | false    | false         | false       |
| `enterVariableDeclaration` | false    | true          | true        |
| `enterMessageDeclaration`  | false    | false         | false       |
| `enterFullType`            | false    | false         | false       |

Rust's `walk_schema_properties` (reader.rs lines 253-344) always
intercepts all three annotations, regardless of which context the
function is called from. The extracted values go into
`SchemaProperties.namespace`, `.aliases`, and `.order`, which are then
silently ignored by callers that don't use them.

The concrete data-loss cases are:

1. **`@namespace` in `fullType`, `variableDeclaration`, and
   `messageDeclaration` contexts**: Extracted but unused. Java would
   put it in the `properties` map.

2. **`@aliases` in `fullType`, `protocolDeclaration`, and
   `messageDeclaration` contexts**: Extracted but unused. Java would
   put it in the `properties` map.

3. **`@order` in all contexts except `variableDeclaration`**: Extracted
   but unused. Java would put it in the `properties` map. This
   includes records, enums, fixed types, protocols, messages, and
   fullTypes.

## Affected files

- `src/reader.rs` -- `walk_schema_properties` function (lines 253-344)
  and all callers: `walk_protocol`, `walk_record`, `walk_enum`,
  `walk_fixed`, `walk_full_type`, `walk_variable`, `walk_message`

## Reproduction

Create a test `.avdl` file with annotations in unexpected positions:

```avdl
protocol Test {
  record Foo {
    @order("DESCENDING") int myField;   // works: variable context has withOrder=true
  }
  @order("IGNORE") record Bar {         // Java: "order" becomes a custom property on the record
    int x;                              // Rust: "order" is silently dropped
  }
}
```

Compare Rust vs Java output:
```sh
# Java output for Bar would include: {"type":"record", ..., "order":"IGNORE"}
# Rust output for Bar would NOT include the "order" property
```

Similarly for `@namespace` on a fullType:
```avdl
protocol Test {
  record Baz {
    @namespace("custom") int weirdField;  // Java: "namespace" as custom prop on the int type
  }                                       // Rust: "namespace" silently extracted and dropped
}
```

## Suggested fix

Add context flags to `walk_schema_properties`, similar to Java's
`SchemaProperties` constructor. One approach:

```rust
struct PropertyContext {
    with_namespace: bool,
    with_aliases: bool,
    with_order: bool,
}
```

Pass the appropriate `PropertyContext` from each call site. When a flag
is false, the corresponding annotation name (`"namespace"`, `"aliases"`,
`"order"`) falls through to the `_` arm and gets inserted into
`result.properties` as a custom property.

Alternatively, each call site could pass the set of "special" annotation
names as a slice, and `walk_schema_properties` would only intercept
names in that set.

## Secondary effect: type reference annotation check is too lenient

Java's "Type references may not be annotated" check (exitNullableType
line 776) uses `propertiesStack.peek().hasProperties()`, which tests the
custom properties map. Since `enterFullType` has all `with*` flags
false, Java treats `@namespace("foo")` on a type reference as a custom
property, and the `hasProperties()` check catches it.

In Rust, `walk_full_type` checks `!props.properties.is_empty()` (line
802). Since `walk_schema_properties` extracts `@namespace` into
`props.namespace` rather than `props.properties`, the check does NOT
fire. This means `@namespace("foo") MyRecord` silently passes in Rust
where Java would reject it with "Type references may not be annotated".

The same applies to `@aliases(...)` and `@order(...)` on type
references.

## Priority

Medium. This only affects users who put `@namespace`, `@aliases`, or
`@order` in non-standard positions. The standard Avro test suite does
not exercise this, but it is a data-loss bug: annotations are silently
dropped instead of being preserved as custom properties. The secondary
annotation-check bypass is a correctness issue where Rust is more
lenient than Java.
