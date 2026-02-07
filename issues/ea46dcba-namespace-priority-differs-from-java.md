# `compute_namespace` priority differs from Java: `@namespace` wins over dots in Rust, dots win in Java

## Symptom

When a named type has BOTH an `@namespace` annotation AND dots in its
identifier, Rust and Java resolve the namespace differently:

```avdl
@namespace("custom")
protocol P {
  @namespace("override") record com.example.Foo {
    int x;
  }
}
```

- **Java**: namespace = `com.example`, name = `Foo` (dots in identifier
  override the `@namespace` annotation)
- **Rust**: namespace = `override`, name = `Foo` (`@namespace` takes
  priority over dots in identifier)

## Root cause

Java's `namespace()` method (IdlReader.java lines 938-948):

```java
private String namespace(String identifier, String namespace) {
    int dotPos = identifier.lastIndexOf('.');
    String ns = dotPos < 0 ? namespace : identifier.substring(0, dotPos);
    // ...
    return ns;
}
```

When the identifier contains dots (`dotPos >= 0`), the dot-derived
namespace is used unconditionally. The `namespace` parameter (from
`@namespace`) is only used as a fallback when there are no dots.

Rust's `compute_namespace` (reader.rs lines 1448-1467):

```rust
fn compute_namespace(identifier: &str, explicit_namespace: &Option<String>) -> Option<String> {
    if let Some(ns) = explicit_namespace {
        return Some(ns.clone());  // @namespace takes priority
    }
    match identifier.rfind('.') { ... }
}
```

Rust checks `explicit_namespace` (`@namespace`) first and returns it
immediately if present. The dot-based namespace extraction only runs as
a fallback.

## Affected files

- `src/reader.rs` -- `compute_namespace` function (line 1448)

## Reproduction

```avdl
@namespace("foo")
protocol P {
  @namespace("bar") record com.example.MyRecord {
    int x;
  }
}
```

Compare Rust vs Java output:
```sh
# Java: {"type": "record", "name": "MyRecord", "namespace": "com.example", ...}
# Rust: {"type": "record", "name": "MyRecord", "namespace": "bar", ...}
```

## Suggested fix

Change `compute_namespace` to check for dots first, matching Java's
priority:

```rust
fn compute_namespace(identifier: &str, explicit_namespace: &Option<String>) -> Option<String> {
    // Java priority: dots in identifier always win over @namespace.
    if let Some(pos) = identifier.rfind('.') {
        let ns = &identifier[..pos];
        return if ns.is_empty() { None } else { Some(ns.to_string()) };
    }
    // Only use @namespace when there are no dots.
    explicit_namespace.clone().filter(|s| !s.is_empty())
}
```

## Priority

Medium. This only triggers when a user combines `@namespace` with a
dotted identifier (e.g., `@namespace("x") record a.b.Foo`), which is
unusual. The standard Avro test suite does not exercise this combination
for named types. However, when it does trigger, the output would have
the wrong namespace, which is a semantic correctness issue.

## Note

This also affects protocol declarations, enum declarations, and fixed
declarations -- anywhere `compute_namespace` is called with both an
explicit `@namespace` and a dotted identifier.
