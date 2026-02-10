# Test section header contradicts actual union default validation behavior

## Symptom

In `src/model/schema.rs`, the test section header at line 913 says:

> Union defaults: must match the first type in the union

But the implementation at lines 367-376 explicitly validates against
*any* branch of the union, not just the first:

```rust
AvroSchema::Union { types, .. } => {
    if types.is_empty() {
        false
    } else {
        types.iter().any(|branch| is_valid_default(value, branch))
    }
}
```

The code comment above this implementation (lines 367-370) correctly
states:

> Java's `Schema.isValidDefault` checks whether the default matches
> *any* branch of the union, not just the first.

The section header directly contradicts the implementation and the
inline comment.

## Root cause

The section header was written to describe the Avro specification rule
(defaults must match the first type), but the implementation follows
Java's relaxed behavior (defaults may match any type). The header was
never updated to reflect the actual semantics.

## Affected files

- `src/model/schema.rs` line 913

## Reproduction

Read the test section header at line 913 and compare with:
1. The `is_valid_default` implementation for `Union` at lines 371-377
2. The inline comment at lines 367-370 explaining Java's behavior
3. The test `union_null_first_accepts_string_from_second_branch` which
   explicitly tests that non-first-branch defaults are accepted

## Suggested fix

Change the section header from:

```rust
// Union defaults: must match the first type in the union
```

to:

```rust
// Union defaults: may match any branch (matching Java's relaxed behavior)
```
