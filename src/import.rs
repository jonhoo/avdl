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

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::model::protocol::Message;
use crate::model::schema::{AvroSchema, FieldOrder, LogicalType, PrimitiveType};
use crate::resolve::SchemaRegistry;
use miette::Result;

/// Parse JSON with C-style comment stripping (`//` and `/* */`).
///
/// Avro's Java implementation uses Jackson with `ALLOW_COMMENTS`, so `.avpr` and
/// `.avsc` files in the wild may contain comments that standard JSON parsers reject.
fn parse_json_with_comments(input: &str) -> std::result::Result<Value, serde_json::Error> {
    serde_json::from_reader(
        json_comments::CommentSettings::c_style().strip_comments(input.as_bytes()),
    )
}

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
                miette::miette!(
                    "canonicalize import path `{import_file}` relative to `{}`: {e}",
                    current_dir.display()
                )
            });
        }

        // Try each import search directory.
        for dir in &self.import_dirs {
            let candidate = dir.join(import_file);
            if candidate.exists() {
                return candidate.canonicalize().map_err(|e| {
                    miette::miette!(
                        "canonicalize import path `{import_file}` in import dir `{}`: {e}",
                        dir.display()
                    )
                });
            }
        }

        Err(miette::miette!(
            "import not found: {import_file} (searched relative to {} and {} import dir(s))",
            current_dir.display(),
            self.import_dirs.len()
        ))
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
// Schema Flattening and Registration for Imports
// ==============================================================================
//
// When importing `.avsc` or `.avpr` files, named types (record, enum, fixed)
// can be nested arbitrarily deep inside record fields, union branches, array
// items, or map values. Java's `Schema.parse()` recursively registers all
// nested named types in a flat `Names` map, and its serialization emits name
// references for already-known types. We replicate this by *flattening*
// imported schemas: extracting nested named types into a flat list and
// replacing inline definitions with `Reference` nodes in the parent schema.
//
// This ensures that:
// 1. Each nested named type appears as a separate top-level entry in the
//    protocol's `types` array (inner types before the outer types that
//    reference them).
// 2. The parent schema references nested types by name string instead of
//    containing inline definitions — matching Java's JSON output.

/// Flatten a schema by extracting all nested named types into a separate list.
///
/// Returns a list of `(nested_type, modified_schema)` pairs where:
/// - The returned `Vec` contains all named types found in the tree, in
///   depth-first order (inner types before outer types that contain them).
/// - Each named type in the returned list has *itself* been flattened: its
///   fields reference further-nested types by name, not inline.
/// - The modified schema has inline named type definitions replaced with
///   `Reference` nodes.
///
/// For a non-named top-level schema (e.g., an array of records), only the
/// nested named types are extracted — the modified top-level schema is
/// returned via the second element.
fn flatten_schema(schema: AvroSchema) -> (Vec<AvroSchema>, AvroSchema) {
    let mut collected = Vec::new();
    let flattened = flatten_schema_inner(schema, &mut collected);
    (collected, flattened)
}

/// Inner recursive helper for `flatten_schema`. Walks the schema tree,
/// collects named types into `collected`, and returns the schema with
/// inline named type definitions replaced by `Reference` nodes.
fn flatten_schema_inner(schema: AvroSchema, collected: &mut Vec<AvroSchema>) -> AvroSchema {
    match schema {
        AvroSchema::Record {
            name,
            namespace,
            doc,
            fields,
            is_error,
            aliases,
            properties,
        } => {
            // First, flatten each field's schema so that any nested named types
            // within fields are extracted and replaced with references.
            let flattened_fields: Vec<_> = fields
                .into_iter()
                .map(|field| {
                    let flattened_field_schema = flatten_schema_inner(field.schema, collected);
                    crate::model::schema::Field {
                        schema: flattened_field_schema,
                        ..field
                    }
                })
                .collect();

            // Build the flattened record (fields now contain references
            // instead of inline named types).
            let flattened_record = AvroSchema::Record {
                name: name.clone(),
                namespace: namespace.clone(),
                doc,
                fields: flattened_fields,
                is_error,
                aliases,
                properties,
            };

            // Add this record to the collected list.
            collected.push(flattened_record.clone());

            // Return a Reference to replace the inline definition in the parent.
            AvroSchema::Reference {
                name,
                namespace,
                properties: HashMap::new(),
                span: None,
            }
        }

        AvroSchema::Enum {
            ref name,
            ref namespace,
            ..
        } => {
            // Enums have no nested types to flatten, but they themselves need
            // to be collected and replaced with a reference.
            let reference = AvroSchema::Reference {
                name: name.clone(),
                namespace: namespace.clone(),
                properties: HashMap::new(),
                span: None,
            };
            collected.push(schema);
            reference
        }

        AvroSchema::Fixed {
            ref name,
            ref namespace,
            ..
        } => {
            let reference = AvroSchema::Reference {
                name: name.clone(),
                namespace: namespace.clone(),
                properties: HashMap::new(),
                span: None,
            };
            collected.push(schema);
            reference
        }

        AvroSchema::Array { items, properties } => {
            let flattened_items = flatten_schema_inner(*items, collected);
            AvroSchema::Array {
                items: Box::new(flattened_items),
                properties,
            }
        }

        AvroSchema::Map { values, properties } => {
            let flattened_values = flatten_schema_inner(*values, collected);
            AvroSchema::Map {
                values: Box::new(flattened_values),
                properties,
            }
        }

        AvroSchema::Union {
            types,
            is_nullable_type,
        } => {
            let flattened_types: Vec<_> = types
                .into_iter()
                .map(|t| flatten_schema_inner(t, collected))
                .collect();
            AvroSchema::Union {
                types: flattened_types,
                is_nullable_type,
            }
        }

        // Primitives, logical types, annotated primitives, and references
        // contain no nested named type definitions to extract.
        other => other,
    }
}

/// Flatten a schema and register all extracted named types in the registry.
///
/// The flattened types are registered in depth-first order (inner types before
/// outer types), matching Java's behavior where `Schema.parse()` recursively
/// registers nested types as it encounters them.
///
/// Duplicate registrations are silently ignored (the first definition wins),
/// matching Java's behavior for imports.
fn flatten_and_register(schema: AvroSchema, registry: &mut SchemaRegistry) {
    let (types, _top_level) = flatten_schema(schema);
    for t in types {
        let _ = registry.register(t);
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
) -> Result<HashMap<String, Message>> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| miette::miette!("read protocol file `{}`: {e}", path.display()))?;
    let json: Value = parse_json_with_comments(&content)
        .map_err(|e| miette::miette!("invalid JSON in {}: {e}", path.display()))?;

    let default_namespace = json.get("namespace").and_then(|n| n.as_str());
    let mut messages = HashMap::new();

    // Extract types from the protocol JSON and register them. Schemas are
    // flattened so that nested named types (records, enums, fixed) within
    // record fields, union branches, etc. are promoted to separate top-level
    // entries in the registry and replaced with Reference nodes.
    if let Some(types) = json.get("types").and_then(|t| t.as_array()) {
        for (i, type_json) in types.iter().enumerate() {
            let schema = json_to_schema(type_json, default_namespace).map_err(|e| {
                miette::miette!(
                    "parse type at index {i} in protocol `{}`: {e}",
                    path.display()
                )
            })?;
            flatten_and_register(schema, registry);
        }
    }

    // Extract messages.
    if let Some(msgs) = json.get("messages").and_then(|m| m.as_object()) {
        for (name, msg_json) in msgs {
            let message = json_to_message(msg_json, default_namespace).map_err(|e| {
                miette::miette!(
                    "parse message `{name}` in protocol `{}`: {e}",
                    path.display()
                )
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
    let content = std::fs::read_to_string(path)
        .map_err(|e| miette::miette!("read schema file `{}`: {e}", path.display()))?;
    let json: Value = parse_json_with_comments(&content)
        .map_err(|e| miette::miette!("invalid JSON in {}: {e}", path.display()))?;

    let schema = json_to_schema(&json, None)
        .map_err(|e| miette::miette!("parse schema from `{}`: {e}", path.display()))?;
    flatten_and_register(schema, registry);

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

        _ => Err(miette::miette!("invalid schema JSON: {json}")),
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
            if let Some((ns, name)) = type_name.rsplit_once('.') {
                Ok(AvroSchema::Reference {
                    name: name.to_string(),
                    namespace: Some(ns.to_string()),
                    properties: HashMap::new(),
                    span: None,
                })
            } else {
                Ok(AvroSchema::Reference {
                    name: type_name.to_string(),
                    namespace: default_namespace.map(|s| s.to_string()),
                    properties: HashMap::new(),
                    span: None,
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
        .ok_or_else(|| miette::miette!("schema object missing 'type' field"))?;

    match type_str {
        "record" | "error" => parse_record(obj, type_str, default_namespace),
        "enum" => parse_enum(obj, default_namespace),
        "fixed" => parse_fixed(obj, default_namespace),
        "array" => parse_array(obj, default_namespace),
        "map" => parse_map(obj, default_namespace),

        // A primitive type with optional logical type or custom properties.
        prim @ ("null" | "boolean" | "int" | "long" | "float" | "double" | "bytes" | "string") => {
            parse_annotated_primitive(obj, prim, default_namespace)
        }

        other => Err(miette::miette!("unknown schema type: {other}")),
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
        (
            raw_name[pos + 1..].to_string(),
            Some(raw_name[..pos].to_string()),
        )
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
        .ok_or_else(|| miette::miette!("record missing 'name'"))?;
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
                    miette::miette!("parse field at index {i} of record `{name}`: {e}")
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
        .ok_or_else(|| miette::miette!("enum missing 'name'"))?;
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
        .ok_or_else(|| miette::miette!("fixed missing 'name'"))?;
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
        .ok_or_else(|| miette::miette!("fixed missing 'size'"))?;
    let size = u32::try_from(size_u64)
        .map_err(|_| miette::miette!("fixed size {size_u64} exceeds maximum ({})", u32::MAX))?;
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
        .ok_or_else(|| miette::miette!("array missing 'items'"))?;
    let items_schema = json_to_schema(items, default_namespace)
        .map_err(|e| miette::miette!("parse array items schema: {e}"))?;
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
        .ok_or_else(|| miette::miette!("map missing 'values'"))?;
    let values_schema = json_to_schema(values, default_namespace)
        .map_err(|e| miette::miette!("parse map values schema: {e}"))?;
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
                let precision_u64 = obj.get("precision").and_then(|p| p.as_u64()).unwrap_or(0);
                if precision_u64 < 1 {
                    miette::bail!("decimal precision must be >= 1");
                }
                let precision = u32::try_from(precision_u64)
                    .map_err(|_| miette::miette!("decimal precision too large"))?;
                let scale_u64 = obj.get("scale").and_then(|s| s.as_u64()).unwrap_or(0);
                let scale = u32::try_from(scale_u64)
                    .map_err(|_| miette::miette!("decimal scale too large"))?;
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

        let properties =
            collect_extra_properties(obj, &["type", "logicalType", "precision", "scale"]);
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
        other => Err(miette::miette!("unknown primitive type: {other}")),
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
        .ok_or_else(|| miette::miette!("field must be an object"))?;

    let name = obj
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| miette::miette!("field missing 'name'"))?
        .to_string();
    let type_json = obj
        .get("type")
        .ok_or_else(|| miette::miette!("field missing 'type'"))?;
    let schema = json_to_schema(type_json, default_namespace)
        .map_err(|e| miette::miette!("parse type for field `{name}`: {e}"))?;
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

    let properties =
        collect_extra_properties(obj, &["name", "type", "doc", "default", "order", "aliases"]);

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
        .ok_or_else(|| miette::miette!("message must be an object"))?;

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
                json_to_field(p, default_namespace)
                    .map_err(|e| miette::miette!("parse request parameter at index {i}: {e}"))
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        vec![]
    };

    let response = if let Some(resp) = obj.get("response") {
        json_to_schema(resp, default_namespace)
            .map_err(|e| miette::miette!("parse response type for message: {e}"))?
    } else {
        AvroSchema::Null
    };

    let errors = if let Some(Value::Array(errs)) = obj.get("errors") {
        Some(
            errs.iter()
                .enumerate()
                .map(|(i, e)| {
                    json_to_schema(e, default_namespace).map_err(|e| {
                        miette::miette!("parse error type at index {i} for message: {e}")
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        )
    } else {
        None
    };

    let properties =
        collect_extra_properties(obj, &["doc", "request", "response", "errors", "one-way"]);

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
) -> HashMap<String, Value> {
    let mut properties = HashMap::new();
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
    use pretty_assertions::assert_eq;
    use serde_json::json;

    // =========================================================================
    // ImportContext tests
    // =========================================================================

    #[test]
    fn mark_imported_returns_false_on_first_call() {
        let mut ctx = ImportContext::new(vec![]);
        let path = PathBuf::from("dummy/test.avdl");
        assert!(
            !ctx.mark_imported(&path),
            "first import should return false"
        );
    }

    #[test]
    fn mark_imported_returns_true_on_subsequent_calls() {
        let mut ctx = ImportContext::new(vec![]);
        let path = PathBuf::from("dummy/test.avdl");
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
        let schema = json_to_schema(&json!("Foo"), Some("org.example")).expect("parse reference");
        assert_eq!(
            schema,
            AvroSchema::Reference {
                name: "Foo".to_string(),
                namespace: Some("org.example".to_string()),
                properties: HashMap::new(),
                span: None,
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
                properties: HashMap::new(),
                span: None,
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
            AvroSchema::Record { name, fields, .. } => {
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
            AvroSchema::Enum { name, symbols, .. } => {
                assert_eq!(name, "Suit");
                assert_eq!(symbols.len(), 4);
            }
            other => panic!("expected Enum, got {other:?}"),
        }
    }

    #[test]
    fn parse_fixed_object() {
        let schema = json_to_schema(&json!({"type": "fixed", "name": "MD5", "size": 16}), None)
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
        let schema = json_to_schema(&json!({"type": "int", "logicalType": "date"}), None)
            .expect("parse date");

        assert_eq!(
            schema,
            AvroSchema::Logical {
                logical_type: LogicalType::Date,
                properties: HashMap::new(),
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
                properties: HashMap::new(),
            }
        );
    }

    #[test]
    fn parse_array_schema() {
        let schema = json_to_schema(&json!({"type": "array", "items": "string"}), None)
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
        let schema =
            json_to_schema(&json!({"type": "map", "values": "long"}), None).expect("parse map");

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
            AvroSchema::Record { name, is_error, .. } => {
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
        let schema = json_to_schema(&json!({"type": "int", "foo.bar": "baz"}), None)
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
        let schema =
            json_to_schema(&json!({"type": "long"}), None).expect("parse bare primitive object");

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
    // flatten_and_register tests (issues 6fbdd004, f812cf8e)
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
        flatten_and_register(schema, &mut registry);

        assert!(
            registry.contains("test.nested.Outer"),
            "outer record should be registered"
        );
        assert!(
            registry.contains("test.nested.Inner"),
            "nested record should be registered"
        );
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
        flatten_and_register(schema, &mut registry);

        assert!(registry.contains("test.nested.Container"));
        assert!(
            registry.contains("test.nested.Status"),
            "nested enum should be registered"
        );
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
        flatten_and_register(schema, &mut registry);

        assert!(registry.contains("test.nested.Container"));
        assert!(
            registry.contains("test.nested.MD5"),
            "nested fixed should be registered"
        );
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
        flatten_and_register(schema, &mut registry);

        assert!(registry.contains("test.nested.Wrapper"));
        assert!(
            registry.contains("test.nested.Payload"),
            "nested record in union should be registered"
        );
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
        flatten_and_register(schema, &mut registry);

        assert!(registry.contains("test.nested.Collection"));
        assert!(
            registry.contains("test.nested.Entry"),
            "nested record in array items should be registered"
        );
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
        flatten_and_register(schema, &mut registry);

        assert!(registry.contains("test.nested.Lookup"));
        assert!(
            registry.contains("test.nested.MapEntry"),
            "nested record in map values should be registered"
        );
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
        flatten_and_register(schema, &mut registry);

        assert!(registry.contains("test.deep.Root"));
        assert!(
            registry.contains("test.deep.Leaf"),
            "deeply nested record should be registered"
        );
        assert!(
            registry.contains("test.deep.Tag"),
            "deeply nested enum should be registered"
        );
    }

    #[test]
    fn register_non_named_schema_is_noop() {
        // A bare primitive or array schema should not cause errors, even though
        // there are no named types to register.
        let schema = json_to_schema(&json!("int"), None).expect("parse int");
        let mut registry = SchemaRegistry::new();
        flatten_and_register(schema, &mut registry);
        assert_eq!(registry.into_schemas().len(), 0);
    }

    // =========================================================================
    // flatten_schema tests (issue f812cf8e)
    // =========================================================================

    #[test]
    fn flatten_replaces_nested_record_with_reference() {
        // After flattening, the outer record's field should contain a Reference
        // to the inner record instead of an inline definition.
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "Outer",
                "namespace": "test",
                "fields": [{
                    "name": "inner",
                    "type": {
                        "type": "record",
                        "name": "Inner",
                        "fields": [{"name": "x", "type": "int"}]
                    }
                }]
            }),
            None,
        )
        .expect("parse nested record");

        let (collected, top_level) = flatten_schema(schema);

        // The top-level schema should be a Reference to Outer (since Outer
        // itself is a named type that gets collected).
        assert!(
            matches!(&top_level, AvroSchema::Reference { name, .. } if name == "Outer"),
            "top-level should be a Reference to Outer, got {top_level:?}"
        );

        // Two types should be collected: Inner first (depth-first), then Outer.
        assert_eq!(collected.len(), 2, "should collect Inner and Outer");

        let inner = &collected[0];
        assert!(
            matches!(inner, AvroSchema::Enum { name, .. } | AvroSchema::Record { name, .. } if name == "Inner"),
            "first collected type should be Inner, got {inner:?}"
        );

        let outer = &collected[1];
        match outer {
            AvroSchema::Record { name, fields, .. } => {
                assert_eq!(name, "Outer");
                // The field's schema should now be a Reference, not an inline Record.
                assert!(
                    matches!(&fields[0].schema, AvroSchema::Reference { name, .. } if name == "Inner"),
                    "Outer's 'inner' field should be a Reference to Inner, got {:?}",
                    fields[0].schema
                );
            }
            other => panic!("expected Record for Outer, got {other:?}"),
        }
    }

    #[test]
    fn flatten_deeply_nested_produces_correct_order() {
        // Level1 -> Level2 -> Level3 -> Tag (enum). After flattening, the
        // collected order should be depth-first: Tag, Level3, Level2, Level1.
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "Level1",
                "namespace": "test",
                "fields": [{
                    "name": "l2",
                    "type": {
                        "type": "record",
                        "name": "Level2",
                        "fields": [{
                            "name": "l3",
                            "type": {
                                "type": "record",
                                "name": "Level3",
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
                    }
                }]
            }),
            None,
        )
        .expect("parse deeply nested schema");

        let (collected, _top_level) = flatten_schema(schema);
        let names: Vec<_> = collected
            .iter()
            .filter_map(|s| s.name().map(str::to_string))
            .collect();
        assert_eq!(
            names,
            vec!["Tag", "Level3", "Level2", "Level1"],
            "types should be collected depth-first (innermost first)"
        );
    }

    #[test]
    fn flatten_nested_in_union_replaces_with_reference() {
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "Container",
                "namespace": "test",
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

        let (collected, _top_level) = flatten_schema(schema);

        // Find Container in collected types and check its union field.
        let container = collected
            .iter()
            .find(|s| s.name() == Some("Container"))
            .expect("Container should be in collected types");
        match container {
            AvroSchema::Record { fields, .. } => match &fields[0].schema {
                AvroSchema::Union { types, .. } => {
                    assert!(
                        matches!(&types[1], AvroSchema::Reference { name, .. } if name == "Payload"),
                        "union branch should be a Reference to Payload, got {:?}",
                        types[1]
                    );
                }
                other => panic!("expected Union, got {other:?}"),
            },
            other => panic!("expected Record, got {other:?}"),
        }
    }

    #[test]
    fn flatten_nested_in_array_replaces_with_reference() {
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "Collection",
                "namespace": "test",
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

        let (collected, _) = flatten_schema(schema);
        let collection = collected
            .iter()
            .find(|s| s.name() == Some("Collection"))
            .expect("Collection should be collected");
        match collection {
            AvroSchema::Record { fields, .. } => match &fields[0].schema {
                AvroSchema::Array { items, .. } => {
                    assert!(
                        matches!(items.as_ref(), AvroSchema::Reference { name, .. } if name == "Entry"),
                        "array items should be a Reference to Entry, got {:?}",
                        items
                    );
                }
                other => panic!("expected Array, got {other:?}"),
            },
            other => panic!("expected Record, got {other:?}"),
        }
    }

    #[test]
    fn flatten_nested_in_map_replaces_with_reference() {
        let schema = json_to_schema(
            &json!({
                "type": "record",
                "name": "Lookup",
                "namespace": "test",
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

        let (collected, _) = flatten_schema(schema);
        let lookup = collected
            .iter()
            .find(|s| s.name() == Some("Lookup"))
            .expect("Lookup should be collected");
        match lookup {
            AvroSchema::Record { fields, .. } => match &fields[0].schema {
                AvroSchema::Map { values, .. } => {
                    assert!(
                        matches!(values.as_ref(), AvroSchema::Reference { name, .. } if name == "MapEntry"),
                        "map values should be a Reference to MapEntry, got {:?}",
                        values
                    );
                }
                other => panic!("expected Map, got {other:?}"),
            },
            other => panic!("expected Record, got {other:?}"),
        }
    }

    #[test]
    fn flatten_enum_replaces_with_reference() {
        // A standalone enum should be collected and replaced with a reference.
        let schema = json_to_schema(
            &json!({
                "type": "enum",
                "name": "Color",
                "namespace": "test",
                "symbols": ["RED", "GREEN", "BLUE"]
            }),
            None,
        )
        .expect("parse enum");

        let (collected, top_level) = flatten_schema(schema);
        assert_eq!(collected.len(), 1);
        assert!(
            matches!(&collected[0], AvroSchema::Enum { name, .. } if name == "Color"),
            "collected should contain the Color enum"
        );
        assert!(
            matches!(&top_level, AvroSchema::Reference { name, .. } if name == "Color"),
            "top-level should be a Reference to Color"
        );
    }

    #[test]
    fn flatten_primitive_is_unchanged() {
        // Primitives have nothing to flatten; they pass through unchanged.
        let schema = AvroSchema::Int;
        let (collected, result) = flatten_schema(schema);
        assert!(collected.is_empty());
        assert_eq!(result, AvroSchema::Int);
    }

    // =========================================================================
    // JSON comment stripping tests
    // =========================================================================

    // Tests below use the module-level `parse_json_with_comments` directly.

    #[test]
    fn strip_line_comment() {
        let input = r#"{
            // this is a line comment
            "type": "string"
        }"#;
        let value = parse_json_with_comments(input).expect("should parse with line comment");
        assert_eq!(value["type"], "string");
    }

    #[test]
    fn strip_block_comment() {
        let input = r#"/* license header
         * Copyright 2024
         */
        {"type": "int"}"#;
        let value = parse_json_with_comments(input).expect("should parse with block comment");
        assert_eq!(value["type"], "int");
    }

    #[test]
    fn preserve_strings_containing_comment_syntax() {
        let input = r#"{"type": "string", "doc": "use // for comments or /* block */"}"#;
        let value = parse_json_with_comments(input).expect("should preserve comment-like strings");
        assert_eq!(value["doc"], "use // for comments or /* block */");
    }

    #[test]
    fn hash_comments_are_not_stripped() {
        // Jackson's ALLOW_COMMENTS does not enable hash comments; neither
        // should we. `CommentSettings::c_style()` only strips `//` and `/* */`.
        let input = "# hash comment\n{\"type\": \"int\"}";
        let result = parse_json_with_comments(input);
        assert!(result.is_err(), "hash comments should cause a parse error");
    }

    #[test]
    fn strip_comments_from_avsc_schema() {
        // End-to-end: a realistic .avsc file with license header and inline
        // comments, parsed through the same path as `import_schema`.
        let input = r#"/*
         * Licensed under the Apache License, Version 2.0
         */
        {
            "type": "record",
            "name": "Event",
            "namespace": "com.example",
            // Primary fields
            "fields": [
                {"name": "id", "type": "long"},  // unique identifier
                {"name": "data", "type": "string"}
            ]
        }"#;

        let value = parse_json_with_comments(input).expect("should parse commented .avsc");
        let schema = json_to_schema(&value, None).expect("should convert to schema");
        match schema {
            AvroSchema::Record {
                name,
                namespace,
                fields,
                ..
            } => {
                assert_eq!(name, "Event");
                assert_eq!(namespace, Some("com.example".to_string()));
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name, "id");
                assert_eq!(fields[1].name, "data");
            }
            other => panic!("expected Record, got {other:?}"),
        }
    }
}
