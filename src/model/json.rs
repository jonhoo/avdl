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
// - JSON object key order is carefully controlled to match Java output.

use indexmap::IndexMap;
use indexmap::IndexSet;
use serde_json::{Map, Value};

use super::protocol::{Message, Protocol};
use super::schema::{AvroSchema, Field, FieldOrder, LogicalType};

/// Serialize a `Protocol` to a `serde_json::Value` matching the Java Avro tools output.
pub fn protocol_to_json(protocol: &Protocol) -> Value {
    let mut known_names = IndexSet::new();
    let mut obj = IndexMap::new();

    obj.insert("protocol".to_string(), Value::String(protocol.name.clone()));
    if let Some(ns) = &protocol.namespace {
        obj.insert("namespace".to_string(), Value::String(ns.clone()));
    }
    if let Some(doc) = &protocol.doc {
        obj.insert("doc".to_string(), Value::String(doc.clone()));
    }
    for (k, v) in &protocol.properties {
        obj.insert(k.clone(), v.clone());
    }

    let types: Vec<Value> = protocol
        .types
        .iter()
        .map(|s| schema_to_json(s, &mut known_names, protocol.namespace.as_deref()))
        .collect();
    obj.insert("types".to_string(), Value::Array(types));

    let mut messages_obj = IndexMap::new();
    for (name, msg) in &protocol.messages {
        messages_obj.insert(
            name.clone(),
            message_to_json(msg, &mut known_names, protocol.namespace.as_deref()),
        );
    }
    obj.insert("messages".to_string(), indexmap_to_value(messages_obj));

    indexmap_to_value(obj)
}

/// Serialize an `AvroSchema` to JSON. For named types, the first occurrence
/// is serialized inline; subsequent occurrences are bare name strings.
pub fn schema_to_json(
    schema: &AvroSchema,
    known_names: &mut IndexSet<String>,
    enclosing_namespace: Option<&str>,
) -> Value {
    match schema {
        // =====================================================================
        // Primitives: serialize as plain strings.
        // =====================================================================
        AvroSchema::Null => Value::String("null".to_string()),
        AvroSchema::Boolean => Value::String("boolean".to_string()),
        AvroSchema::Int => Value::String("int".to_string()),
        AvroSchema::Long => Value::String("long".to_string()),
        AvroSchema::Float => Value::String("float".to_string()),
        AvroSchema::Double => Value::String("double".to_string()),
        AvroSchema::Bytes => Value::String("bytes".to_string()),
        AvroSchema::String => Value::String("string".to_string()),

        // =====================================================================
        // Record: key order is type, name, namespace (if different), doc,
        // fields, aliases, then properties.
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
            let full_name = match namespace {
                Some(ns) => format!("{ns}.{name}"),
                None => name.clone(),
            };
            if known_names.contains(&full_name) {
                return Value::String(schema_ref_name(
                    name,
                    namespace.as_deref(),
                    enclosing_namespace,
                ));
            }
            known_names.insert(full_name);

            let mut obj = IndexMap::new();
            obj.insert(
                "type".to_string(),
                Value::String(if *is_error { "error" } else { "record" }.to_string()),
            );
            obj.insert("name".to_string(), Value::String(name.clone()));
            if namespace.as_deref() != enclosing_namespace {
                if let Some(ns) = namespace {
                    obj.insert("namespace".to_string(), Value::String(ns.clone()));
                }
            }
            if let Some(doc) = doc {
                obj.insert("doc".to_string(), Value::String(doc.clone()));
            }
            let fields_json: Vec<Value> = fields
                .iter()
                .map(|f| {
                    field_to_json(
                        f,
                        known_names,
                        namespace.as_deref().or(enclosing_namespace),
                    )
                })
                .collect();
            obj.insert("fields".to_string(), Value::Array(fields_json));
            if !aliases.is_empty() {
                let aliases_json: Vec<Value> =
                    aliases.iter().map(|a| Value::String(a.clone())).collect();
                obj.insert("aliases".to_string(), Value::Array(aliases_json));
            }
            for (k, v) in properties {
                obj.insert(k.clone(), v.clone());
            }
            indexmap_to_value(obj)
        }

        // =====================================================================
        // Enum: key order is type, name, namespace (if different), doc,
        // symbols, default, aliases, then properties.
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
            let full_name = match namespace {
                Some(ns) => format!("{ns}.{name}"),
                None => name.clone(),
            };
            if known_names.contains(&full_name) {
                return Value::String(schema_ref_name(
                    name,
                    namespace.as_deref(),
                    enclosing_namespace,
                ));
            }
            known_names.insert(full_name);

            let mut obj = IndexMap::new();
            obj.insert(
                "type".to_string(),
                Value::String("enum".to_string()),
            );
            obj.insert("name".to_string(), Value::String(name.clone()));
            if namespace.as_deref() != enclosing_namespace {
                if let Some(ns) = namespace {
                    obj.insert("namespace".to_string(), Value::String(ns.clone()));
                }
            }
            if let Some(doc) = doc {
                obj.insert("doc".to_string(), Value::String(doc.clone()));
            }
            let symbols_json: Vec<Value> =
                symbols.iter().map(|s| Value::String(s.clone())).collect();
            obj.insert("symbols".to_string(), Value::Array(symbols_json));
            if let Some(def) = default {
                obj.insert("default".to_string(), Value::String(def.clone()));
            }
            if !aliases.is_empty() {
                let aliases_json: Vec<Value> =
                    aliases.iter().map(|a| Value::String(a.clone())).collect();
                obj.insert("aliases".to_string(), Value::Array(aliases_json));
            }
            for (k, v) in properties {
                obj.insert(k.clone(), v.clone());
            }
            indexmap_to_value(obj)
        }

        // =====================================================================
        // Fixed: key order is type, name, namespace (if different), doc,
        // size, aliases, then properties.
        // =====================================================================
        AvroSchema::Fixed {
            name,
            namespace,
            doc,
            size,
            aliases,
            properties,
        } => {
            let full_name = match namespace {
                Some(ns) => format!("{ns}.{name}"),
                None => name.clone(),
            };
            if known_names.contains(&full_name) {
                return Value::String(schema_ref_name(
                    name,
                    namespace.as_deref(),
                    enclosing_namespace,
                ));
            }
            known_names.insert(full_name);

            let mut obj = IndexMap::new();
            obj.insert(
                "type".to_string(),
                Value::String("fixed".to_string()),
            );
            obj.insert("name".to_string(), Value::String(name.clone()));
            if namespace.as_deref() != enclosing_namespace {
                if let Some(ns) = namespace {
                    obj.insert("namespace".to_string(), Value::String(ns.clone()));
                }
            }
            if let Some(doc) = doc {
                obj.insert("doc".to_string(), Value::String(doc.clone()));
            }
            obj.insert("size".to_string(), Value::Number((*size).into()));
            if !aliases.is_empty() {
                let aliases_json: Vec<Value> =
                    aliases.iter().map(|a| Value::String(a.clone())).collect();
                obj.insert("aliases".to_string(), Value::Array(aliases_json));
            }
            for (k, v) in properties {
                obj.insert(k.clone(), v.clone());
            }
            indexmap_to_value(obj)
        }

        // =====================================================================
        // Array: {"type": "array", "items": ..., ...properties}
        // =====================================================================
        AvroSchema::Array { items, properties } => {
            let mut obj = IndexMap::new();
            obj.insert(
                "type".to_string(),
                Value::String("array".to_string()),
            );
            obj.insert(
                "items".to_string(),
                schema_to_json(items, known_names, enclosing_namespace),
            );
            for (k, v) in properties {
                obj.insert(k.clone(), v.clone());
            }
            indexmap_to_value(obj)
        }

        // =====================================================================
        // Map: {"type": "map", "values": ..., ...properties}
        // =====================================================================
        AvroSchema::Map { values, properties } => {
            let mut obj = IndexMap::new();
            obj.insert(
                "type".to_string(),
                Value::String("map".to_string()),
            );
            obj.insert(
                "values".to_string(),
                schema_to_json(values, known_names, enclosing_namespace),
            );
            for (k, v) in properties {
                obj.insert(k.clone(), v.clone());
            }
            indexmap_to_value(obj)
        }

        // =====================================================================
        // Union: serialize as a JSON array of the constituent types.
        // The `is_nullable_type` flag is internal only and not serialized.
        // =====================================================================
        AvroSchema::Union { types, .. } => {
            let types_json: Vec<Value> = types
                .iter()
                .map(|t| schema_to_json(t, known_names, enclosing_namespace))
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
            let mut obj = IndexMap::new();
            match logical_type {
                LogicalType::Date => {
                    obj.insert("type".to_string(), Value::String("int".to_string()));
                    obj.insert(
                        "logicalType".to_string(),
                        Value::String("date".to_string()),
                    );
                }
                LogicalType::TimeMillis => {
                    obj.insert("type".to_string(), Value::String("int".to_string()));
                    obj.insert(
                        "logicalType".to_string(),
                        Value::String("time-millis".to_string()),
                    );
                }
                LogicalType::TimestampMillis => {
                    obj.insert("type".to_string(), Value::String("long".to_string()));
                    obj.insert(
                        "logicalType".to_string(),
                        Value::String("timestamp-millis".to_string()),
                    );
                }
                LogicalType::LocalTimestampMillis => {
                    obj.insert("type".to_string(), Value::String("long".to_string()));
                    obj.insert(
                        "logicalType".to_string(),
                        Value::String("local-timestamp-millis".to_string()),
                    );
                }
                LogicalType::Uuid => {
                    obj.insert("type".to_string(), Value::String("string".to_string()));
                    obj.insert(
                        "logicalType".to_string(),
                        Value::String("uuid".to_string()),
                    );
                }
                LogicalType::Decimal {
                    precision,
                    scale,
                } => {
                    obj.insert("type".to_string(), Value::String("bytes".to_string()));
                    obj.insert(
                        "logicalType".to_string(),
                        Value::String("decimal".to_string()),
                    );
                    obj.insert(
                        "precision".to_string(),
                        Value::Number((*precision).into()),
                    );
                    obj.insert("scale".to_string(), Value::Number((*scale).into()));
                }
            }
            for (k, v) in properties {
                obj.insert(k.clone(), v.clone());
            }
            indexmap_to_value(obj)
        }

        // =====================================================================
        // Reference: a forward reference that should have been resolved by now,
        // but we serialize as a bare name string regardless.
        // =====================================================================
        AvroSchema::Reference(name) => Value::String(name.clone()),
    }
}

// =============================================================================
// Helper: serialize a record field to JSON.
// Key order: name, type, doc, default, order (if not ascending), aliases, properties.
// =============================================================================

fn field_to_json(
    field: &Field,
    known_names: &mut IndexSet<String>,
    enclosing_namespace: Option<&str>,
) -> Value {
    let mut obj = IndexMap::new();
    obj.insert("name".to_string(), Value::String(field.name.clone()));
    obj.insert(
        "type".to_string(),
        schema_to_json(&field.schema, known_names, enclosing_namespace),
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
            obj.insert(
                "order".to_string(),
                Value::String("descending".to_string()),
            );
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
    indexmap_to_value(obj)
}

// =============================================================================
// Helper: serialize a protocol message to JSON.
// Key order: doc, properties, request, response, errors (if any), one-way (if true).
// =============================================================================

fn message_to_json(
    msg: &Message,
    known_names: &mut IndexSet<String>,
    enclosing_namespace: Option<&str>,
) -> Value {
    let mut obj = IndexMap::new();
    if let Some(doc) = &msg.doc {
        obj.insert("doc".to_string(), Value::String(doc.clone()));
    }
    for (k, v) in &msg.properties {
        obj.insert(k.clone(), v.clone());
    }
    let request: Vec<Value> = msg
        .request
        .iter()
        .map(|f| field_to_json(f, known_names, enclosing_namespace))
        .collect();
    obj.insert("request".to_string(), Value::Array(request));
    obj.insert(
        "response".to_string(),
        schema_to_json(&msg.response, known_names, enclosing_namespace),
    );
    if let Some(errors) = &msg.errors {
        let errors_json: Vec<Value> = errors
            .iter()
            .map(|e| schema_to_json(e, known_names, enclosing_namespace))
            .collect();
        obj.insert("errors".to_string(), Value::Array(errors_json));
    }
    if msg.one_way {
        obj.insert("one-way".to_string(), Value::Bool(true));
    }
    indexmap_to_value(obj)
}

/// When referencing a named type, use just the simple name if it shares the same
/// namespace as the enclosing context; otherwise use the fully qualified name.
fn schema_ref_name(
    name: &str,
    namespace: Option<&str>,
    enclosing_namespace: Option<&str>,
) -> String {
    if namespace == enclosing_namespace {
        name.to_string()
    } else {
        match namespace {
            Some(ns) => format!("{ns}.{name}"),
            None => name.to_string(),
        }
    }
}

/// Convert an `IndexMap` to a `serde_json::Value::Object`, preserving insertion order.
fn indexmap_to_value(map: IndexMap<String, Value>) -> Value {
    let json_map: Map<String, Value> = map.into_iter().collect();
    Value::Object(json_map)
}
