# Near-identical Record/Enum/Fixed serialization arms in `schema_to_json`

## Symptom

The Record, Enum, and Fixed arms of `schema_to_json` in `json.rs`
(lines 213-392) follow an identical structural pattern totaling roughly
180 lines of duplicated logic. Each arm:

1. Computes the full name
2. Checks `known_names.contains` and returns a bare string reference
3. Inserts the name into `known_names`
4. Builds a JSON object with `"type"`, `"name"`, namespace (identical
   special-case logic), and `"doc"`
5. Inserts properties with `for (k, v) in properties { obj.insert(...) }`
6. Serializes aliases with the same `alias_ref_name` shortening logic

The following blocks are copy-pasted verbatim across all three arms:

- **Namespace emission** (5-line block with identical comment):
  ```rust
  // Emit the namespace key when it differs from the enclosing context.
  // Special case: when there's no enclosing namespace (standalone .avsc),
  // treat an empty-string namespace the same as None -- Java normalizes
  // empty namespace to null, so `writeName()` omits it.
  if namespace.as_deref() != enclosing_namespace
      && let Some(ns) = namespace
      && !(ns.is_empty() && enclosing_namespace.is_none())
  ```
  Record: lines 245-250, Enum: lines 308-317, Fixed: lines 366-375

- **Alias serialization** (6-line block):
  ```rust
  if !aliases.is_empty() {
      let aliases_json: Vec<Value> = aliases
          .iter()
          .map(|a| Value::String(alias_ref_name(a, namespace.as_deref())))
          .collect();
      obj.insert("aliases".to_string(), Value::Array(aliases_json));
  }
  ```
  Record: lines 270-276, Enum: lines 330-336, Fixed: lines 384-390

- **Properties insertion** (3-line block):
  ```rust
  for (k, v) in properties {
      obj.insert(k.clone(), v.clone());
  }
  ```
  Record: line 267-269, Enum: line 327-329, Fixed: line 381-383

## Root cause

Each named type has different type-specific fields (fields for Record,
symbols/default for Enum, size for Fixed), which makes it natural to
write them as separate match arms. The shared boilerplate around them
was not factored out.

## Affected files

- `src/model/json.rs` lines 213-392 (the three `schema_to_json` arms)

## Reproduction

Read the three arms side by side; the structural skeleton is identical.

## Suggested fix

Extract a helper function like:

```rust
fn serialize_named_type_preamble(
    name: &str,
    namespace: &Option<String>,
    doc: &Option<String>,
    aliases: &[String],
    properties: &HashMap<String, Value>,
    type_str: &str,
    known_names: &mut HashSet<String>,
    enclosing_namespace: Option<&str>,
) -> Result<Map<String, Value>, Value>
```

This function handles steps 1-3 (returning `Err(bare_name)` for the
early-return case), then inserts `type`, `name`, namespace, and `doc`.
Each caller adds its type-specific fields, then calls a shared
`finish_named_type` to append properties and aliases.

This would reduce roughly 60 duplicated lines (the shared parts of 180
total) to a single ~25-line helper.
