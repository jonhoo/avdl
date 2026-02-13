# Duplicated logical type parsing logic in reader.rs and import.rs

- **Symptom**: Two separate implementations parse logical type annotations
  and construct `AvroSchema::Logical` variants:

  1. `try_promote_logical_type` in `reader.rs` (lines 3125-3228):
     Handles `@logicalType("date")` annotations from IDL parsing. Matches
     on `(logical_name, kind)` tuples and constructs `LogicalType` variants.

  2. `parse_annotated_primitive` in `import.rs` (lines 675-714):
     Handles `"logicalType": "date"` keys from JSON imports. Also matches
     on logical type strings and constructs `LogicalType` variants.

  Both implementations:
  - Recognize the same logical types: `date`, `time-millis`,
    `timestamp-millis`, `local-timestamp-millis`, `uuid`, `decimal`
  - Handle `decimal` precision/scale extraction
  - Fall back to `AnnotatedPrimitive` for unknown logical types
  - Remove recognized keys from properties before wrapping in `Logical`

  However, they differ in:
  - `reader.rs` validates base type compatibility (e.g., `date` requires
    `Int`), while `import.rs` does not explicitly check
  - `reader.rs` handles additional logical types: `time-micros`,
    `timestamp-micros`, `local-timestamp-micros` that `import.rs` lacks
  - The data structures differ: `reader.rs` works with `HashMap<String, Value>`
    while `import.rs` works with `serde_json::Map<String, Value>`

- **Root cause**: The IDL reader (parsing `.avdl`) and JSON importer
  (parsing `.avpr`/`.avsc`) were implemented separately with different
  entry points for logical type handling.

- **Affected files**:
  - `src/reader.rs` (lines 3125-3228)
  - `src/import.rs` (lines 675-714)

- **Reproduction**: Compare the match arms in both functions -- they
  enumerate the same logical types with nearly identical construction.

- **Suggested fix**:
  1. Create a shared helper in `model/schema.rs`:
     ```rust
     /// Try to construct a LogicalType from a type name string and optional
     /// precision/scale values. Returns None for unrecognized types.
     pub(crate) fn parse_logical_type(
         name: &str,
         precision: Option<u32>,
         scale: Option<u32>,
     ) -> Option<LogicalType> {
         match name {
             "date" => Some(LogicalType::Date),
             "time-millis" => Some(LogicalType::TimeMillis),
             "time-micros" => Some(LogicalType::TimeMicros),
             // ... etc
             "decimal" => {
                 let precision = precision?;
                 Some(LogicalType::Decimal { precision, scale: scale.unwrap_or(0) })
             }
             _ => None,
         }
     }
     ```

  2. Update `import.rs::parse_annotated_primitive` to use this helper
     for logical type construction.

  3. Update `reader.rs::try_promote_logical_type` to use this helper,
     keeping the base-type validation logic as a separate step.

  4. Add the missing logical types (`time-micros`, `timestamp-micros`,
     `local-timestamp-micros`) to `import.rs` for completeness.

  This ensures consistent logical type recognition across both code paths
  and makes it easier to add new logical types in the future.
