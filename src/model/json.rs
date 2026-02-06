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
//
// References (`AvroSchema::Reference`) are resolved against a lookup table so
// they can be inlined at their first use, just as the Java tools do. This is
// critical for test cases like `forward_ref.avdl` where an enum is defined
// after the record that uses it -- the expected JSON inlines the enum inside
// the record's field.

use indexmap::IndexMap;
use indexmap::IndexSet;
use serde_json::{Map, Value};

use super::protocol::{Message, Protocol};
use super::schema::{AvroSchema, Field, FieldOrder, LogicalType, PrimitiveType};

/// A lookup table from full type name to the actual schema definition. This
/// allows `Reference` nodes to be resolved and inlined at their first use.
pub type SchemaLookup = IndexMap<String, AvroSchema>;

/// Serialize a `Protocol` to a `serde_json::Value` matching the Java Avro tools output.
pub fn protocol_to_json(protocol: &Protocol) -> Value {
    // Build a lookup table from all named types in the protocol's type list.
    // This includes nested types inside records/fields that were registered
    // in the schema registry.
    let lookup = build_lookup(&protocol.types, protocol.namespace.as_deref());

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

    let mut messages_obj = IndexMap::new();
    for (name, msg) in &protocol.messages {
        messages_obj.insert(
            name.clone(),
            message_to_json(msg, &mut known_names, protocol.namespace.as_deref(), &lookup),
        );
    }
    obj.insert("messages".to_string(), indexmap_to_value(messages_obj));

    indexmap_to_value(obj)
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
    let mut lookup = IndexMap::new();
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
            let full_name = match effective_ns {
                Some(ns) => format!("{ns}.{name}"),
                None => name.clone(),
            };
            lookup.insert(full_name, schema.clone());
            for field in fields {
                collect_named_types(&field.schema, default_namespace, lookup);
            }
        }
        AvroSchema::Enum {
            name, namespace, ..
        }
        | AvroSchema::Fixed {
            name, namespace, ..
        } => {
            let effective_ns = namespace.as_deref().or(default_namespace);
            let full_name = match effective_ns {
                Some(ns) => format!("{ns}.{name}"),
                None => name.clone(),
            };
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

/// Serialize an `AvroSchema` to JSON. For named types, the first occurrence
/// is serialized inline; subsequent occurrences are bare name strings.
///
/// The `lookup` parameter allows `Reference` nodes to be resolved and inlined
/// at their first use.
pub fn schema_to_json(
    schema: &AvroSchema,
    known_names: &mut IndexSet<String>,
    enclosing_namespace: Option<&str>,
    lookup: &SchemaLookup,
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
        // Annotated primitive: a primitive with custom properties, serialized
        // as {"type": "int", ...properties} instead of bare "int".
        // =====================================================================
        AvroSchema::AnnotatedPrimitive { kind, properties } => {
            let mut obj = IndexMap::new();
            obj.insert(
                "type".to_string(),
                Value::String(kind.as_str().to_string()),
            );
            for (k, v) in properties {
                obj.insert(k.clone(), v.clone());
            }
            indexmap_to_value(obj)
        }

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
                        lookup,
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
                schema_to_json(items, known_names, enclosing_namespace, lookup),
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
                schema_to_json(values, known_names, enclosing_namespace, lookup),
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
        // Reference: resolve against the lookup to inline at first use, or
        // output a bare name for subsequent uses.
        // =====================================================================
        AvroSchema::Reference {
            name,
            namespace,
            ..
        } => {
            let full_name = match namespace {
                Some(ns) => format!("{ns}.{name}"),
                None => name.clone(),
            };

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
    lookup: &SchemaLookup,
) -> Value {
    let mut obj = IndexMap::new();
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
    lookup: &SchemaLookup,
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
