# Repetitive destructure-extend-reconstruct in `apply_properties_to_schema`

## Symptom

The function `apply_properties_to_schema` in `reader.rs` (lines
2950-3110) has 10+ match arms that all follow the same pattern:
destructure the variant, call `existing.extend(properties)`, and
reconstruct the variant with the updated properties map. The bare
primitive arms (lines 3058-3089) are especially repetitive: 8 near-
identical arms differing only in the `PrimitiveType` variant name.

Total: roughly 160 lines, of which perhaps 100 are structural
boilerplate.

## Root cause

`AvroSchema` is a plain enum without a common accessor for its
`properties` field. Each variant that carries properties has the field
in a different position within its struct, so the only way to mutate
it is to destructure and reconstruct.

## Affected files

- `src/reader.rs` lines 2950-3110 (`apply_properties_to_schema`)

## Reproduction

Read the function -- every arm except Union follows the same
structure.

## Suggested fix

Two complementary approaches:

1. **Collapse bare primitive arms using a helper or a `PrimitiveType`
   constructor.** All 8 bare-primitive arms could be replaced with a
   single arm using a pattern guard or a `PrimitiveType::from_schema()`
   mapping:
   ```rust
   schema if schema.is_bare_primitive() => {
       let kind = schema.to_primitive_type()
           .expect("is_bare_primitive guarantees this");
       try_promote_logical_type(AvroSchema::AnnotatedPrimitive {
           kind,
           properties,
       })
   }
   ```
   This would reduce 8 arms (~32 lines) to ~5 lines.

2. **Add an `AvroSchema::with_merged_properties` method** that handles
   the destructure-extend-reconstruct pattern generically. Each variant
   with a `properties` field would be handled internally, and the method
   would return the updated schema. This would collapse 7 complex arms
   (~70 lines of boilerplate) into a single method call.

The total savings is approximately 90 lines.
