# Warn when annotations on non-nullable unions are silently dropped

## Symptom

Annotations placed on a non-nullable union type are silently discarded:

```avro
protocol P {
    record R {
        @deprecated("yes") union { null, string } field;
    }
}
```

The `@deprecated("yes")` annotation is dropped without any diagnostic.
Java's `IdlReader` also rejects annotations on unions, but the user
gets no feedback that their annotation had no effect.

## Root cause

In `apply_properties_to_type` (reader.rs:2944), the `other =>` match
arm catches union types and returns them unchanged, ignoring any
non-empty `properties` map. The existing TODO comment notes this:

```rust
// TODO: warn when `properties` is non-empty here — annotations on
// non-nullable unions are silently dropped (Java also rejects them).
other => other,
```

## Affected files

- `src/reader.rs` (line 2944, `apply_properties_to_type` function)

## Reproduction

```sh
cat > tmp/union-anno.avdl <<'EOF'
protocol P {
    record R {
        @deprecated("yes") union { null, string } field;
    }
}
EOF
cargo run -- idl tmp/union-anno.avdl
# Output: no warning, annotation silently dropped
```

## Suggested fix

Add a warning (not an error) when `properties` is non-empty in the
union/other arm of `apply_properties_to_type`. This requires
threading the warning collection through this function (or returning
an optional warning alongside the schema). The warning message should
indicate which annotations were dropped and on which field.

This is low priority since Java also drops these silently (it only
rejects them for type references, not unions), but adding a warning
improves the user experience.
