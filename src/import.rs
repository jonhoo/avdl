// ==============================================================================
// Import Resolution for .avdl, .avpr, and .avsc Files
// ==============================================================================
//
// In Avro IDL, three types of imports are supported:
//   1. `import idl "file.avdl"` -- recursively parse another IDL file, merging
//      its types and messages into the current protocol.
//   2. `import protocol "file.avpr"` -- parse a JSON protocol file, registering
//      its schemas and extracting its messages.
//   3. `import schema "file.avsc"` -- parse a single JSON schema file,
//      registering it.
//
// This module provides:
//   - `ImportContext`: state tracking for cycle prevention and search paths
//   - `import_protocol` / `import_schema`: JSON-based import helpers
//   - `json_to_schema` and friends: conversion from serde_json `Value` to our
//     `AvroSchema` model
//
// The `import idl` case is intentionally NOT handled here because it requires
// calling the IDL reader/parser, which would create a circular dependency.
// Instead, the reader calls into `ImportContext` to resolve paths and check for
// cycles, then handles the recursive parse itself.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use serde_json::Value;

use crate::error::{IdlError, Result};
use crate::model::protocol::Message;
use crate::model::schema::{AvroSchema, FieldOrder, LogicalType, PrimitiveType};
use crate::resolve::SchemaRegistry;

// ==============================================================================
// Import Context: Cycle Prevention and Path Resolution
// ==============================================================================

/// Tracks import state across recursive IDL parsing.
///
/// The Java Avro compiler resolves imports first relative to the current file's
/// directory, then via the classpath. We replace the classpath mechanism with
/// configurable import search directories, which serves the same purpose.
pub struct ImportContext {
    /// Files that have already been imported (canonical paths, for cycle prevention).
    pub read_locations: HashSet<PathBuf>,
    /// Additional directories to search for imports (replaces Java classpath).
    pub import_dirs: Vec<PathBuf>,
}

impl ImportContext {
    pub fn new(import_dirs: Vec<PathBuf>) -> Self {
        ImportContext {
            read_locations: HashSet::new(),
            import_dirs,
        }
    }

    /// Resolve an import file path. Searches:
    /// 1. Relative to `current_dir` (the directory containing the importing file)
    /// 2. In each import search directory, in order
    ///
    /// Returns the canonical (absolute, symlink-resolved) path on success.
    pub fn resolve_import(&self, import_file: &str, current_dir: &Path) -> Result<PathBuf> {
        // Try relative to current file's directory first.
        let relative = current_dir.join(import_file);
        if relative.exists() {
            return relative.canonicalize().map_err(|e| {
                IdlError::Other(format!(
                    "canonicalize import path `{import_file}` relative to `{}`: {e}",
                    current_dir.display()
                ))
            });
        }

        // Try each import search directory.
        for dir in &self.import_dirs {
            let candidate = dir.join(import_file);
            if candidate.exists() {
                return candidate.canonicalize().map_err(|e| {
                    IdlError::Other(format!(
                        "canonicalize import path `{import_file}` in import dir `{}`: {e}",
                        dir.display()
                    ))
                });
            }
        }

        Err(IdlError::Other(format!(
            "import not found: {import_file} (searched relative to {} and {} import dir(s))",
            current_dir.display(),
            self.import_dirs.len()
        )))
    }

    /// Check if a file has already been imported (cycle prevention).
    ///
    /// If the file has not yet been imported, marks it as imported and returns
    /// `false`. If the file was already imported, returns `true` (indicating
    /// the caller should skip re-importing it).
    pub fn mark_imported(&mut self, path: &Path) -> bool {
        !self.read_locations.insert(path.to_path_buf())
    }
}

// ==============================================================================
// Recursive Named Type Registration
// ==============================================================================
//
// When importing `.avsc` or `.avpr` files, named types (record, enum, fixed)
// can be nested arbitrarily deep inside record fields, union branches, array
// items, or map values. Java's `JsonSchemaParser.parse()` recursively registers
// all such nested types via `ParseContext.put()`. We replicate this behavior
// here so that subsequent IDL code can reference nested types by name.
//
// This walk follows the same structure as `collect_named_types` in
// `model/json.rs`, which does the equivalent for JSON serialization lookups.

/// Recursively walk an `AvroSchema` tree and register every named type
/// (record, enum, fixed) found in the `SchemaRegistry`.
///
/// Duplicate registrations are silently ignored (the first definition wins),
/// matching Java's behavior for imports.
fn register_all_named_types(schema: &AvroSchema, registry: &mut SchemaRegistry) {
    match schema {
        AvroSchema::Record { fields, .. } => {
            // Register the record itself first, then recurse into its fields
            // to pick up any nested named types.
            let _ = registry.register(schema.clone());
            for field in fields {
                register_all_named_types(&field.schema, registry);
            }
        }
        AvroSchema::Enum { .. } | AvroSchema::Fixed { .. } => {
            let _ = registry.register(schema.clone());
        }
        AvroSchema::Array { items, .. } => {
            register_all_named_types(items, registry);
        }
        AvroSchema::Map { values, .. } => {
            register_all_named_types(values, registry);
        }
        AvroSchema::Union { types, .. } => {
            for t in types {
                register_all_named_types(t, registry);
            }
        }
        // Primitives, logical types, annotated primitives, and references
        // contain no nested named type definitions to register.
        _ => {}
    }
}

// ==============================================================================
// JSON Protocol Import (.avpr)
// ==============================================================================

/// Import a JSON protocol file (.avpr), registering its types and returning
/// its messages.
///
/// The `.avpr` format is the JSON serialization of an Avro protocol. It contains
/// a `types` array of named schema definitions and a `messages` object mapping
/// message names to their definitions.
pub fn import_protocol(
    path: &Path,
    registry: &mut SchemaRegistry,
) -> Result<IndexMap<String, Message>> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        IdlError::Other(format!("read protocol file `{}`: {e}", path.display()))
    })?;
    let json: Value = serde_json::from_str(&content)
        .map_err(|e| IdlError::Parse(format!("invalid JSON in {}: {e}", path.display())))?;

    let default_namespace = json.get("namespace").and_then(|n| n.as_str());
    let mut messages = IndexMap::new();

    // Extract types from the protocol JSON and register them, including any
    // nested named types within record fields, union branches, etc.
    if let Some(types) = json.get("types").and_then(|t| t.as_array()) {
        for (i, type_json) in types.iter().enumerate() {
            let schema = json_to_schema(type_json, default_namespace).map_err(|e| {
                IdlError::Other(format!(
                    "parse type at index {i} in protocol `{}`: {e}",
                    path.display()
                ))
            })?;
            register_all_named_types(&schema, registry);
        }
    }

    // Extract messages.
    if let Some(msgs) = json.get("messages").and_then(|m| m.as_object()) {
        for (name, msg_json) in msgs {
            let message = json_to_message(msg_json, default_namespace).map_err(|e| {
                IdlError::Other(format!(
                    "parse message `{name}` in protocol `{}`: {e}",
                    path.display()
                ))
            })?;
            messages.insert(name.clone(), message);
        }
    }

    Ok(messages)
}

// ==============================================================================
// JSON Schema Import (.avsc)
// ==============================================================================

/// Import a JSON schema file (.avsc), registering the schema and any nested
/// named types it contains.
///
/// The `.avsc` format is the JSON serialization of a single Avro schema. All
/// named types (record, enum, fixed) found in the schema tree -- including those
/// nested inside record fields, union branches, array items, or map values --
/// are registered so that subsequent IDL code can reference them by name.
pub fn import_schema(path: &Path, registry: &mut SchemaRegistry) -> Result<()> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        IdlError::Other(format!("read schema file `{}`: {e}", path.display()))
    })?;
    let json: Value = serde_json::from_str(&content)
        .map_err(|e| IdlError::Parse(format!("invalid JSON in {}: {e}", path.display())))?;

    let schema = json_to_schema(&json, None).map_err(|e| {
        IdlError::Other(format!("parse schema from `{}`: {e}", path.display()))
    })?;
    register_all_named_types(&schema, registry);

    Ok(())
}

// ==============================================================================
// JSON -> Schema Conversion
// ==============================================================================
//
// These functions convert serde_json `Value` trees into our `AvroSchema` model.
// They handle the three JSON representations of Avro schemas:
//   - A string: either a primitive type name or a reference to a named type
//   - An array: a union type
//   - An object: a complex type (record, enum, fixed, array, map, or a
//     primitive with annotations like logical types)

/// Convert a JSON value to an `AvroSchema`.
///
/// The `default_namespace` is inherited from the enclosing protocol or record
/// and is used to qualify bare type names into fully-qualified references.
fn json_to_schema(json: &Value, default_namespace: Option<&str>) -> Result<AvroSchema> {
    match json {
        // String references: either a primitive name or a named type reference.
        Value::String(s) => string_to_schema(s, default_namespace),

        // Array = union type.
        Value::Array(types) => {
            let schemas: Result<Vec<_>> = types
                .iter()
                .map(|t| json_to_schema(t, default_namespace))
                .collect();
            Ok(AvroSchema::Union {
                types: schemas?,
                is_nullable_type: false,
            })
        }

        // Object = complex type (record, enum, fixed, array, map, or annotated primitive).
        Value::Object(obj) => object_to_schema(obj, default_namespace),

        _ => Err(IdlError::Parse(format!("invalid schema JSON: {json}"))),
    }
}

/// Parse a string as either a primitive type name or a named type reference.
fn string_to_schema(s: &str, default_namespace: Option<&str>) -> Result<AvroSchema> {
    match s {
        "null" => Ok(AvroSchema::Null),
        "boolean" => Ok(AvroSchema::Boolean),
        "int" => Ok(AvroSchema::Int),
        "long" => Ok(AvroSchema::Long),
        "float" => Ok(AvroSchema::Float),
        "double" => Ok(AvroSchema::Double),
        "bytes" => Ok(AvroSchema::Bytes),
        "string" => Ok(AvroSchema::String),
        type_name => {
            // Named type reference. Split into separate name and namespace
            // so the Reference tracks them independently.
            if type_name.contains('.') {
                let pos = type_name.rfind('.').expect("dot presence checked above");
                Ok(AvroSchema::Reference {
                    name: type_name[pos + 1..].to_string(),
                    namespace: Some(type_name[..pos].to_string()),
                    properties: IndexMap::new(),
                })
            } else {
                Ok(AvroSchema::Reference {
                    name: type_name.to_string(),
                    namespace: default_namespace.map(|s| s.to_string()),
                    properties: IndexMap::new(),
                })
            }
        }
    }
}

/// Parse a JSON object into an `AvroSchema` based on its `type` field.
fn object_to_schema(
    obj: &serde_json::Map<String, Value>,
    default_namespace: Option<&str>,
) -> Result<AvroSchema> {
    let type_str = obj
        .get("type")
        .and_then(|t| t.as_str())
        .ok_or_else(|| IdlError::Parse("schema object missing 'type' field".to_string()))?;

    match type_str {
        "record" | "error" => parse_record(obj, type_str, default_namespace),
        "enum" => parse_enum(obj, default_namespace),
        "fixed" => parse_fixed(obj, default_namespace),
        "array" => parse_array(obj, default_namespace),
        "map" => parse_map(obj, default_namespace),

        // A primitive type with optional logical type or custom properties.
        prim @ ("null" | "boolean" | "int" | "long" | "float" | "double" | "bytes"
        | "string") => parse_annotated_primitive(obj, prim, default_namespace),

        other => Err(IdlError::Parse(format!("unknown schema type: {other}"))),
    }
}

// ==============================================================================
// Named Type Parsers
// ==============================================================================

/// Split a potentially fully-qualified Avro name into `(simple_name, Option<namespace>)`.
///
/// In Avro's JSON encoding, the `name` field of a named type may contain a
/// fully-qualified name like `"ns.other.schema.Baz"`. Per the Avro spec, the
/// portion before the last dot is the namespace and the portion after is the
/// simple name. This mirrors the Java `Schema.Name` constructor behavior.
fn split_qualified_name(raw_name: &str) -> (String, Option<String>) {
    if let Some(pos) = raw_name.rfind('.') {
        (raw_name[pos + 1..].to_string(), Some(raw_name[..pos].to_string()))
    } else {
        (raw_name.to_string(), None)
    }
}

fn parse_record(
    obj: &serde_json::Map<String, Value>,
    type_str: &str,
    default_namespace: Option<&str>,
) -> Result<AvroSchema> {
    let raw_name = obj
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| IdlError::Parse("record missing 'name'".to_string()))?;
    let (name, inferred_ns) = split_qualified_name(raw_name);
    let namespace = obj
        .get("namespace")
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
        .or(inferred_ns)
        .or_else(|| default_namespace.map(|s| s.to_string()));
    let doc = obj
        .get("doc")
        .and_then(|d| d.as_str())
        .map(|s| s.to_string());
    let is_error = type_str == "error";

    // Fields inherit the record's namespace (or the default) for resolving
    // unqualified type references.
    let ns_for_fields = namespace.as_deref().or(default_namespace);
    let fields = if let Some(Value::Array(fields_json)) = obj.get("fields") {
        fields_json
            .iter()
            .enumerate()
            .map(|(i, f)| {
                json_to_field(f, ns_for_fields).map_err(|e| {
                    IdlError::Other(format!("parse field at index {i} of record `{name}`: {e}"))
                })
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        vec![]
    };

    let aliases = extract_string_array(obj.get("aliases"));

    let properties = collect_extra_properties(
        obj,
        &["type", "name", "namespace", "doc", "fields", "aliases"],
    );

    Ok(AvroSchema::Record {
        name,
        namespace,
        doc,
        fields,
        is_error,
        aliases,
        properties,
    })
}

fn parse_enum(
    obj: &serde_json::Map<String, Value>,
    default_namespace: Option<&str>,
) -> Result<AvroSchema> {
    let raw_name = obj
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| IdlError::Parse("enum missing 'name'".to_string()))?;
    let (name, inferred_ns) = split_qualified_name(raw_name);
    let namespace = obj
        .get("namespace")
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
        .or(inferred_ns)
        .or_else(|| default_namespace.map(|s| s.to_string()));
    let doc = obj
        .get("doc")
        .and_then(|d| d.as_str())
        .map(|s| s.to_string());
    let symbols = extract_string_array(obj.get("symbols"));
    let default = obj
        .get("default")
        .and_then(|d| d.as_str())
        .map(|s| s.to_string());
    let aliases = extract_string_array(obj.get("aliases"));

    let properties = collect_extra_properties(
        obj,
        &[
            "type",
            "name",
            "namespace",
            "doc",
            "symbols",
            "default",
            "aliases",
        ],
    );

    Ok(AvroSchema::Enum {
        name,
        namespace,
        doc,
        symbols,
        default,
        aliases,
        properties,
    })
}

fn parse_fixed(
    obj: &serde_json::Map<String, Value>,
    default_namespace: Option<&str>,
) -> Result<AvroSchema> {
    let raw_name = obj
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| IdlError::Parse("fixed missing 'name'".to_string()))?;
    let (name, inferred_ns) = split_qualified_name(raw_name);
    let namespace = obj
        .get("namespace")
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
        .or(inferred_ns)
        .or_else(|| default_namespace.map(|s| s.to_string()));
    let doc = obj
        .get("doc")
        .and_then(|d| d.as_str())
        .map(|s| s.to_string());
    let size_u64 = obj
        .get("size")
        .and_then(|s| s.as_u64())
        .ok_or_else(|| IdlError::Parse("fixed missing 'size'".to_string()))?;
    let size = u32::try_from(size_u64).map_err(|_| {
        IdlError::Parse(format!("fixed size {size_u64} exceeds maximum ({})", u32::MAX))
    })?;
    let aliases = extract_string_array(obj.get("aliases"));

    let properties = collect_extra_properties(
        obj,
        &["type", "name", "namespace", "doc", "size", "aliases"],
    );

    Ok(AvroSchema::Fixed {
        name,
        namespace,
        doc,
        size,
        aliases,
        properties,
    })
}

// ==============================================================================
// Complex Type Parsers
// ==============================================================================

fn parse_array(
    obj: &serde_json::Map<String, Value>,
    default_namespace: Option<&str>,
) -> Result<AvroSchema> {
    let items = obj
        .get("items")
        .ok_or_else(|| IdlError::Parse("array missing 'items'".to_string()))?;
    let items_schema = json_to_schema(items, default_namespace)
        .map_err(|e| IdlError::Other(format!("parse array items schema: {e}")))?;
    let properties = collect_extra_properties(obj, &["type", "items"]);
    Ok(AvroSchema::Array {
        items: Box::new(items_schema),
        properties,
    })
}

fn parse_map(
    obj: &serde_json::Map<String, Value>,
    default_namespace: Option<&str>,
) -> Result<AvroSchema> {
    let values = obj
        .get("values")
        .ok_or_else(|| IdlError::Parse("map missing 'values'".to_string()))?;
    let values_schema = json_to_schema(values, default_namespace)
        .map_err(|e| IdlError::Other(format!("parse map values schema: {e}")))?;
    let properties = collect_extra_properties(obj, &["type", "values"]);
    Ok(AvroSchema::Map {
        values: Box::new(values_schema),
        properties,
    })
}

// ==============================================================================
// Annotated Primitive Parser (handles logical types and custom properties)
// ==============================================================================

/// Parse a JSON object whose `type` is a primitive (e.g., `{"type": "long",
/// "logicalType": "timestamp-millis"}`).
///
/// If a `logicalType` annotation is present and recognized, we produce an
/// `AvroSchema::Logical`. Otherwise, we return the bare primitive.
fn parse_annotated_primitive(
    obj: &serde_json::Map<String, Value>,
    prim: &str,
    _default_namespace: Option<&str>,
) -> Result<AvroSchema> {
    if let Some(logical) = obj.get("logicalType").and_then(|l| l.as_str()) {
        let lt = match logical {
            "date" => LogicalType::Date,
            "time-millis" => LogicalType::TimeMillis,
            "timestamp-millis" => LogicalType::TimestampMillis,
            "local-timestamp-millis" => LogicalType::LocalTimestampMillis,
            "uuid" => LogicalType::Uuid,
            "decimal" => {
                let precision_u64 =
                    obj.get("precision").and_then(|p| p.as_u64()).unwrap_or(0);
                if precision_u64 < 1 {
                    return Err(IdlError::Parse(
                        "decimal precision must be >= 1".to_string(),
                    ));
                }
                let precision = u32::try_from(precision_u64).map_err(|_| {
                    IdlError::Parse("decimal precision too large".to_string())
                })?;
                let scale_u64 =
                    obj.get("scale").and_then(|s| s.as_u64()).unwrap_or(0);
                let scale = u32::try_from(scale_u64).map_err(|_| {
                    IdlError::Parse("decimal scale too large".to_string())
                })?;
                LogicalType::Decimal { precision, scale }
            }
            _ => {
                // Unknown logical type -- preserve as properties on AnnotatedPrimitive.
                let properties = collect_extra_properties(obj, &["type"]);
                return Ok(AvroSchema::AnnotatedPrimitive {
                    kind: str_to_primitive_type(prim),
                    properties,
                });
            }
        };

        let properties = collect_extra_properties(
            obj,
            &["type", "logicalType", "precision", "scale"],
        );
        return Ok(AvroSchema::Logical {
            logical_type: lt,
            properties,
        });
    }

    // Primitive with no logical type. If there are extra properties beyond
    // "type", wrap in AnnotatedPrimitive to preserve them.
    let properties = collect_extra_properties(obj, &["type"]);
    if properties.is_empty() {
        primitive_from_str(prim)
    } else {
        Ok(AvroSchema::AnnotatedPrimitive {
            kind: str_to_primitive_type(prim),
            properties,
        })
    }
}

/// Map a primitive type name string to its `AvroSchema` variant.
fn primitive_from_str(name: &str) -> Result<AvroSchema> {
    match name {
        "null" => Ok(AvroSchema::Null),
        "boolean" => Ok(AvroSchema::Boolean),
        "int" => Ok(AvroSchema::Int),
        "long" => Ok(AvroSchema::Long),
        "float" => Ok(AvroSchema::Float),
        "double" => Ok(AvroSchema::Double),
        "bytes" => Ok(AvroSchema::Bytes),
        "string" => Ok(AvroSchema::String),
        other => Err(IdlError::Parse(format!("unknown primitive type: {other}"))),
    }
}

/// Map a primitive type name string to its `PrimitiveType` variant.
///
/// Only called from `parse_annotated_primitive` where the type name has
/// already been matched as a primitive in `object_to_schema`.
fn str_to_primitive_type(name: &str) -> PrimitiveType {
    match name {
        "null" => PrimitiveType::Null,
        "boolean" => PrimitiveType::Boolean,
        "int" => PrimitiveType::Int,
        "long" => PrimitiveType::Long,
        "float" => PrimitiveType::Float,
        "double" => PrimitiveType::Double,
        "bytes" => PrimitiveType::Bytes,
        "string" => PrimitiveType::String,
        // Only called from parse_annotated_primitive where type was already matched as primitive
        _ => unreachable!("str_to_primitive_type called with non-primitive: {name}"),
    }
}

// ==============================================================================
// Field and Message Parsers
// ==============================================================================

fn json_to_field(
    json: &Value,
    default_namespace: Option<&str>,
) -> Result<crate::model::schema::Field> {
    let obj = json
        .as_object()
        .ok_or_else(|| IdlError::Parse("field must be an object".to_string()))?;

    let name = obj
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| IdlError::Parse("field missing 'name'".to_string()))?
        .to_string();
    let type_json = obj
        .get("type")
        .ok_or_else(|| IdlError::Parse("field missing 'type'".to_string()))?;
    let schema = json_to_schema(type_json, default_namespace)
        .map_err(|e| IdlError::Other(format!("parse type for field `{name}`: {e}")))?;
    let doc = obj
        .get("doc")
        .and_then(|d| d.as_str())
        .map(|s| s.to_string());
    let default = obj.get("default").cloned();

    let order = obj
        .get("order")
        .and_then(|o| o.as_str())
        .and_then(|o| match o {
            "ascending" => Some(FieldOrder::Ascending),
            "descending" => Some(FieldOrder::Descending),
            "ignore" => Some(FieldOrder::Ignore),
            _ => None,
        });

    let aliases = extract_string_array(obj.get("aliases"));

    let properties = collect_extra_properties(
        obj,
        &["name", "type", "doc", "default", "order", "aliases"],
    );

    Ok(crate::model::schema::Field {
        name,
        schema,
        doc,
        default,
        order,
        aliases,
        properties,
    })
}

fn json_to_message(json: &Value, default_namespace: Option<&str>) -> Result<Message> {
    let obj = json
        .as_object()
        .ok_or_else(|| IdlError::Parse("message must be an object".to_string()))?;

    let doc = obj
        .get("doc")
        .and_then(|d| d.as_str())
        .map(|s| s.to_string());
    let one_way = obj
        .get("one-way")
        .and_then(|o| o.as_bool())
        .unwrap_or(false);

    let request = if let Some(Value::Array(params)) = obj.get("request") {
        params
            .iter()
            .enumerate()
            .map(|(i, p)| {
                json_to_field(p, default_namespace).map_err(|e| {
                    IdlError::Other(format!("parse request parameter at index {i}: {e}"))
                })
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        vec![]
    };

    let response = if let Some(resp) = obj.get("response") {
        json_to_schema(resp, default_namespace)
            .map_err(|e| IdlError::Other(format!("parse response type for message: {e}")))?
    } else {
        AvroSchema::Null
    };

    let errors = if let Some(Value::Array(errs)) = obj.get("errors") {
        Some(
            errs.iter()
                .enumerate()
                .map(|(i, e)| {
                    json_to_schema(e, default_namespace).map_err(|e| {
                        IdlError::Other(format!("parse error type at index {i} for message: {e}"))
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        )
    } else {
        None
    };

    let properties = collect_extra_properties(
        obj,
        &["doc", "request", "response", "errors", "one-way"],
    );

    Ok(Message {
        doc,
        properties,
        request,
        response,
        errors,
        one_way,
    })
}

// ==============================================================================
// Helpers
// ==============================================================================

/// Extract a JSON array of strings into a `Vec<String>`, returning an empty
/// vec if the value is absent or not an array of strings.
fn extract_string_array(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => vec![],
    }
}

/// Collect all keys from a JSON object that are NOT in the `known_keys` set,
/// preserving insertion order. This captures custom properties/annotations.
fn collect_extra_properties(
    obj: &serde_json::Map<String, Value>,
    known_keys: &[&str],
) -> IndexMap<String, Value> {
    let mut properties = IndexMap::new();
    for (k, v) in obj {
        if !known_keys.contains(&k.as_str()) {
            properties.insert(k.clone(), v.clone());
        }
    }
    properties
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // =========================================================================
    // ImportContext tests
    // =========================================================================

    #[test]
    fn mark_imported_returns_false_on_first_call() {
        let mut ctx = ImportContext::new(vec![]);
        let path = PathBuf::from("/tmp/test.avdl");
        assert!(!ctx.mark_imported(&path), "first import should return false");
    }

    #[test]
    fn mark_imported_returns_true_on_subsequent_calls() {
        let mut ctx = ImportContext::new(vec![]);
        let path = PathBuf::from("/tmp/test.avdl");
        ctx.mark_imported(&path);
        assert!(
            ctx.mark_imported(&path),
            "second import should return true (cycle detected)"
        );
    }

    // =========================================================================
    // json_to_schema tests
    // =========================================================================

    #[test]
    fn parse_primitive_string() {
        assert_eq!(
            json_to_schema(&json!("int"), None).expect("parse int"),
            AvroSchema::Int
        );
        assert_eq!(
            json_to_schema(&json!("string"), None).expect("parse string"),
            AvroSchema::String
        );
    }

    #[test]
    fn parse_named_reference_with_namespace() {
        let schema =
            json_to_schema(&json!("Foo"), Some("org.example")).expect("parse reference");
        assert_eq!(
            schema,
            AvroSchema::Reference {
                name: "Foo".to_string(),
                namespace: Some("org.example".to_string()),
                properties: IndexMap::new(),
            }
        );
    }

    #[test]
    fn parse_named_reference_already_qualified() {
        let schema =
            json_to_schema(&json!("com.other.Bar"), Some("org.example")).expect("parse fqn");
        assert_eq!(
            schema,
            AvroSchema::Reference {
                name: "Bar".to_string(),
                namespace: Some("com.other".to_string()),
                properties: IndexMap::new(),
            }
        );
    }

    #[test]
    fn parse_union_array() {
        let schema = json_to_schema(&json!(["null", "string"]), None).expect("parse union");
        assert_eq!(
            schema,
            AvroSchema::Union {
                types: vec![AvroSchema::Null, AvroSchema::String],
                is_nullable_type: false,
            }
        );
    }

    #[test]
    fn parse_record_object() {
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "Event",
                "fields": [
                    {"name": "id", "type": "long"},
                    {"name": "data", "type": "string"}
                ]
            }),
            None,
        )
        .expect("parse record");

        match schema {
            AvroSchema::Record {
                name, fields, ..
            } => {
                assert_eq!(name, "Event");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name, "id");
                assert_eq!(fields[1].name, "data");
            }
            other => panic!("expected Record, got {other:?}"),
        }
    }

    #[test]
    fn parse_enum_object() {
        let schema = json_to_schema(
            &json!({
                "type": "enum",
                "name": "Suit",
                "symbols": ["HEARTS", "DIAMONDS", "CLUBS", "SPADES"]
            }),
            None,
        )
        .expect("parse enum");

        match schema {
            AvroSchema::Enum {
                name, symbols, ..
            } => {
                assert_eq!(name, "Suit");
                assert_eq!(symbols.len(), 4);
            }
            other => panic!("expected Enum, got {other:?}"),
        }
    }

    #[test]
    fn parse_fixed_object() {
        let schema = json_to_schema(
            &json!({"type": "fixed", "name": "MD5", "size": 16}),
            None,
        )
        .expect("parse fixed");

        match schema {
            AvroSchema::Fixed { name, size, .. } => {
                assert_eq!(name, "MD5");
                assert_eq!(size, 16);
            }
            other => panic!("expected Fixed, got {other:?}"),
        }
    }

    #[test]
    fn parse_logical_type_date() {
        let schema = json_to_schema(
            &json!({"type": "int", "logicalType": "date"}),
            None,
        )
        .expect("parse date");

        assert_eq!(
            schema,
            AvroSchema::Logical {
                logical_type: LogicalType::Date,
                properties: IndexMap::new(),
            }
        );
    }

    #[test]
    fn parse_logical_type_decimal() {
        let schema = json_to_schema(
            &json!({"type": "bytes", "logicalType": "decimal", "precision": 10, "scale": 2}),
            None,
        )
        .expect("parse decimal");

        assert_eq!(
            schema,
            AvroSchema::Logical {
                logical_type: LogicalType::Decimal {
                    precision: 10,
                    scale: 2,
                },
                properties: IndexMap::new(),
            }
        );
    }

    #[test]
    fn parse_array_schema() {
        let schema = json_to_schema(
            &json!({"type": "array", "items": "string"}),
            None,
        )
        .expect("parse array");

        match schema {
            AvroSchema::Array { items, .. } => {
                assert_eq!(*items, AvroSchema::String);
            }
            other => panic!("expected Array, got {other:?}"),
        }
    }

    #[test]
    fn parse_map_schema() {
        let schema = json_to_schema(
            &json!({"type": "map", "values": "long"}),
            None,
        )
        .expect("parse map");

        match schema {
            AvroSchema::Map { values, .. } => {
                assert_eq!(*values, AvroSchema::Long);
            }
            other => panic!("expected Map, got {other:?}"),
        }
    }

    #[test]
    fn parse_error_record() {
        let schema = json_to_schema(
            &json!({
                "type": "error",
                "name": "NotFound",
                "fields": []
            }),
            None,
        )
        .expect("parse error record");

        match schema {
            AvroSchema::Record {
                name, is_error, ..
            } => {
                assert_eq!(name, "NotFound");
                assert!(is_error);
            }
            other => panic!("expected Record with is_error=true, got {other:?}"),
        }
    }

    // =========================================================================
    // json_to_message tests
    // =========================================================================

    #[test]
    fn parse_simple_message() {
        let msg = json_to_message(
            &json!({
                "request": [{"name": "greeting", "type": "string"}],
                "response": "string"
            }),
            None,
        )
        .expect("parse message");

        assert_eq!(msg.request.len(), 1);
        assert_eq!(msg.request[0].name, "greeting");
        assert_eq!(msg.response, AvroSchema::String);
        assert!(!msg.one_way);
        assert!(msg.errors.is_none());
    }

    #[test]
    fn parse_one_way_message() {
        let msg = json_to_message(
            &json!({
                "request": [],
                "response": "null",
                "one-way": true
            }),
            None,
        )
        .expect("parse one-way message");

        assert!(msg.one_way);
    }

    #[test]
    fn parse_message_with_errors() {
        let msg = json_to_message(
            &json!({
                "request": [],
                "response": "string",
                "errors": ["string"]
            }),
            None,
        )
        .expect("parse message with errors");

        assert_eq!(msg.errors, Some(vec![AvroSchema::String]));
    }

    // =========================================================================
    // Field parsing tests
    // =========================================================================

    #[test]
    fn parse_field_with_order_and_aliases() {
        let field = json_to_field(
            &json!({
                "name": "ts",
                "type": "long",
                "order": "descending",
                "aliases": ["timestamp"]
            }),
            None,
        )
        .expect("parse field");

        assert_eq!(field.name, "ts");
        assert_eq!(field.order, Some(FieldOrder::Descending));
        assert_eq!(field.aliases, vec!["timestamp".to_string()]);
    }

    #[test]
    fn parse_field_with_default() {
        let field = json_to_field(
            &json!({
                "name": "count",
                "type": "int",
                "default": 42
            }),
            None,
        )
        .expect("parse field with default");

        assert_eq!(field.default, Some(json!(42)));
    }

    // =========================================================================
    // primitive_from_str tests
    // =========================================================================

    #[test]
    fn primitive_from_str_returns_error_on_unknown_type() {
        let result = primitive_from_str("timestamp");
        assert!(result.is_err(), "unknown type should produce an error");
        let err = result.unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("unknown primitive type: timestamp"),
            "error message should mention the unknown type, got: {msg}"
        );
    }

    // =========================================================================
    // Annotated primitive tests (issues #3 and #4)
    // =========================================================================

    #[test]
    fn unknown_logical_type_preserved_as_annotated_primitive() {
        let schema = json_to_schema(
            &json!({"type": "string", "logicalType": "my-custom-type", "extra": 42}),
            None,
        )
        .expect("parse unknown logical type");

        match schema {
            AvroSchema::AnnotatedPrimitive { kind, properties } => {
                assert_eq!(kind, PrimitiveType::String);
                assert_eq!(
                    properties.get("logicalType"),
                    Some(&json!("my-custom-type"))
                );
                assert_eq!(properties.get("extra"), Some(&json!(42)));
            }
            other => panic!("expected AnnotatedPrimitive, got {other:?}"),
        }
    }

    #[test]
    fn custom_properties_on_primitive_without_logical_type_preserved() {
        let schema = json_to_schema(
            &json!({"type": "int", "foo.bar": "baz"}),
            None,
        )
        .expect("parse primitive with custom property");

        match schema {
            AvroSchema::AnnotatedPrimitive { kind, properties } => {
                assert_eq!(kind, PrimitiveType::Int);
                assert_eq!(properties.get("foo.bar"), Some(&json!("baz")));
            }
            other => panic!("expected AnnotatedPrimitive, got {other:?}"),
        }
    }

    #[test]
    fn bare_primitive_object_without_extra_properties_stays_primitive() {
        let schema = json_to_schema(
            &json!({"type": "long"}),
            None,
        )
        .expect("parse bare primitive object");

        assert_eq!(schema, AvroSchema::Long);
    }

    // =========================================================================
    // Qualified name splitting tests (issue #19)
    // =========================================================================

    #[test]
    fn split_qualified_name_simple() {
        let (name, ns) = split_qualified_name("Baz");
        assert_eq!(name, "Baz");
        assert_eq!(ns, None);
    }

    #[test]
    fn split_qualified_name_fully_qualified() {
        let (name, ns) = split_qualified_name("ns.other.schema.Baz");
        assert_eq!(name, "Baz");
        assert_eq!(ns, Some("ns.other.schema".to_string()));
    }

    #[test]
    fn split_qualified_name_single_dot() {
        let (name, ns) = split_qualified_name("org.Foo");
        assert_eq!(name, "Foo");
        assert_eq!(ns, Some("org".to_string()));
    }

    #[test]
    fn record_with_qualified_name_splits_into_name_and_namespace() {
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "ns.other.schema.Baz",
                "fields": []
            }),
            None,
        )
        .expect("parse record with qualified name");

        match schema {
            AvroSchema::Record {
                name, namespace, ..
            } => {
                assert_eq!(name, "Baz");
                assert_eq!(namespace, Some("ns.other.schema".to_string()));
            }
            other => panic!("expected Record, got {other:?}"),
        }
    }

    #[test]
    fn record_with_qualified_name_explicit_namespace_takes_precedence() {
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "ns.other.schema.Baz",
                "namespace": "explicit.ns",
                "fields": []
            }),
            None,
        )
        .expect("parse record with explicit namespace");

        match schema {
            AvroSchema::Record {
                name, namespace, ..
            } => {
                assert_eq!(name, "Baz");
                assert_eq!(namespace, Some("explicit.ns".to_string()));
            }
            other => panic!("expected Record, got {other:?}"),
        }
    }

    #[test]
    fn enum_with_qualified_name_splits() {
        let schema = json_to_schema(
            &json!({
                "type": "enum",
                "name": "org.example.Suit",
                "symbols": ["HEARTS"]
            }),
            None,
        )
        .expect("parse enum with qualified name");

        match schema {
            AvroSchema::Enum {
                name, namespace, ..
            } => {
                assert_eq!(name, "Suit");
                assert_eq!(namespace, Some("org.example".to_string()));
            }
            other => panic!("expected Enum, got {other:?}"),
        }
    }

    #[test]
    fn fixed_with_qualified_name_splits() {
        let schema = json_to_schema(
            &json!({
                "type": "fixed",
                "name": "com.example.MD5",
                "size": 16
            }),
            None,
        )
        .expect("parse fixed with qualified name");

        match schema {
            AvroSchema::Fixed {
                name, namespace, ..
            } => {
                assert_eq!(name, "MD5");
                assert_eq!(namespace, Some("com.example".to_string()));
            }
            other => panic!("expected Fixed, got {other:?}"),
        }
    }

    #[test]
    fn record_simple_name_falls_back_to_default_namespace() {
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "Event",
                "fields": []
            }),
            Some("org.default"),
        )
        .expect("parse record with default namespace");

        match schema {
            AvroSchema::Record {
                name, namespace, ..
            } => {
                assert_eq!(name, "Event");
                assert_eq!(namespace, Some("org.default".to_string()));
            }
            other => panic!("expected Record, got {other:?}"),
        }
    }

    #[test]
    fn record_qualified_name_overrides_default_namespace() {
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "org.foo.Foo",
                "fields": []
            }),
            Some("org.default"),
        )
        .expect("parse record with qualified name overriding default");

        match schema {
            AvroSchema::Record {
                name, namespace, ..
            } => {
                assert_eq!(name, "Foo");
                assert_eq!(namespace, Some("org.foo".to_string()));
            }
            other => panic!("expected Record, got {other:?}"),
        }
    }

    // =========================================================================
    // register_all_named_types tests (issue 6fbdd004)
    // =========================================================================

    #[test]
    fn register_nested_record_in_field() {
        // A record with an inline nested record in one of its fields. Both the
        // outer and inner records should be registered.
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "Outer",
                "namespace": "test.nested",
                "fields": [{
                    "name": "inner",
                    "type": {
                        "type": "record",
                        "name": "Inner",
                        "fields": [{"name": "value", "type": "int"}]
                    }
                }]
            }),
            None,
        )
        .expect("parse nested record");

        let mut registry = SchemaRegistry::new();
        register_all_named_types(&schema, &mut registry);

        assert!(registry.contains("test.nested.Outer"), "outer record should be registered");
        assert!(registry.contains("test.nested.Inner"), "nested record should be registered");
    }

    #[test]
    fn register_nested_enum_in_field() {
        // A record with an inline enum in one of its fields.
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "Container",
                "namespace": "test.nested",
                "fields": [{
                    "name": "status",
                    "type": {
                        "type": "enum",
                        "name": "Status",
                        "symbols": ["ACTIVE", "INACTIVE"]
                    }
                }]
            }),
            None,
        )
        .expect("parse record with nested enum");

        let mut registry = SchemaRegistry::new();
        register_all_named_types(&schema, &mut registry);

        assert!(registry.contains("test.nested.Container"));
        assert!(registry.contains("test.nested.Status"), "nested enum should be registered");
    }

    #[test]
    fn register_nested_fixed_in_field() {
        // A record with an inline fixed in one of its fields.
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "Container",
                "namespace": "test.nested",
                "fields": [{
                    "name": "hash",
                    "type": {
                        "type": "fixed",
                        "name": "MD5",
                        "size": 16
                    }
                }]
            }),
            None,
        )
        .expect("parse record with nested fixed");

        let mut registry = SchemaRegistry::new();
        register_all_named_types(&schema, &mut registry);

        assert!(registry.contains("test.nested.Container"));
        assert!(registry.contains("test.nested.MD5"), "nested fixed should be registered");
    }

    #[test]
    fn register_nested_record_in_union() {
        // A record with a union field containing an inline record.
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "Wrapper",
                "namespace": "test.nested",
                "fields": [{
                    "name": "payload",
                    "type": ["null", {
                        "type": "record",
                        "name": "Payload",
                        "fields": [{"name": "data", "type": "bytes"}]
                    }]
                }]
            }),
            None,
        )
        .expect("parse record with nested record in union");

        let mut registry = SchemaRegistry::new();
        register_all_named_types(&schema, &mut registry);

        assert!(registry.contains("test.nested.Wrapper"));
        assert!(registry.contains("test.nested.Payload"), "nested record in union should be registered");
    }

    #[test]
    fn register_nested_record_in_array_items() {
        // A record with an array field whose items are an inline record.
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "Collection",
                "namespace": "test.nested",
                "fields": [{
                    "name": "entries",
                    "type": {
                        "type": "array",
                        "items": {
                            "type": "record",
                            "name": "Entry",
                            "fields": [{"name": "key", "type": "string"}]
                        }
                    }
                }]
            }),
            None,
        )
        .expect("parse record with nested record in array");

        let mut registry = SchemaRegistry::new();
        register_all_named_types(&schema, &mut registry);

        assert!(registry.contains("test.nested.Collection"));
        assert!(registry.contains("test.nested.Entry"), "nested record in array items should be registered");
    }

    #[test]
    fn register_nested_record_in_map_values() {
        // A record with a map field whose values are an inline record.
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "Lookup",
                "namespace": "test.nested",
                "fields": [{
                    "name": "entries",
                    "type": {
                        "type": "map",
                        "values": {
                            "type": "record",
                            "name": "MapEntry",
                            "fields": [{"name": "value", "type": "int"}]
                        }
                    }
                }]
            }),
            None,
        )
        .expect("parse record with nested record in map");

        let mut registry = SchemaRegistry::new();
        register_all_named_types(&schema, &mut registry);

        assert!(registry.contains("test.nested.Lookup"));
        assert!(registry.contains("test.nested.MapEntry"), "nested record in map values should be registered");
    }

    #[test]
    fn register_deeply_nested_types() {
        // A record with a field containing a union -> array -> record chain.
        // All named types at every level should be registered.
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "Root",
                "namespace": "test.deep",
                "fields": [{
                    "name": "data",
                    "type": ["null", {
                        "type": "array",
                        "items": {
                            "type": "record",
                            "name": "Leaf",
                            "fields": [{
                                "name": "tag",
                                "type": {
                                    "type": "enum",
                                    "name": "Tag",
                                    "symbols": ["A", "B"]
                                }
                            }]
                        }
                    }]
                }]
            }),
            None,
        )
        .expect("parse deeply nested schema");

        let mut registry = SchemaRegistry::new();
        register_all_named_types(&schema, &mut registry);

        assert!(registry.contains("test.deep.Root"));
        assert!(registry.contains("test.deep.Leaf"), "deeply nested record should be registered");
        assert!(registry.contains("test.deep.Tag"), "deeply nested enum should be registered");
    }

    #[test]
    fn register_non_named_schema_is_noop() {
        // A bare primitive or array schema should not cause errors, even though
        // there are no named types to register.
        let schema = json_to_schema(&json!("int"), None).expect("parse int");
        let mut registry = SchemaRegistry::new();
        register_all_named_types(&schema, &mut registry);
        assert_eq!(registry.into_schemas().len(), 0);
    }
}
