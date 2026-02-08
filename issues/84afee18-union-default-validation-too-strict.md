# Union default validation rejects non-first-branch defaults

## Symptom

The Rust tool rejects explicit union fields whose default value matches
a non-first branch. For example:

```avdl
union { null, string } x = "hello";
union { null, int } y = 42;
union { null, Color } c = "RED";
union { null, Inner } val = {"name": "test"};
```

All of these produce:

    Invalid default for field `x`: expected union, got string

Java accepts all of them and emits the union with the original branch
order and the non-null default.

## Root cause

`is_valid_default` in `src/model/schema.rs` (line ~362) validates union
defaults against only the **first** branch:

```rust
AvroSchema::Union { types, .. } => {
    if let Some(first) = types.first() {
        is_valid_default(value, first)
    } else {
        false
    }
}
```

Java's `Schema.isValidDefault` (line ~1785 of `Schema.java`) validates
against **any** branch:

```java
case UNION: // union default: any branch
    return schema.getTypes().stream()
        .anyMatch((Schema s) -> isValidValue(s, defaultValue));
```

The Avro specification says "the default for a union must correspond to
the first schema in the union", but Java intentionally relaxes this to
accept any branch. Since we are porting the Java tool, we should match
its behavior.

Note: the `type?` nullable syntax is **not** affected because
`fix_optional_schema` reorders the union branches so the default always
matches the first branch. This bug only affects explicit `union { ... }`
declarations where the user provides a non-null default and `null` is
the first branch.

## Affected files

- `src/model/schema.rs` — `is_valid_default` function, the `Union`
  match arm

## Reproduction

```sh
# Write test file:
cat > tmp/union-default-bug.avdl <<'EOF'
protocol P {
  record R {
    union { null, string } x = "hello";
  }
}
EOF

# Rust rejects it:
cargo run -- idl tmp/union-default-bug.avdl
# Error: Invalid default for field `x`: expected union, got string

# Java accepts it:
java -jar ../avro-tools-1.12.1.jar idl tmp/union-default-bug.avdl
# Produces valid .avpr with "default": "hello"
```

## Suggested fix

Change the `Union` arm of `is_valid_default` to check whether the
default value is valid for **any** branch in the union, not just the
first:

```rust
AvroSchema::Union { types, .. } => {
    types.iter().any(|branch| is_valid_default(value, branch))
}
```

Also add corresponding unit tests and an integration test with an
`.avdl` file exercising the pattern.
