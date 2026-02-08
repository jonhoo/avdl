# Type references use simple name even when it collides with Avro Schema.Type names

## Symptom

When a named type has a name that collides with an Avro `Schema.Type`
name (e.g., `record`, `enum`, `fixed`, `array`, `map`, `union`), the
Rust tool shortens type references to just the simple name even though
it is ambiguous. Java always uses the fully-qualified name for such
types to avoid ambiguity with the built-in type names.

For example, given:

```avdl
@namespace("test.bt")
protocol P {
  record `record` { string x; }
  record Ref { `record` r; }
}
```

Rust outputs the reference as:
```json
"type": "record"
```

Java outputs:
```json
"type": "test.bt.record"
```

The bare `"record"` string is ambiguous in Avro JSON: a parser would
interpret it as the Avro `record` complex type (expecting `"fields"`,
etc.) rather than as a reference to the named type `test.bt.record`.

## Affected names

The issue affects any type whose simple name matches a `Schema.Type`
value (case-sensitive, lowercase):

- `record`, `enum`, `array`, `map`, `union`, `fixed`

The primitive type names (`string`, `bytes`, `int`, `long`, `float`,
`double`, `boolean`, `null`) are already blocked by `INVALID_TYPE_NAMES`
in `reader.rs`, so they cannot trigger this bug.

The name `error` is NOT in Java's `Schema.Type` enum, so it is
correctly shortened by both tools.

## Root cause

The `schema_ref_name` function in `json.rs` only checks whether the
namespace matches the enclosing namespace — if it does, it returns just
the simple name. It does not check whether the simple name collides with
a built-in Avro type name.

Java's `Name.shouldWriteFull()` in `Schema.java` (lines 785–798)
includes an additional check:

```java
private boolean shouldWriteFull(String defaultSpace) {
    if (space != null && space.equals(defaultSpace)) {
        for (Type schemaType : Type.values()) {
            if (schemaType.name.equals(name)) {
                return true; // name is a Type, so full name required
            }
        }
        return false;
    }
    return true;
}
```

## Affected files

- `src/model/json.rs` — `schema_ref_name` function (line ~625)

## Reproduction

```sh
cat > tmp/test-keyword-record.avdl <<'EOF'
@namespace("test.kw")
protocol P {
  record `record` { string x; }
  record Ref { `record` r; }
}
EOF
scripts/compare-adhoc.sh tmp/test-keyword-record.avdl

cat > tmp/test-keyword-enum.avdl <<'EOF'
@namespace("test.kw")
protocol P {
  record `enum` { string x; }
  record Ref { `enum` e; }
}
EOF
scripts/compare-adhoc.sh tmp/test-keyword-enum.avdl
```

Both show a diff: Rust uses `"record"` / `"enum"`, Java uses
`"test.kw.record"` / `"test.kw.enum"`.

## Suggested fix

Add a constant list of Avro Schema.Type names to `json.rs`:

```rust
const SCHEMA_TYPE_NAMES: &[&str] = &[
    "record", "enum", "array", "map", "union", "fixed",
    "string", "bytes", "int", "long", "float", "double",
    "boolean", "null",
];
```

Then in `schema_ref_name`, after checking that namespaces match, also
check whether the simple name is in `SCHEMA_TYPE_NAMES`. If it is,
return the fully-qualified name instead of the simple name:

```rust
fn schema_ref_name(name: &str, namespace: Option<&str>, enclosing_namespace: Option<&str>) -> String {
    if namespace == enclosing_namespace {
        if SCHEMA_TYPE_NAMES.contains(&name) {
            // Name collides with built-in type — must use full name.
            match namespace {
                Some(ns) if !ns.is_empty() => format!("{ns}.{name}"),
                _ => name.to_string(),
            }
        } else {
            name.to_string()
        }
    } else {
        match namespace {
            Some(ns) if !ns.is_empty() => format!("{ns}.{name}"),
            _ => name.to_string(),
        }
    }
}
```

This fix also affects alias shortening (see issue 7afa667a): when
checking whether to shorten an alias, the same collision check should
apply.
