// ==============================================================================
// JSON Serialization for Avro Schemas and Protocols
// ==============================================================================
//
// This module serializes our domain model to JSON matching the exact output
// format of the Java Avro tools (`avro-tools idl2schemata` / `idl`). Key rules:
//
// - Named types (record, enum, fixed) are serialized inline on FIRST occurrence,
//   then as bare string names afterward.
// - Primitives serialize as plain strings: "null", "int", etc.
// - Unions serialize as JSON arrays: ["null", "string"].
// - JSON object keys are alphabetically sorted (BTreeMap-backed serde_json::Map).
//
// References (`AvroSchema::Reference`) are resolved against a lookup table so
// they can be inlined at their first use, just as the Java tools do. This is
// critical for test cases like `forward_ref.avdl` where an enum is defined
// after the record that uses it -- the expected JSON inlines the enum inside
// the record's field.

use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value};

use super::protocol::{Message, Protocol};
use super::schema::{AvroSchema, Field, FieldOrder, LogicalType, make_full_name};

/// Names from Java's `Schema.Type` enum. When a named type's simple name
/// collides with one of these, the fully-qualified name must always be used
/// in JSON references and aliases — otherwise a JSON parser would interpret
/// the bare name as the built-in Avro type rather than as a reference to
/// the user-defined named type. This mirrors Java's `Name.shouldWriteFull()`
/// logic in `Schema.java`.
///
/// The primitive names (`string`, `bytes`, etc.) are already blocked from
/// being used as type names by `INVALID_TYPE_NAMES` in `reader.rs`, so in
/// practice only the complex-type names (`record`, `enum`, `array`, `map`,
/// `union`, `fixed`) can trigger this code path. We include all of them for
/// completeness and to stay aligned with the Java implementation.
const SCHEMA_TYPE_NAMES: &[&str] = &[
    "record", "enum", "array", "map", "union", "fixed", "string", "bytes", "int", "long", "float",
    "double", "boolean", "null",
];

/// A lookup table from full type name to the actual schema definition. This
/// allows `Reference` nodes to be resolved and inlined at their first use.
pub type SchemaLookup = HashMap<String, AvroSchema>;

/// Serialize a `Protocol` to a `serde_json::Value` matching the Java Avro tools output.
pub fn protocol_to_json(protocol: &Protocol) -> Value {
    // Build a lookup table from all named types in the protocol's type list.
    // This includes nested types inside records/fields that were registered
    // in the schema registry.
    let lookup = build_lookup(&protocol.types, protocol.namespace.as_deref());

    let mut known_names = HashSet::new();
    let mut obj = Map::new();

    obj.insert("protocol".to_string(), Value::String(protocol.name.clone()));
    // Java treats an empty namespace as equivalent to no namespace and omits
    // the key entirely from the JSON output. We match that behavior.
    if let Some(ns) = &protocol.namespace
        && !ns.is_empty()
    {
        obj.insert("namespace".to_string(), Value::String(ns.clone()));
    }
    if let Some(doc) = &protocol.doc {
        obj.insert("doc".to_string(), Value::String(doc.clone()));
    }
    for (k, v) in &protocol.properties {
        obj.insert(k.clone(), v.clone());
    }

    // Serialize each top-level type. If a named type was already inlined via
    // Reference resolution (e.g., a forward reference inside a record field),
    // `schema_to_json` returns a bare string. The Java tools omit such
    // already-inlined types from the top-level array, so we filter them out.
    let types: Vec<Value> = protocol
        .types
        .iter()
        .map(|s| schema_to_json(s, &mut known_names, protocol.namespace.as_deref(), &lookup))
        .filter(|v| !v.is_string())
        .collect();
    obj.insert("types".to_string(), Value::Array(types));

    let mut messages_obj = Map::new();
    for (name, msg) in &protocol.messages {
        messages_obj.insert(
            name.clone(),
            message_to_json(
                msg,
                &mut known_names,
                protocol.namespace.as_deref(),
                &lookup,
            ),
        );
    }
    obj.insert("messages".to_string(), Value::Object(messages_obj));

    Value::Object(obj)
}

/// Build a lookup table of full_name -> AvroSchema for all named types,
/// recursively collecting types nested inside records, unions, arrays, etc.
///
/// The `default_namespace` is used for types that have no explicit namespace
/// (they inherit the protocol's namespace). This ensures the lookup key matches
/// the fully-qualified names used in `Reference` nodes.
///
/// This is public so that schema-mode callers (which don't go through
/// `protocol_to_json`) can build a lookup from registry schemas.
pub fn build_lookup(types: &[AvroSchema], default_namespace: Option<&str>) -> SchemaLookup {
    let mut lookup = HashMap::new();
    for schema in types {
        collect_named_types(schema, default_namespace, &mut lookup);
    }
    lookup
}

/// Recursively collect named types from a schema tree into the lookup.
fn collect_named_types(
    schema: &AvroSchema,
    default_namespace: Option<&str>,
    lookup: &mut SchemaLookup,
) {
    match schema {
        AvroSchema::Record {
            name,
            namespace,
            fields,
            ..
        } => {
            let effective_ns = namespace.as_deref().or(default_namespace);
            let full_name = make_full_name(name, effective_ns).into_owned();
            lookup.insert(full_name, schema.clone());
            // Nested types inside a record's fields inherit the record's
            // effective namespace (not the protocol-level default), per the
            // Avro specification.
            for field in fields {
                collect_named_types(&field.schema, effective_ns, lookup);
            }
        }
        AvroSchema::Enum {
            name, namespace, ..
        }
        | AvroSchema::Fixed {
            name, namespace, ..
        } => {
            let effective_ns = namespace.as_deref().or(default_namespace);
            let full_name = make_full_name(name, effective_ns).into_owned();
            lookup.insert(full_name, schema.clone());
        }
        AvroSchema::Array { items, .. } => {
            collect_named_types(items, default_namespace, lookup);
        }
        AvroSchema::Map { values, .. } => {
            collect_named_types(values, default_namespace, lookup);
        }
        AvroSchema::Union { types, .. } => {
            for t in types {
                collect_named_types(t, default_namespace, lookup);
            }
        }
        _ => {}
    }
}

// =============================================================================
// Helpers: shared preamble/postamble for named types (Record, Enum, Fixed)
// =============================================================================
//
// Named types share identical boilerplate for:
// 1. Computing the full name, checking/inserting `known_names`, and returning
//    a bare string on second occurrence (preamble).
// 2. Inserting properties and aliases into the JSON object (postamble).
//
// These helpers factor out that duplication so each match arm only contains
// its type-specific fields.

/// Handle the common preamble for a named type: compute its full name, check
/// whether it has already been serialized (returning a bare-name `Value` if so),
/// and build the initial JSON object with `"type"`, `"name"`, optional
/// `"namespace"`, and optional `"doc"` keys.
///
/// Returns `Err(bare_name_value)` when the type was already serialized and the
/// caller should return early. Returns `Ok(obj)` with the partially-built JSON
/// object when the type is being serialized for the first time.
fn named_type_preamble(
    type_str: &str,
    name: &str,
    namespace: &Option<String>,
    doc: &Option<String>,
    known_names: &mut HashSet<String>,
    enclosing_namespace: Option<&str>,
) -> Result<Map<String, Value>, Value> {
    let full_name = make_full_name(name, namespace.as_deref()).into_owned();
    if known_names.contains(&full_name) {
        return Err(Value::String(schema_ref_name(
            name,
            namespace.as_deref(),
            enclosing_namespace,
        )));
    }
    known_names.insert(full_name);

    let mut obj = Map::new();
    obj.insert("type".to_string(), Value::String(type_str.to_string()));
    obj.insert("name".to_string(), Value::String(name.to_string()));
    // Emit the namespace key when it differs from the enclosing context.
    // Special case: when there's no enclosing namespace (standalone .avsc),
    // treat an empty-string namespace the same as None — Java normalizes
    // empty namespace to null, so `writeName()` omits it.
    if namespace.as_deref() != enclosing_namespace
        && let Some(ns) = namespace
        && !(ns.is_empty() && enclosing_namespace.is_none())
    {
        obj.insert("namespace".to_string(), Value::String(ns.clone()));
    }
    if let Some(doc) = doc {
        obj.insert("doc".to_string(), Value::String(doc.clone()));
    }
    Ok(obj)
}

/// Append the common trailing fields for a named type: custom properties and
/// aliases. Called after the caller has inserted all type-specific keys.
fn finish_named_type(
    obj: &mut Map<String, Value>,
    properties: &HashMap<String, Value>,
    aliases: &[String],
    namespace: &Option<String>,
) {
    // Java emits properties before aliases for named types.
    for (k, v) in properties {
        obj.insert(k.clone(), v.clone());
    }
    if !aliases.is_empty() {
        let aliases_json: Vec<Value> = aliases
            .iter()
            .map(|a| Value::String(alias_ref_name(a, namespace.as_deref())))
            .collect();
        obj.insert("aliases".to_string(), Value::Array(aliases_json));
    }
}

/// Serialize an `AvroSchema` to JSON. For named types, the first occurrence
/// is serialized inline; subsequent occurrences are bare name strings.
///
/// The `lookup` parameter allows `Reference` nodes to be resolved and inlined
/// at their first use.
pub fn schema_to_json(
    schema: &AvroSchema,
    known_names: &mut HashSet<String>,
    enclosing_namespace: Option<&str>,
    lookup: &SchemaLookup,
) -> Value {
    // Primitives: serialize as plain strings.
    if let Some(name) = schema.primitive_type_name() {
        return Value::String(name.to_string());
    }

    match schema {
        // =====================================================================
        // Annotated primitive: a primitive with custom properties, serialized
        // as {"type": "int", ...properties} instead of bare "int".
        // =====================================================================
        AvroSchema::AnnotatedPrimitive { kind, properties } => {
            let mut obj = Map::new();
            obj.insert("type".to_string(), Value::String(kind.as_str().to_string()));
            for (k, v) in properties {
                obj.insert(k.clone(), v.clone());
            }
            Value::Object(obj)
        }

        // =====================================================================
        // Record
        // =====================================================================
        AvroSchema::Record {
            name,
            namespace,
            doc,
            fields,
            is_error,
            aliases,
            properties,
        } => {
            let type_str = if *is_error { "error" } else { "record" };
            let mut obj = match named_type_preamble(
                type_str,
                name,
                namespace,
                doc,
                known_names,
                enclosing_namespace,
            ) {
                Ok(obj) => obj,
                Err(bare_name) => return bare_name,
            };
            let fields_json: Vec<Value> = fields
                .iter()
                .map(|f| {
                    field_to_json(
                        f,
                        known_names,
                        namespace.as_deref().or(enclosing_namespace),
                        lookup,
                    )
                })
                .collect();
            obj.insert("fields".to_string(), Value::Array(fields_json));
            finish_named_type(&mut obj, properties, aliases, namespace);
            Value::Object(obj)
        }

        // =====================================================================
        // Enum
        // =====================================================================
        AvroSchema::Enum {
            name,
            namespace,
            doc,
            symbols,
            default,
            aliases,
            properties,
        } => {
            let mut obj = match named_type_preamble(
                "enum",
                name,
                namespace,
                doc,
                known_names,
                enclosing_namespace,
            ) {
                Ok(obj) => obj,
                Err(bare_name) => return bare_name,
            };
            let symbols_json: Vec<Value> =
                symbols.iter().map(|s| Value::String(s.clone())).collect();
            obj.insert("symbols".to_string(), Value::Array(symbols_json));
            if let Some(def) = default {
                obj.insert("default".to_string(), Value::String(def.clone()));
            }
            finish_named_type(&mut obj, properties, aliases, namespace);
            Value::Object(obj)
        }

        // =====================================================================
        // Fixed
        // =====================================================================
        AvroSchema::Fixed {
            name,
            namespace,
            doc,
            size,
            aliases,
            properties,
        } => {
            let mut obj = match named_type_preamble(
                "fixed",
                name,
                namespace,
                doc,
                known_names,
                enclosing_namespace,
            ) {
                Ok(obj) => obj,
                Err(bare_name) => return bare_name,
            };
            obj.insert("size".to_string(), Value::Number((*size).into()));
            finish_named_type(&mut obj, properties, aliases, namespace);
            Value::Object(obj)
        }

        // =====================================================================
        // Array: {"type": "array", "items": ..., ...properties}
        // =====================================================================
        AvroSchema::Array { items, properties } => {
            let mut obj = Map::new();
            obj.insert("type".to_string(), Value::String("array".to_string()));
            obj.insert(
                "items".to_string(),
                schema_to_json(items, known_names, enclosing_namespace, lookup),
            );
            for (k, v) in properties {
                obj.insert(k.clone(), v.clone());
            }
            Value::Object(obj)
        }

        // =====================================================================
        // Map: {"type": "map", "values": ..., ...properties}
        // =====================================================================
        AvroSchema::Map { values, properties } => {
            let mut obj = Map::new();
            obj.insert("type".to_string(), Value::String("map".to_string()));
            obj.insert(
                "values".to_string(),
                schema_to_json(values, known_names, enclosing_namespace, lookup),
            );
            for (k, v) in properties {
                obj.insert(k.clone(), v.clone());
            }
            Value::Object(obj)
        }

        // =====================================================================
        // Union: serialize as a JSON array of the constituent types.
        // The `is_nullable_type` flag is internal only and not serialized.
        // =====================================================================
        AvroSchema::Union { types, .. } => {
            let types_json: Vec<Value> = types
                .iter()
                .map(|t| schema_to_json(t, known_names, enclosing_namespace, lookup))
                .collect();
            Value::Array(types_json)
        }

        // =====================================================================
        // Logical types: serialize as the base type with a `logicalType` key.
        // =====================================================================
        AvroSchema::Logical {
            logical_type,
            properties,
        } => {
            let mut obj = Map::new();
            match logical_type {
                LogicalType::Date => {
                    obj.insert("type".to_string(), Value::String("int".to_string()));
                    obj.insert("logicalType".to_string(), Value::String("date".to_string()));
                }
                LogicalType::TimeMillis => {
                    obj.insert("type".to_string(), Value::String("int".to_string()));
                    obj.insert(
                        "logicalType".to_string(),
                        Value::String("time-millis".to_string()),
                    );
                }
                LogicalType::TimeMicros => {
                    obj.insert("type".to_string(), Value::String("long".to_string()));
                    obj.insert(
                        "logicalType".to_string(),
                        Value::String("time-micros".to_string()),
                    );
                }
                LogicalType::TimestampMillis => {
                    obj.insert("type".to_string(), Value::String("long".to_string()));
                    obj.insert(
                        "logicalType".to_string(),
                        Value::String("timestamp-millis".to_string()),
                    );
                }
                LogicalType::TimestampMicros => {
                    obj.insert("type".to_string(), Value::String("long".to_string()));
                    obj.insert(
                        "logicalType".to_string(),
                        Value::String("timestamp-micros".to_string()),
                    );
                }
                LogicalType::LocalTimestampMillis => {
                    obj.insert("type".to_string(), Value::String("long".to_string()));
                    obj.insert(
                        "logicalType".to_string(),
                        Value::String("local-timestamp-millis".to_string()),
                    );
                }
                LogicalType::LocalTimestampMicros => {
                    obj.insert("type".to_string(), Value::String("long".to_string()));
                    obj.insert(
                        "logicalType".to_string(),
                        Value::String("local-timestamp-micros".to_string()),
                    );
                }
                LogicalType::Uuid => {
                    obj.insert("type".to_string(), Value::String("string".to_string()));
                    obj.insert("logicalType".to_string(), Value::String("uuid".to_string()));
                }
                LogicalType::Decimal { precision, scale } => {
                    obj.insert("type".to_string(), Value::String("bytes".to_string()));
                    obj.insert(
                        "logicalType".to_string(),
                        Value::String("decimal".to_string()),
                    );
                    obj.insert("precision".to_string(), Value::Number((*precision).into()));
                    obj.insert("scale".to_string(), Value::Number((*scale).into()));
                }
            }
            for (k, v) in properties {
                obj.insert(k.clone(), v.clone());
            }
            Value::Object(obj)
        }

        // =====================================================================
        // Reference: resolve against the lookup to inline at first use, or
        // output a bare name for subsequent uses.
        // =====================================================================
        AvroSchema::Reference {
            name, namespace, ..
        } => {
            let full_name = make_full_name(name, namespace.as_deref()).into_owned();

            // If already serialized, output a bare name (possibly shortened).
            if known_names.contains(&full_name) {
                return Value::String(schema_ref_name(
                    name,
                    namespace.as_deref(),
                    enclosing_namespace,
                ));
            }

            // Try to resolve from the lookup and inline the full definition.
            if let Some(resolved) = lookup.get(&full_name) {
                return schema_to_json(resolved, known_names, enclosing_namespace, lookup);
            }

            // Unresolvable reference -- output as a bare name string, applying
            // namespace shortening when possible.
            Value::String(schema_ref_name(
                name,
                namespace.as_deref(),
                enclosing_namespace,
            ))
        }

        // Primitives are handled above by `primitive_type_name()`.
        _ => unreachable!("all AvroSchema variants are covered"),
    }
}

// =============================================================================
// Helper: serialize a record field to JSON.
// =============================================================================

fn field_to_json(
    field: &Field,
    known_names: &mut HashSet<String>,
    enclosing_namespace: Option<&str>,
    lookup: &SchemaLookup,
) -> Value {
    let mut obj = Map::new();
    obj.insert("name".to_string(), Value::String(field.name.clone()));
    obj.insert(
        "type".to_string(),
        schema_to_json(&field.schema, known_names, enclosing_namespace, lookup),
    );
    if let Some(doc) = &field.doc {
        obj.insert("doc".to_string(), Value::String(doc.clone()));
    }
    if let Some(default) = &field.default {
        obj.insert("default".to_string(), default.clone());
    }
    // Ascending is the default -- omit it.
    match &field.order {
        Some(FieldOrder::Descending) => {
            obj.insert("order".to_string(), Value::String("descending".to_string()));
        }
        Some(FieldOrder::Ignore) => {
            obj.insert("order".to_string(), Value::String("ignore".to_string()));
        }
        Some(FieldOrder::Ascending) | None => {}
    }
    if !field.aliases.is_empty() {
        let aliases_json: Vec<Value> = field
            .aliases
            .iter()
            .map(|a| Value::String(a.clone()))
            .collect();
        obj.insert("aliases".to_string(), Value::Array(aliases_json));
    }
    for (k, v) in &field.properties {
        obj.insert(k.clone(), v.clone());
    }
    Value::Object(obj)
}

// =============================================================================
// Helper: serialize a protocol message to JSON.
// =============================================================================

fn message_to_json(
    msg: &Message,
    known_names: &mut HashSet<String>,
    enclosing_namespace: Option<&str>,
    lookup: &SchemaLookup,
) -> Value {
    let mut obj = Map::new();
    if let Some(doc) = &msg.doc {
        obj.insert("doc".to_string(), Value::String(doc.clone()));
    }
    for (k, v) in &msg.properties {
        obj.insert(k.clone(), v.clone());
    }
    let request: Vec<Value> = msg
        .request
        .iter()
        .map(|f| field_to_json(f, known_names, enclosing_namespace, lookup))
        .collect();
    obj.insert("request".to_string(), Value::Array(request));
    obj.insert(
        "response".to_string(),
        schema_to_json(&msg.response, known_names, enclosing_namespace, lookup),
    );
    if let Some(errors) = &msg.errors {
        let errors_json: Vec<Value> = errors
            .iter()
            .map(|e| schema_to_json(e, known_names, enclosing_namespace, lookup))
            .collect();
        obj.insert("errors".to_string(), Value::Array(errors_json));
    }
    if msg.one_way {
        obj.insert("one-way".to_string(), Value::Bool(true));
    }
    Value::Object(obj)
}

/// When referencing a named type, use just the simple name if it shares the same
/// namespace as the enclosing context; otherwise use the fully qualified name.
///
/// If the simple name collides with an Avro `Schema.Type` name (e.g., `record`,
/// `enum`), the fully-qualified name is always used even when namespaces match.
/// This mirrors Java's `Name.shouldWriteFull()` logic, which prevents ambiguity
/// between a user-defined type reference and a built-in Avro type keyword.
fn schema_ref_name(
    name: &str,
    namespace: Option<&str>,
    enclosing_namespace: Option<&str>,
) -> String {
    if namespace == enclosing_namespace {
        if SCHEMA_TYPE_NAMES.contains(&name) {
            // Name collides with a built-in type -- must use the full name
            // to avoid ambiguity in the JSON output.
            make_full_name(name, namespace).into_owned()
        } else {
            name.to_string()
        }
    } else {
        make_full_name(name, namespace).into_owned()
    }
}

/// Shorten an alias name using the same logic as Java's `Name.shouldWriteFull()`.
///
/// Each alias is a potentially fully-qualified name (e.g., `"com.example.OldName"`).
/// If the alias namespace matches the owning schema's namespace and the simple
/// name does not collide with a `Schema.Type` name, the alias is shortened to
/// just the simple name. Otherwise the full name is preserved.
fn alias_ref_name(alias: &str, schema_namespace: Option<&str>) -> String {
    // Split at the last '.' to separate namespace from simple name.
    match alias.rfind('.') {
        Some(pos) => {
            let alias_ns = &alias[..pos];
            let alias_simple = &alias[pos + 1..];
            schema_ref_name(alias_simple, Some(alias_ns), schema_namespace)
        }
        // No dot -- the alias has no namespace; emit it as-is.
        None => alias.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    // =========================================================================
    // Helpers
    // =========================================================================

    /// Serialize a schema with no prior known names, no enclosing namespace,
    /// and an empty lookup table. Suitable for testing standalone schemas.
    fn serialize_schema(schema: &AvroSchema) -> Value {
        schema_to_json(schema, &mut HashSet::new(), None, &HashMap::new())
    }

    /// Serialize a schema with the given known names and lookup, returning the
    /// updated known_names set for subsequent assertions.
    fn serialize_schema_tracking(
        schema: &AvroSchema,
        known_names: &mut HashSet<String>,
        enclosing_ns: Option<&str>,
        lookup: &SchemaLookup,
    ) -> Value {
        schema_to_json(schema, known_names, enclosing_ns, lookup)
    }

    // =========================================================================
    // Primitive types
    // =========================================================================

    #[test]
    fn primitives_serialize_as_bare_strings() {
        assert_eq!(serialize_schema(&AvroSchema::Null), json!("null"));
        assert_eq!(serialize_schema(&AvroSchema::Boolean), json!("boolean"));
        assert_eq!(serialize_schema(&AvroSchema::Int), json!("int"));
        assert_eq!(serialize_schema(&AvroSchema::Long), json!("long"));
        assert_eq!(serialize_schema(&AvroSchema::Float), json!("float"));
        assert_eq!(serialize_schema(&AvroSchema::Double), json!("double"));
        assert_eq!(serialize_schema(&AvroSchema::Bytes), json!("bytes"));
        assert_eq!(serialize_schema(&AvroSchema::String), json!("string"));
    }

    // =========================================================================
    // Annotated primitives
    // =========================================================================

    #[test]
    fn annotated_primitive_serializes_as_object() {
        let mut props = HashMap::new();
        props.insert("foo.bar".to_string(), json!("baz"));

        let schema = AvroSchema::AnnotatedPrimitive {
            kind: super::super::schema::PrimitiveType::Long,
            properties: props,
        };

        let result = serialize_schema(&schema);
        assert_eq!(result, json!({"type": "long", "foo.bar": "baz"}));
    }

    // =========================================================================
    // Record
    // =========================================================================

    #[test]
    fn record_serializes_correctly() {
        let schema = AvroSchema::Record {
            name: "Person".to_string(),
            namespace: Some("com.example".to_string()),
            doc: Some("A person record.".to_string()),
            fields: vec![
                Field {
                    name: "name".to_string(),
                    schema: AvroSchema::String,
                    doc: None,
                    default: None,
                    order: None,
                    aliases: vec![],
                    properties: HashMap::new(),
                },
                Field {
                    name: "age".to_string(),
                    schema: AvroSchema::Int,
                    doc: None,
                    default: Some(json!(0)),
                    order: None,
                    aliases: vec![],
                    properties: HashMap::new(),
                },
            ],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        };

        let result = serialize_schema(&schema);
        assert_eq!(
            result,
            json!({
                "type": "record",
                "name": "Person",
                "namespace": "com.example",
                "doc": "A person record.",
                "fields": [
                    {"name": "name", "type": "string"},
                    {"name": "age", "type": "int", "default": 0}
                ]
            })
        );
    }

    #[test]
    fn error_record_uses_error_type() {
        let schema = AvroSchema::Record {
            name: "TestError".to_string(),
            namespace: None,
            doc: None,
            fields: vec![Field {
                name: "message".to_string(),
                schema: AvroSchema::String,
                doc: None,
                default: None,
                order: None,
                aliases: vec![],
                properties: HashMap::new(),
            }],
            is_error: true,
            aliases: vec![],
            properties: HashMap::new(),
        };

        let result = serialize_schema(&schema);
        assert_eq!(result["type"], json!("error"));
        assert_eq!(result["name"], json!("TestError"));
    }

    #[test]
    fn record_with_aliases_and_properties() {
        let mut props = HashMap::new();
        props.insert("my-prop".to_string(), json!({"key": 42}));

        let schema = AvroSchema::Record {
            name: "Rec".to_string(),
            namespace: None,
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec!["OldRec".to_string()],
            properties: props,
        };

        let result = serialize_schema(&schema);
        assert_eq!(result["aliases"], json!(["OldRec"]));
        assert_eq!(result["my-prop"], json!({"key": 42}));
    }

    #[test]
    fn record_omits_namespace_when_same_as_enclosing() {
        // When the record's namespace matches the enclosing protocol namespace,
        // the "namespace" key should be omitted from the JSON output.
        let schema = AvroSchema::Record {
            name: "Rec".to_string(),
            namespace: Some("org.example".to_string()),
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        };

        let result = schema_to_json(
            &schema,
            &mut HashSet::new(),
            Some("org.example"),
            &HashMap::new(),
        );
        assert!(result.get("namespace").is_none());
    }

    #[test]
    fn record_includes_namespace_when_different_from_enclosing() {
        let schema = AvroSchema::Record {
            name: "Rec".to_string(),
            namespace: Some("org.other".to_string()),
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        };

        let result = schema_to_json(
            &schema,
            &mut HashSet::new(),
            Some("org.example"),
            &HashMap::new(),
        );
        assert_eq!(result["namespace"], json!("org.other"));
    }

    // =========================================================================
    // Enum
    // =========================================================================

    #[test]
    fn enum_serializes_correctly() {
        let schema = AvroSchema::Enum {
            name: "Status".to_string(),
            namespace: Some("org.test".to_string()),
            doc: Some("Status enum.".to_string()),
            symbols: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            default: Some("C".to_string()),
            aliases: vec![],
            properties: HashMap::new(),
        };

        let result = serialize_schema(&schema);
        assert_eq!(
            result,
            json!({
                "type": "enum",
                "name": "Status",
                "namespace": "org.test",
                "doc": "Status enum.",
                "symbols": ["A", "B", "C"],
                "default": "C"
            })
        );
    }

    #[test]
    fn enum_omits_default_when_absent() {
        let schema = AvroSchema::Enum {
            name: "Kind".to_string(),
            namespace: None,
            doc: None,
            symbols: vec!["FOO".to_string(), "BAR".to_string()],
            default: None,
            aliases: vec![],
            properties: HashMap::new(),
        };

        let result = serialize_schema(&schema);
        assert!(result.get("default").is_none());
    }

    // =========================================================================
    // Fixed
    // =========================================================================

    #[test]
    fn fixed_serializes_correctly() {
        let schema = AvroSchema::Fixed {
            name: "MD5".to_string(),
            namespace: None,
            doc: Some("An MD5 hash.".to_string()),
            size: 16,
            aliases: vec![],
            properties: HashMap::new(),
        };

        let result = serialize_schema(&schema);
        assert_eq!(
            result,
            json!({
                "type": "fixed",
                "name": "MD5",
                "doc": "An MD5 hash.",
                "size": 16
            })
        );
    }

    // =========================================================================
    // Array
    // =========================================================================

    #[test]
    fn array_serializes_correctly() {
        let schema = AvroSchema::Array {
            items: Box::new(AvroSchema::String),
            properties: HashMap::new(),
        };

        let result = serialize_schema(&schema);
        assert_eq!(result, json!({"type": "array", "items": "string"}));
    }

    #[test]
    fn array_with_properties() {
        let mut props = HashMap::new();
        props.insert("foo.bar".to_string(), json!("baz"));

        let schema = AvroSchema::Array {
            items: Box::new(AvroSchema::Int),
            properties: props,
        };

        let result = serialize_schema(&schema);
        assert_eq!(
            result,
            json!({"type": "array", "items": "int", "foo.bar": "baz"})
        );
    }

    // =========================================================================
    // Map
    // =========================================================================

    #[test]
    fn map_serializes_correctly() {
        let schema = AvroSchema::Map {
            values: Box::new(AvroSchema::Int),
            properties: HashMap::new(),
        };

        let result = serialize_schema(&schema);
        assert_eq!(result, json!({"type": "map", "values": "int"}));
    }

    // =========================================================================
    // Union
    // =========================================================================

    #[test]
    fn union_serializes_as_array() {
        let schema = AvroSchema::Union {
            types: vec![AvroSchema::Null, AvroSchema::String],
            is_nullable_type: true,
        };

        let result = serialize_schema(&schema);
        assert_eq!(result, json!(["null", "string"]));
    }

    // =========================================================================
    // Logical types
    // =========================================================================

    #[test]
    fn logical_type_date() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::Date,
            properties: HashMap::new(),
        };
        assert_eq!(
            serialize_schema(&schema),
            json!({"type": "int", "logicalType": "date"})
        );
    }

    #[test]
    fn logical_type_time_millis() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::TimeMillis,
            properties: HashMap::new(),
        };
        assert_eq!(
            serialize_schema(&schema),
            json!({"type": "int", "logicalType": "time-millis"})
        );
    }

    #[test]
    fn logical_type_timestamp_millis() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::TimestampMillis,
            properties: HashMap::new(),
        };
        assert_eq!(
            serialize_schema(&schema),
            json!({"type": "long", "logicalType": "timestamp-millis"})
        );
    }

    #[test]
    fn logical_type_local_timestamp_millis() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::LocalTimestampMillis,
            properties: HashMap::new(),
        };
        assert_eq!(
            serialize_schema(&schema),
            json!({"type": "long", "logicalType": "local-timestamp-millis"})
        );
    }

    #[test]
    fn logical_type_uuid() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::Uuid,
            properties: HashMap::new(),
        };
        assert_eq!(
            serialize_schema(&schema),
            json!({"type": "string", "logicalType": "uuid"})
        );
    }

    #[test]
    fn logical_type_decimal() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::Decimal {
                precision: 6,
                scale: 2,
            },
            properties: HashMap::new(),
        };
        assert_eq!(
            serialize_schema(&schema),
            json!({"type": "bytes", "logicalType": "decimal", "precision": 6, "scale": 2})
        );
    }

    // =========================================================================
    // Reference inlining behavior
    // =========================================================================

    #[test]
    fn reference_inlines_full_definition_on_first_use() {
        // Build a lookup with a record definition. A Reference pointing to it
        // should inline the full record JSON on first encounter.
        let record = AvroSchema::Record {
            name: "Ping".to_string(),
            namespace: Some("org.example".to_string()),
            doc: None,
            fields: vec![Field {
                name: "ts".to_string(),
                schema: AvroSchema::Long,
                doc: None,
                default: None,
                order: None,
                aliases: vec![],
                properties: HashMap::new(),
            }],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        };

        let mut lookup = HashMap::new();
        lookup.insert("org.example.Ping".to_string(), record);

        let reference = AvroSchema::Reference {
            name: "Ping".to_string(),
            namespace: Some("org.example".to_string()),
            properties: HashMap::new(),
            span: None,
        };

        let mut known = HashSet::new();
        let result = serialize_schema_tracking(&reference, &mut known, None, &lookup);

        // First use: should be the full record definition (an object).
        assert!(
            result.is_object(),
            "first use of reference should inline the full definition"
        );
        assert_eq!(result["type"], json!("record"));
        assert_eq!(result["name"], json!("Ping"));
    }

    #[test]
    fn reference_emits_bare_name_on_subsequent_use() {
        let record = AvroSchema::Record {
            name: "Ping".to_string(),
            namespace: Some("org.example".to_string()),
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        };

        let mut lookup = HashMap::new();
        lookup.insert("org.example.Ping".to_string(), record);

        let reference = AvroSchema::Reference {
            name: "Ping".to_string(),
            namespace: Some("org.example".to_string()),
            properties: HashMap::new(),
            span: None,
        };

        let mut known = HashSet::new();
        // First use inlines the definition.
        let _ = serialize_schema_tracking(&reference, &mut known, None, &lookup);
        // Second use should be a bare name string.
        let result = serialize_schema_tracking(&reference, &mut known, None, &lookup);
        assert_eq!(result, json!("org.example.Ping"));
    }

    #[test]
    fn reference_uses_short_name_when_namespace_matches_enclosing() {
        let record = AvroSchema::Record {
            name: "Ping".to_string(),
            namespace: Some("org.example".to_string()),
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        };

        let mut lookup = HashMap::new();
        lookup.insert("org.example.Ping".to_string(), record);

        let reference = AvroSchema::Reference {
            name: "Ping".to_string(),
            namespace: Some("org.example".to_string()),
            properties: HashMap::new(),
            span: None,
        };

        let mut known = HashSet::new();
        // First use inlines.
        let _ = serialize_schema_tracking(&reference, &mut known, Some("org.example"), &lookup);
        // Second use within the same namespace should use the short name.
        let result =
            serialize_schema_tracking(&reference, &mut known, Some("org.example"), &lookup);
        assert_eq!(result, json!("Ping"));
    }

    // =========================================================================
    // schema_ref_name
    // =========================================================================

    #[test]
    fn ref_name_returns_simple_when_namespace_matches() {
        assert_eq!(
            schema_ref_name("Foo", Some("org.example"), Some("org.example")),
            "Foo"
        );
    }

    #[test]
    fn ref_name_returns_qualified_when_namespaces_differ() {
        assert_eq!(
            schema_ref_name("Foo", Some("org.other"), Some("org.example")),
            "org.other.Foo"
        );
    }

    #[test]
    fn ref_name_returns_simple_when_no_namespace() {
        assert_eq!(schema_ref_name("Foo", None, None), "Foo");
    }

    #[test]
    fn ref_name_returns_qualified_when_only_type_has_namespace() {
        assert_eq!(
            schema_ref_name("Foo", Some("org.example"), None),
            "org.example.Foo"
        );
    }

    // =========================================================================
    // schema_ref_name: Schema.Type name collision
    // =========================================================================

    #[test]
    fn ref_name_uses_full_name_for_schema_type_collision() {
        // A type named `record` in namespace `test.kw` must use the full name
        // even when the enclosing namespace matches.
        assert_eq!(
            schema_ref_name("record", Some("test.kw"), Some("test.kw")),
            "test.kw.record"
        );
    }

    #[test]
    fn ref_name_uses_full_name_for_enum_collision() {
        assert_eq!(
            schema_ref_name("enum", Some("test.kw"), Some("test.kw")),
            "test.kw.enum"
        );
    }

    #[test]
    fn ref_name_uses_full_name_for_fixed_collision() {
        assert_eq!(
            schema_ref_name("fixed", Some("test.kw"), Some("test.kw")),
            "test.kw.fixed"
        );
    }

    #[test]
    fn ref_name_uses_full_name_for_array_collision() {
        assert_eq!(
            schema_ref_name("array", Some("test.kw"), Some("test.kw")),
            "test.kw.array"
        );
    }

    #[test]
    fn ref_name_uses_full_name_for_map_collision() {
        assert_eq!(
            schema_ref_name("map", Some("test.kw"), Some("test.kw")),
            "test.kw.map"
        );
    }

    #[test]
    fn ref_name_uses_full_name_for_union_collision() {
        assert_eq!(
            schema_ref_name("union", Some("test.kw"), Some("test.kw")),
            "test.kw.union"
        );
    }

    #[test]
    fn ref_name_collision_with_no_namespace_returns_bare_name() {
        // With no namespace on either side, we can only emit the bare name,
        // even though it collides. (This matches Java's behavior: if
        // space == null, shouldWriteFull returns true but getQualified just
        // returns the name portion.)
        assert_eq!(schema_ref_name("record", None, None), "record");
    }

    #[test]
    fn ref_name_collision_different_namespaces_uses_full() {
        // Different namespaces -- always full, regardless of collision.
        assert_eq!(
            schema_ref_name("record", Some("test.kw"), Some("other.ns")),
            "test.kw.record"
        );
    }

    // =========================================================================
    // alias_ref_name
    // =========================================================================

    #[test]
    fn alias_same_namespace_shortens_to_simple_name() {
        assert_eq!(
            alias_ref_name("test.aliases.OldName", Some("test.aliases")),
            "OldName"
        );
    }

    #[test]
    fn alias_different_namespace_keeps_full_name() {
        assert_eq!(
            alias_ref_name("other.ns.DiffNsAlias", Some("test.aliases")),
            "other.ns.DiffNsAlias"
        );
    }

    #[test]
    fn alias_no_namespace_keeps_simple_name() {
        assert_eq!(alias_ref_name("NoNs", Some("test.aliases")), "NoNs");
    }

    #[test]
    fn alias_schema_type_collision_keeps_full_name() {
        // An alias named `record` in the same namespace must not be shortened
        // to avoid ambiguity with the built-in `record` type.
        assert_eq!(
            alias_ref_name("test.kw.record", Some("test.kw")),
            "test.kw.record"
        );
    }

    #[test]
    fn alias_schema_type_collision_enum_keeps_full_name() {
        assert_eq!(
            alias_ref_name("test.kw.enum", Some("test.kw")),
            "test.kw.enum"
        );
    }

    #[test]
    fn alias_no_collision_same_namespace_shortens() {
        // A normal alias name (no collision) in the same namespace is shortened.
        assert_eq!(
            alias_ref_name("test.kw.NormalAlias", Some("test.kw")),
            "NormalAlias"
        );
    }

    #[test]
    fn alias_schema_type_collision_different_namespace() {
        // Different namespace -- always full, regardless of collision.
        assert_eq!(
            alias_ref_name("other.ns.record", Some("test.kw")),
            "other.ns.record"
        );
    }

    #[test]
    fn alias_schema_nil_namespace() {
        // Schema has no namespace; alias has namespace -- should keep full.
        assert_eq!(alias_ref_name("some.ns.Alias", None), "some.ns.Alias");
    }

    // =========================================================================
    // Record/Enum/Fixed alias shortening in schema_to_json
    // =========================================================================

    #[test]
    fn record_aliases_shortened_in_same_namespace() {
        let schema = AvroSchema::Record {
            name: "NewName".to_string(),
            namespace: Some("test.aliases".to_string()),
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![
                "test.aliases.SameNs".to_string(),
                "other.DiffNs".to_string(),
                "NoNs".to_string(),
            ],
            properties: HashMap::new(),
        };

        let result = schema_to_json(
            &schema,
            &mut HashSet::new(),
            Some("test.aliases"),
            &HashMap::new(),
        );
        assert_eq!(result["aliases"], json!(["SameNs", "other.DiffNs", "NoNs"]));
    }

    #[test]
    fn enum_aliases_shortened_in_same_namespace() {
        let schema = AvroSchema::Enum {
            name: "NewEnum".to_string(),
            namespace: Some("test.aliases".to_string()),
            doc: None,
            symbols: vec!["A".to_string()],
            default: None,
            aliases: vec![
                "test.aliases.OldEnum".to_string(),
                "other.ns.ForeignEnum".to_string(),
            ],
            properties: HashMap::new(),
        };

        let result = schema_to_json(
            &schema,
            &mut HashSet::new(),
            Some("test.aliases"),
            &HashMap::new(),
        );
        assert_eq!(
            result["aliases"],
            json!(["OldEnum", "other.ns.ForeignEnum"])
        );
    }

    #[test]
    fn fixed_aliases_shortened_in_same_namespace() {
        let schema = AvroSchema::Fixed {
            name: "NewFixed".to_string(),
            namespace: Some("test.aliases".to_string()),
            doc: None,
            size: 16,
            aliases: vec!["test.aliases.OldFixed".to_string()],
            properties: HashMap::new(),
        };

        let result = schema_to_json(
            &schema,
            &mut HashSet::new(),
            Some("test.aliases"),
            &HashMap::new(),
        );
        assert_eq!(result["aliases"], json!(["OldFixed"]));
    }

    #[test]
    fn record_alias_with_schema_type_collision_not_shortened() {
        // Alias named `record` in the same namespace must not be shortened.
        let schema = AvroSchema::Record {
            name: "MyRecord".to_string(),
            namespace: Some("test.kw".to_string()),
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![
                "test.kw.record".to_string(),
                "test.kw.NormalAlias".to_string(),
            ],
            properties: HashMap::new(),
        };

        let result = schema_to_json(
            &schema,
            &mut HashSet::new(),
            Some("test.kw"),
            &HashMap::new(),
        );
        assert_eq!(result["aliases"], json!(["test.kw.record", "NormalAlias"]));
    }

    // =========================================================================
    // Reference with Schema.Type collision uses full name
    // =========================================================================

    #[test]
    fn reference_uses_full_name_for_schema_type_collision() {
        // A type named `record` in namespace `test.kw`. On second occurrence
        // within the same namespace, it should still use the full name.
        let record = AvroSchema::Record {
            name: "record".to_string(),
            namespace: Some("test.kw".to_string()),
            doc: None,
            fields: vec![Field {
                name: "x".to_string(),
                schema: AvroSchema::String,
                doc: None,
                default: None,
                order: None,
                aliases: vec![],
                properties: HashMap::new(),
            }],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        };

        let mut lookup = HashMap::new();
        lookup.insert("test.kw.record".to_string(), record.clone());

        let reference = AvroSchema::Reference {
            name: "record".to_string(),
            namespace: Some("test.kw".to_string()),
            properties: HashMap::new(),
            span: None,
        };

        let mut known = HashSet::new();
        // First use inlines the definition.
        let _ = serialize_schema_tracking(&reference, &mut known, Some("test.kw"), &lookup);
        // Second use: even though namespaces match, the name `record` collides
        // with a Schema.Type name, so the full name must be used.
        let result = serialize_schema_tracking(&reference, &mut known, Some("test.kw"), &lookup);
        assert_eq!(result, json!("test.kw.record"));
    }

    // =========================================================================
    // Field serialization
    // =========================================================================

    #[test]
    fn field_with_doc_default_and_order() {
        let field = Field {
            name: "kind".to_string(),
            schema: AvroSchema::String,
            doc: Some("The kind.".to_string()),
            default: Some(json!("FOO")),
            order: Some(FieldOrder::Descending),
            aliases: vec![],
            properties: HashMap::new(),
        };

        let result = field_to_json(&field, &mut HashSet::new(), None, &HashMap::new());
        assert_eq!(result["name"], json!("kind"));
        assert_eq!(result["type"], json!("string"));
        assert_eq!(result["doc"], json!("The kind."));
        assert_eq!(result["default"], json!("FOO"));
        assert_eq!(result["order"], json!("descending"));
    }

    #[test]
    fn field_ascending_order_is_omitted() {
        let field = Field {
            name: "x".to_string(),
            schema: AvroSchema::Int,
            doc: None,
            default: None,
            order: Some(FieldOrder::Ascending),
            aliases: vec![],
            properties: HashMap::new(),
        };

        let result = field_to_json(&field, &mut HashSet::new(), None, &HashMap::new());
        // Ascending is the default and should be omitted.
        assert!(result.get("order").is_none());
    }

    #[test]
    fn field_with_ignore_order() {
        let field = Field {
            name: "x".to_string(),
            schema: AvroSchema::Int,
            doc: None,
            default: None,
            order: Some(FieldOrder::Ignore),
            aliases: vec![],
            properties: HashMap::new(),
        };

        let result = field_to_json(&field, &mut HashSet::new(), None, &HashMap::new());
        assert_eq!(result["order"], json!("ignore"));
    }

    #[test]
    fn field_with_aliases_and_properties() {
        let mut props = HashMap::new();
        props.insert("custom-prop".to_string(), json!(true));

        let field = Field {
            name: "hash".to_string(),
            schema: AvroSchema::Bytes,
            doc: None,
            default: None,
            order: None,
            aliases: vec!["old_hash".to_string(), "h".to_string()],
            properties: props,
        };

        let result = field_to_json(&field, &mut HashSet::new(), None, &HashMap::new());
        assert_eq!(result["aliases"], json!(["old_hash", "h"]));
        assert_eq!(result["custom-prop"], json!(true));
    }

    // =========================================================================
    // Protocol serialization
    // =========================================================================

    #[test]
    fn protocol_to_json_minimal() {
        let protocol = Protocol {
            name: "Echo".to_string(),
            namespace: Some("org.example".to_string()),
            doc: None,
            properties: HashMap::new(),
            types: vec![AvroSchema::Record {
                name: "Ping".to_string(),
                namespace: Some("org.example".to_string()),
                doc: None,
                fields: vec![Field {
                    name: "ts".to_string(),
                    schema: AvroSchema::Long,
                    doc: None,
                    default: Some(json!(-1)),
                    order: None,
                    aliases: vec![],
                    properties: HashMap::new(),
                }],
                is_error: false,
                aliases: vec![],
                properties: HashMap::new(),
            }],
            messages: HashMap::new(),
        };

        let result = protocol_to_json(&protocol);
        assert_eq!(result["protocol"], json!("Echo"));
        assert_eq!(result["namespace"], json!("org.example"));
        let types = result["types"]
            .as_array()
            .expect("types should be an array");
        assert_eq!(types.len(), 1);
        assert_eq!(types[0]["name"], json!("Ping"));
        assert_eq!(result["messages"], json!({}));
    }

    #[test]
    fn protocol_with_empty_namespace_omits_key() {
        // Java treats an empty namespace as equivalent to no namespace and
        // omits the key from JSON output. We match that behavior.
        let protocol = Protocol {
            name: "Simple".to_string(),
            namespace: Some(String::new()),
            doc: None,
            properties: HashMap::new(),
            types: vec![],
            messages: HashMap::new(),
        };

        let result = protocol_to_json(&protocol);
        assert_eq!(result["protocol"], json!("Simple"));
        assert!(
            result.get("namespace").is_none(),
            "empty namespace should be omitted from JSON"
        );
    }

    #[test]
    fn protocol_with_doc_and_properties() {
        let mut props = HashMap::new();
        props.insert("version".to_string(), json!("1.0"));

        let protocol = Protocol {
            name: "Greeter".to_string(),
            namespace: None,
            doc: Some("A greeter protocol.".to_string()),
            properties: props,
            types: vec![],
            messages: HashMap::new(),
        };

        let result = protocol_to_json(&protocol);
        assert_eq!(result["protocol"], json!("Greeter"));
        assert_eq!(result["doc"], json!("A greeter protocol."));
        assert_eq!(result["version"], json!("1.0"));
    }

    #[test]
    fn protocol_with_messages() {
        let protocol = Protocol {
            name: "Svc".to_string(),
            namespace: Some("org.example".to_string()),
            doc: None,
            properties: HashMap::new(),
            types: vec![],
            messages: {
                let mut msgs = HashMap::new();
                msgs.insert(
                    "hello".to_string(),
                    Message {
                        doc: Some("Say hello.".to_string()),
                        properties: HashMap::new(),
                        request: vec![Field {
                            name: "greeting".to_string(),
                            schema: AvroSchema::String,
                            doc: None,
                            default: None,
                            order: None,
                            aliases: vec![],
                            properties: HashMap::new(),
                        }],
                        response: AvroSchema::String,
                        errors: None,
                        one_way: false,
                    },
                );
                msgs.insert(
                    "ping".to_string(),
                    Message {
                        doc: None,
                        properties: HashMap::new(),
                        request: vec![],
                        response: AvroSchema::Null,
                        errors: None,
                        one_way: true,
                    },
                );
                msgs
            },
        };

        let result = protocol_to_json(&protocol);
        let messages = result["messages"]
            .as_object()
            .expect("messages should be an object");
        assert_eq!(messages.len(), 2);

        let hello = &messages["hello"];
        assert_eq!(hello["doc"], json!("Say hello."));
        assert_eq!(hello["response"], json!("string"));
        assert!(hello.get("one-way").is_none());

        let ping = &messages["ping"];
        assert_eq!(ping["one-way"], json!(true));
        assert_eq!(ping["response"], json!("null"));
    }

    // =========================================================================
    // build_lookup
    // =========================================================================

    #[test]
    fn build_lookup_collects_nested_types() {
        // A record containing a field with an enum type. Both the record and
        // the enum should appear in the lookup.
        let status_enum = AvroSchema::Enum {
            name: "Status".to_string(),
            namespace: Some("org.example".to_string()),
            doc: None,
            symbols: vec!["A".to_string(), "B".to_string()],
            default: None,
            aliases: vec![],
            properties: HashMap::new(),
        };

        let record = AvroSchema::Record {
            name: "Rec".to_string(),
            namespace: Some("org.example".to_string()),
            doc: None,
            fields: vec![Field {
                name: "status".to_string(),
                schema: status_enum,
                doc: None,
                default: None,
                order: None,
                aliases: vec![],
                properties: HashMap::new(),
            }],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        };

        let lookup = build_lookup(&[record], Some("org.example"));
        assert!(lookup.contains_key("org.example.Rec"));
        assert!(lookup.contains_key("org.example.Status"));
    }

    #[test]
    fn build_lookup_uses_default_namespace_for_unqualified_types() {
        // A record with no explicit namespace should inherit the default.
        let record = AvroSchema::Record {
            name: "Rec".to_string(),
            namespace: None,
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        };

        let lookup = build_lookup(&[record], Some("org.default"));
        assert!(lookup.contains_key("org.default.Rec"));
    }

    // =========================================================================
    // Named type deduplication: second occurrence becomes bare string
    // =========================================================================

    #[test]
    fn named_type_second_occurrence_is_bare_string() {
        let schema = AvroSchema::Record {
            name: "Rec".to_string(),
            namespace: Some("org.test".to_string()),
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        };

        let mut known = HashSet::new();
        let lookup = HashMap::new();

        // First serialization: full object.
        let first = serialize_schema_tracking(&schema, &mut known, None, &lookup);
        assert!(first.is_object());

        // Second serialization: bare string.
        let second = serialize_schema_tracking(&schema, &mut known, None, &lookup);
        assert_eq!(second, json!("org.test.Rec"));
    }

    #[test]
    fn named_type_second_occurrence_uses_short_name_in_same_namespace() {
        let schema = AvroSchema::Enum {
            name: "Color".to_string(),
            namespace: Some("org.palette".to_string()),
            doc: None,
            symbols: vec!["RED".to_string()],
            default: None,
            aliases: vec![],
            properties: HashMap::new(),
        };

        let mut known = HashSet::new();
        let lookup = HashMap::new();

        // First serialization within matching namespace: full object.
        let first = serialize_schema_tracking(&schema, &mut known, Some("org.palette"), &lookup);
        assert!(first.is_object());

        // Second serialization within same namespace: short name.
        let second = serialize_schema_tracking(&schema, &mut known, Some("org.palette"), &lookup);
        assert_eq!(second, json!("Color"));
    }
}
