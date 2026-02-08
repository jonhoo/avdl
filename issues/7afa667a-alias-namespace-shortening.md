# Alias names not shortened when namespace matches the owning schema

## Symptom

When a named type (record, enum, fixed) has `@aliases` with
fully-qualified names, the Rust tool always emits the alias exactly as
provided. Java shortens alias names to the simple name when the alias
namespace matches the owning schema's namespace.

For example, given:

```avdl
@namespace("test.aliases")
protocol P {
  @aliases(["test.aliases.OldName"])
  record NewName { string name; }
}
```

Rust outputs:
```json
"aliases": ["test.aliases.OldName"]
```

Java outputs:
```json
"aliases": ["OldName"]
```

Aliases in a *different* namespace are correctly preserved as
fully-qualified by both tools:

```json
"aliases": ["other.ns.DiffNsAlias"]
```

## Root cause

Java's `Schema.NamedSchema.aliasesToJson()` (line 891 of `Schema.java`)
calls `alias.getQualified(name.space)`, which invokes
`Name.shouldWriteFull()`. This method returns the simple name when the
alias namespace matches the schema's own namespace. The Rust tool's
`schema_to_json` function stores aliases as raw strings and emits them
verbatim without any namespace-relative shortening.

The relevant Java logic in `Name.shouldWriteFull()`:

```java
private boolean shouldWriteFull(String defaultSpace) {
    if (space != null && space.equals(defaultSpace)) {
        for (Type schemaType : Type.values()) {
            if (schemaType.name.equals(name)) {
                return true; // collision with built-in type name
            }
        }
        return false; // same namespace, no collision — use simple name
    }
    return true; // different namespace — use full name
}
```

## Affected files

- `src/model/json.rs` — `schema_to_json` for `Record`, `Enum`, `Fixed`
  variants; the aliases are emitted as-is without namespace shortening
- `src/reader.rs` — `walk_schema_properties` parses `@aliases` values
  into raw strings; may need to parse them into `(name, namespace)` pairs

## Reproduction

```sh
cat > tmp/test-alias-shortening.avdl <<'EOF'
@namespace("test.aliases")
protocol P {
  @aliases(["test.aliases.SameNs", "other.DiffNs", "NoNs"])
  record R { string name; }
}
EOF
scripts/compare-adhoc.sh tmp/test-alias-shortening.avdl
```

Expected diff on aliases:
- Java: `["SameNs", "other.DiffNs", "NoNs"]`
- Rust: `["test.aliases.SameNs", "other.DiffNs", "NoNs"]`

## Suggested fix

In `schema_to_json`, when emitting the `"aliases"` array for named
types, apply the same namespace-shortening logic that Java uses:

1. Parse each alias string into `(simple_name, namespace)` by splitting
   at the last `.`.
2. Compare the alias namespace against the schema's own namespace (not
   the enclosing protocol namespace).
3. If they match and the simple name does not collide with a
   `Schema.Type` name (see issue 4194dd45), emit only the simple name.
4. Otherwise emit the full name.

This could reuse the existing `schema_ref_name` function (or a variant
of it) to keep the logic centralized.
