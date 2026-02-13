# INVALID_TYPE_NAMES and SCHEMA_TYPE_NAMES lists partially overlap

- **Symptom**: Two separate constant arrays define sets of reserved/built-in
  Avro type names:

  1. `INVALID_TYPE_NAMES` in `reader.rs` (lines 739-753):
     ```rust
     const INVALID_TYPE_NAMES: &[&str] = &[
         "boolean", "int", "long", "float", "double", "null", "bytes", "string",
         "date", "time_ms", "timestamp_ms", "local_timestamp_ms", "uuid",
         "time_us", "timestamp_us", "local_timestamp_us", "decimal",
     ];
     ```
     Used to reject user-defined types whose names collide with primitives
     or logical types (e.g., `record date { ... }` is illegal).

  2. `SCHEMA_TYPE_NAMES` in `model/json.rs` (lines 39-42):
     ```rust
     const SCHEMA_TYPE_NAMES: &[&str] = &[
         "record", "enum", "array", "map", "union", "fixed",
         "string", "bytes", "int", "long", "float", "double", "boolean", "null",
     ];
     ```
     Used during JSON serialization to determine when a reference name
     must be fully qualified to avoid ambiguity with built-in type keywords.

  The lists overlap on the primitive type names (`boolean`, `int`, `long`,
  `float`, `double`, `null`, `bytes`, `string`) but diverge otherwise:
  - `INVALID_TYPE_NAMES` includes logical type aliases (`date`, `time_ms`, etc.)
  - `SCHEMA_TYPE_NAMES` includes complex type keywords (`record`, `enum`, etc.)

- **Root cause**: The two lists serve different purposes:
  - `INVALID_TYPE_NAMES` prevents IDL from defining types that shadow
    primitive or logical type names
  - `SCHEMA_TYPE_NAMES` ensures JSON references don't collide with
    Java's `Schema.Type` enum values

  They were defined independently based on each context's needs.

- **Affected files**:
  - `src/reader.rs` (lines 739-753)
  - `src/model/json.rs` (lines 39-42)

- **Reproduction**: Read both constant definitions and note the overlap
  and differences.

- **Suggested fix**:
  This is a **borderline case** for unification. The lists serve genuinely
  different purposes:

  1. **Option A (minimal change)**: Document the relationship between
     the two lists with cross-references in comments. Explain why they
     differ and why both are needed.

  2. **Option B (partial unification)**: Extract the shared primitive
     names into a common constant in `model/schema.rs`:
     ```rust
     pub(crate) const PRIMITIVE_TYPE_NAMES: &[&str] = &[
         "null", "boolean", "int", "long", "float", "double", "bytes", "string",
     ];
     ```
     Then compose each list from this base:
     - `INVALID_TYPE_NAMES = PRIMITIVE_TYPE_NAMES + logical types`
     - `SCHEMA_TYPE_NAMES = PRIMITIVE_TYPE_NAMES + complex types`

     This makes the relationship explicit without forcing full unification.

  Given that the lists serve different purposes and both are correct for
  their use cases, Option A (documentation) may be sufficient. Option B
  is worthwhile if the shared subset is frequently modified.
