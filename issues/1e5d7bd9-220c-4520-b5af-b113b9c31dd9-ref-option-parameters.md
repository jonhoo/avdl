# Use `Option<&T>` instead of `&Option<T>` in function signatures

## Symptom

`cargo clippy --all-targets -- -W clippy::pedantic` reports `clippy::ref_option`
warnings across many functions: passing `&Option<String>` where `Option<&String>`
(or `Option<&str>`) would be more idiomatic. This violates the Rust API guideline
that function parameters should prefer borrowed-optional over reference-to-optional.

## Root cause

The `namespace` and `doc` fields are stored as `Option<String>` in the structs,
and the tree-walking functions pass `&Option<String>` through their call chains
rather than converting at the boundary with `.as_deref()` or `.as_ref()`.

## Affected files

- `src/reader.rs`: ~12 functions including `walk_variable`, `walk_enum`,
  `walk_full_type`, `walk_nullable_type`, `walk_array_type`, `walk_map_type`,
  `walk_union_type`, `walk_message`, `walk_record_body`, `fix_optional_schema`,
  `try_promote_logical_type`, `walk_fixed`
- `src/model/json.rs`: `named_type_preamble`, `finish_named_type`

## Reproduction

```sh
cargo clippy --all-targets -- -W clippy::ref_option
```

## Suggested fix

Change function signatures from `&Option<String>` to `Option<&str>` and update
callers to pass `.as_deref()`. For parameters that currently pattern-match on
`&Option<Value>`, change to `Option<&Value>` with `.as_ref()` at call sites.

This is a pure signature refactor with no behavioral change. The main risk is
the breadth of the change since `namespace` is threaded through many functions.
A find-and-replace approach starting from the leaf functions and working upward
would minimize intermediate compilation failures.
