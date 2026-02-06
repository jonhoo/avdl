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

use std::io;

use indexmap::IndexMap;
use indexmap::IndexSet;
use serde::Serialize;
use serde_json::{Map, Value};

use super::protocol::{Message, Protocol};
use super::schema::{AvroSchema, Field, FieldOrder, LogicalType};

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
            if namespace.as_deref() != enclosing_namespace
                && let Some(ns) = namespace
            {
                obj.insert("namespace".to_string(), Value::String(ns.clone()));
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
            if namespace.as_deref() != enclosing_namespace
                && let Some(ns) = namespace
            {
                obj.insert("namespace".to_string(), Value::String(ns.clone()));
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
            if namespace.as_deref() != enclosing_namespace
                && let Some(ns) = namespace
            {
                obj.insert("namespace".to_string(), Value::String(ns.clone()));
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

// =============================================================================
// Java-Compatible JSON Serialization
// =============================================================================
//
// Java's Jackson library uses `Double.toString()` when serializing double values
// in JSON. Since JDK 12, `Double.toString()` uses scientific notation when
// abs(value) >= 1e7 or abs(value) < 1e-3 (for non-zero values). For example,
// `-1.0e12` is serialized as `-1.0E12` rather than `-1000000000000.0`.
//
// Rust's `serde_json` always uses the `ryu` crate's shortest decimal
// representation, which avoids scientific notation for these values. To match
// Java's output byte-for-byte, we use a custom `serde_json::ser::Formatter`
// that overrides `write_f64` to apply Java-style formatting.

/// Format an `f64` value the way Java's `Double.toString()` does.
///
/// Java uses scientific notation when `abs(value) >= 1e7` or when `abs(value) > 0`
/// and `abs(value) < 1e-3`. The significand always includes at least one digit
/// after the decimal point, and positive exponents have no `+` sign.
///
/// Examples:
/// - `-1.0e12` -> `"-1.0E12"`
/// - `1.0e-4`  -> `"1.0E-4"`
/// - `1.5`     -> `"1.5"` (uses default formatting)
/// - `0.0`     -> `"0.0"` (uses default formatting)
pub fn format_f64_like_java(val: f64) -> String {
    // NaN and infinity are handled elsewhere (as JSON strings), but guard
    // against them here for safety.
    if val.is_nan() || val.is_infinite() {
        return format!("{val}");
    }

    let abs = val.abs();
    let needs_scientific = (abs >= 1e7) || (abs > 0.0 && abs < 1e-3);

    if !needs_scientific {
        // For values in the normal range, use ryu's shortest representation
        // (matching serde_json's default behavior).
        return ryu::Buffer::new().format(val).to_string();
    }

    // Use Rust's {:E} formatter to get scientific notation, then adjust to
    // match Java's format. Rust produces e.g. "-1E12" or "1.23456E10", but
    // Java always includes a decimal point in the significand: "-1.0E12".
    let formatted = format!("{val:E}");

    // Find the 'E' separator to split significand and exponent.
    let e_pos = formatted
        .find('E')
        .expect("format {:E} always produces an 'E'");
    let (significand, exponent_part) = formatted.split_at(e_pos);

    // If the significand lacks a decimal point, insert ".0" before the E.
    // For example, "-1E12" becomes "-1.0E12".
    if significand.contains('.') {
        formatted
    } else {
        format!("{significand}.0{exponent_part}")
    }
}

/// A JSON formatter that wraps `serde_json::ser::PrettyFormatter` but overrides
/// `write_f64` to match Java's `Double.toString()` scientific notation behavior.
///
/// All other formatting (indentation, key ordering, etc.) is delegated to the
/// inner `PrettyFormatter`.
struct JavaPrettyFormatter<'a> {
    inner: serde_json::ser::PrettyFormatter<'a>,
}

impl<'a> JavaPrettyFormatter<'a> {
    fn new() -> Self {
        Self {
            inner: serde_json::ser::PrettyFormatter::new(),
        }
    }
}

impl serde_json::ser::Formatter for JavaPrettyFormatter<'_> {
    // =========================================================================
    // Override write_f64 to use Java-style scientific notation.
    // =========================================================================

    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        let formatted = format_f64_like_java(value);
        writer.write_all(formatted.as_bytes())
    }

    // =========================================================================
    // Delegate all indentation and structural formatting to PrettyFormatter.
    //
    // PrettyFormatter only overrides these methods from the Formatter trait;
    // all other methods (write_null, write_bool, write_i32, etc.) use the
    // default trait implementations, which are identical between our wrapper
    // and PrettyFormatter. We only need to delegate the methods that
    // PrettyFormatter actually overrides.
    // =========================================================================

    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.begin_array(writer)
    }

    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.end_array(writer)
    }

    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.begin_array_value(writer, first)
    }

    fn end_array_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.end_array_value(writer)
    }

    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.begin_object(writer)
    }

    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.end_object(writer)
    }

    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.begin_object_key(writer, first)
    }

    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.begin_object_value(writer)
    }

    fn end_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + io::Write,
    {
        self.inner.end_object_value(writer)
    }
}

/// Serialize a `serde_json::Value` to a pretty-printed JSON string using
/// Java-compatible float formatting.
///
/// This is a drop-in replacement for `serde_json::to_string_pretty` that
/// formats `f64` values to match Java's `Double.toString()` output, using
/// scientific notation for very large or very small values.
pub fn to_string_pretty_java(value: &Value) -> serde_json::Result<String> {
    let mut writer = Vec::with_capacity(128);
    let formatter = JavaPrettyFormatter::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut writer, formatter);
    value.serialize(&mut serializer)?;
    // Safety: serde_json only produces valid UTF-8.
    Ok(unsafe { String::from_utf8_unchecked(writer) })
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // format_f64_like_java: Scientific notation for large/small values
    // =========================================================================

    #[test]
    fn large_negative_value_uses_scientific_notation() {
        assert_eq!(format_f64_like_java(-1.0e12), "-1.0E12");
    }

    #[test]
    fn large_positive_value_uses_scientific_notation() {
        assert_eq!(format_f64_like_java(1.0e12), "1.0E12");
    }

    #[test]
    fn boundary_1e7_uses_scientific_notation() {
        assert_eq!(format_f64_like_java(1.0e7), "1.0E7");
    }

    #[test]
    fn just_below_1e7_uses_decimal() {
        // 9_999_999.0 is below 1e7, so it should use decimal notation.
        assert_eq!(format_f64_like_java(9_999_999.0), "9999999.0");
    }

    #[test]
    fn small_positive_value_uses_scientific_notation() {
        assert_eq!(format_f64_like_java(1.0e-4), "1.0E-4");
    }

    #[test]
    fn boundary_1e_minus_3_uses_decimal() {
        // 1e-3 is NOT less than 1e-3, so it should use decimal notation.
        assert_eq!(format_f64_like_java(1.0e-3), "0.001");
    }

    #[test]
    fn small_negative_value_uses_scientific_notation() {
        assert_eq!(format_f64_like_java(-7.89e-5), "-7.89E-5");
    }

    #[test]
    fn large_value_with_multiple_significant_digits() {
        assert_eq!(format_f64_like_java(1.23456e10), "1.23456E10");
    }

    #[test]
    fn very_large_value() {
        assert_eq!(format_f64_like_java(2.5e20), "2.5E20");
    }

    // =========================================================================
    // format_f64_like_java: Normal range values (no scientific notation)
    // =========================================================================

    #[test]
    fn zero_uses_decimal() {
        assert_eq!(format_f64_like_java(0.0), "0.0");
    }

    #[test]
    fn negative_zero_uses_decimal() {
        assert_eq!(format_f64_like_java(-0.0), "-0.0");
    }

    #[test]
    fn normal_positive_value() {
        assert_eq!(format_f64_like_java(1.5), "1.5");
    }

    #[test]
    fn normal_negative_value() {
        assert_eq!(format_f64_like_java(-3.14), "-3.14");
    }

    #[test]
    fn integer_like_float() {
        assert_eq!(format_f64_like_java(42.0), "42.0");
    }

    // =========================================================================
    // format_f64_like_java: Edge cases
    // =========================================================================

    #[test]
    fn nan_returns_nan_string() {
        assert_eq!(format_f64_like_java(f64::NAN), "NaN");
    }

    #[test]
    fn positive_infinity_returns_inf_string() {
        assert_eq!(format_f64_like_java(f64::INFINITY), "inf");
    }

    #[test]
    fn negative_infinity_returns_neg_inf_string() {
        assert_eq!(format_f64_like_java(f64::NEG_INFINITY), "-inf");
    }

    // =========================================================================
    // to_string_pretty_java: Integration with serde_json Value
    // =========================================================================

    #[test]
    fn pretty_java_formats_large_float_in_object() {
        let val = serde_json::json!({
            "default": -1000000000000.0_f64
        });
        let json_str = to_string_pretty_java(&val).expect("serialization succeeds");
        assert!(
            json_str.contains("-1.0E12"),
            "expected scientific notation, got: {json_str}"
        );
    }

    #[test]
    fn pretty_java_preserves_normal_float_formatting() {
        let val = serde_json::json!({
            "default": 0.0_f64
        });
        let json_str = to_string_pretty_java(&val).expect("serialization succeeds");
        assert!(
            json_str.contains("0.0"),
            "expected 0.0, got: {json_str}"
        );
    }

    #[test]
    fn pretty_java_preserves_integer_formatting() {
        let val = serde_json::json!({
            "size": 16
        });
        let json_str = to_string_pretty_java(&val).expect("serialization succeeds");
        assert!(
            json_str.contains("16"),
            "expected 16, got: {json_str}"
        );
    }
}
