# Duplicated qualified name splitting logic across modules

- **Symptom**: Three nearly identical implementations split fully-qualified
  Avro names (`namespace.Name`) at the last dot:

  1. `split_qualified_name` in `import.rs` (lines 470-478):
     ```rust
     fn split_qualified_name(raw_name: &str) -> (String, Option<String>) {
         if let Some(pos) = raw_name.rfind('.') {
             (raw_name[pos + 1..].to_string(), Some(raw_name[..pos].to_string()))
         } else {
             (raw_name.to_string(), None)
         }
     }
     ```

  2. `extract_name` + `compute_namespace` in `reader.rs` (lines 2984-3007):
     ```rust
     fn extract_name(identifier: &str) -> String {
         match identifier.rfind('.') {
             Some(pos) => identifier[pos + 1..].to_string(),
             None => identifier.to_string(),
         }
     }
     fn compute_namespace(identifier: &str, explicit_namespace: &Option<String>) -> Option<String> {
         if let Some(pos) = identifier.rfind('.') {
             return Some(identifier[..pos].to_string());
         }
         explicit_namespace.clone()
     }
     ```

  3. `alias_ref_name` in `model/json.rs` (lines 648-658) also uses
     `alias.rfind('.')` to split a name for shortening.

- **Root cause**: These functions were written independently when their
  respective modules were implemented. They all perform the same fundamental
  operation (split a potentially qualified name into simple name + namespace),
  but with slightly different APIs and ownership semantics.

- **Affected files**:
  - `src/import.rs` (lines 470-478)
  - `src/reader.rs` (lines 2984-3007)
  - `src/model/json.rs` (lines 648-658)

- **Reproduction**: Search for `rfind('.')` across the codebase -- all four
  occurrences in these files implement the same splitting logic.

- **Suggested fix**:
  1. Add a unified helper to `model/schema.rs` alongside the existing
     `make_full_name`:
     ```rust
     /// Split a potentially qualified name into (simple_name, namespace).
     /// If the name contains no dots, namespace is `None`.
     pub(crate) fn split_full_name(full_name: &str) -> (&str, Option<&str>) {
         match full_name.rfind('.') {
             Some(pos) => (&full_name[pos + 1..], Some(&full_name[..pos])),
             None => (full_name, None),
         }
     }
     ```
  2. Update `import.rs::split_qualified_name` to call this helper and
     convert to owned strings as needed.
  3. Update `reader.rs::extract_name` and `compute_namespace` to use
     this helper. Note that `compute_namespace` has additional logic
     for falling back to an explicit namespace, so it would call the
     helper and then apply the fallback.
  4. Update `model/json.rs::alias_ref_name` to use this helper.

  This consolidates the string-splitting logic in one place, making the
  semantics clearer and reducing the risk of subtle inconsistencies.
