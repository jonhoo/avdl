use indexmap::IndexMap;
use serde_json::Value;

/// Field sort order in Avro schemas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldOrder {
    Ascending,
    Descending,
    Ignore,
}

/// The primitive Avro type names, used with `AnnotatedPrimitive` to carry
/// properties on a primitive type that would otherwise be a bare string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveType {
    Null,
    Boolean,
    Int,
    Long,
    Float,
    Double,
    Bytes,
    String,
}

impl PrimitiveType {
    /// Return the Avro type name string for this primitive.
    pub fn as_str(&self) -> &'static str {
        match self {
            PrimitiveType::Null => "null",
            PrimitiveType::Boolean => "boolean",
            PrimitiveType::Int => "int",
            PrimitiveType::Long => "long",
            PrimitiveType::Float => "float",
            PrimitiveType::Double => "double",
            PrimitiveType::Bytes => "bytes",
            PrimitiveType::String => "string",
        }
    }
}

/// Avro logical types that overlay primitive types.
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalType {
    /// `date` -> int
    Date,
    /// `time-millis` -> int
    TimeMillis,
    /// `timestamp-millis` -> long
    TimestampMillis,
    /// `local-timestamp-millis` -> long
    LocalTimestampMillis,
    /// `uuid` -> string
    Uuid,
    /// `decimal` -> bytes, with precision and scale
    Decimal { precision: u32, scale: u32 },
}

/// An Avro schema.
///
/// We use our own domain model rather than depending on the `apache-avro` crate,
/// because we need full control over JSON serialization to match the Java Avro
/// tools output format exactly.
#[derive(Debug, Clone, PartialEq)]
pub enum AvroSchema {
    // =========================================================================
    // Primitives
    // =========================================================================
    Null,
    Boolean,
    Int,
    Long,
    Float,
    Double,
    Bytes,
    String,

    // =========================================================================
    // Named types
    // =========================================================================
    Record {
        name: std::string::String,
        namespace: Option<std::string::String>,
        doc: Option<std::string::String>,
        fields: Vec<Field>,
        is_error: bool,
        aliases: Vec<std::string::String>,
        properties: IndexMap<std::string::String, Value>,
    },
    Enum {
        name: std::string::String,
        namespace: Option<std::string::String>,
        doc: Option<std::string::String>,
        symbols: Vec<std::string::String>,
        default: Option<std::string::String>,
        aliases: Vec<std::string::String>,
        properties: IndexMap<std::string::String, Value>,
    },
    Fixed {
        name: std::string::String,
        namespace: Option<std::string::String>,
        doc: Option<std::string::String>,
        size: u32,
        aliases: Vec<std::string::String>,
        properties: IndexMap<std::string::String, Value>,
    },

    // =========================================================================
    // Complex types
    // =========================================================================
    Array {
        items: Box<AvroSchema>,
        properties: IndexMap<std::string::String, Value>,
    },
    Map {
        values: Box<AvroSchema>,
        properties: IndexMap<std::string::String, Value>,
    },
    Union {
        types: Vec<AvroSchema>,
        /// Internal flag: when true, this union was created by the `type?`
        /// nullable syntax and may need reordering based on the field default.
        is_nullable_type: bool,
    },

    // =========================================================================
    // Primitive with custom properties (e.g., `@foo.bar("baz") long`)
    // =========================================================================
    /// A primitive type annotated with custom properties. When serialized to
    /// JSON, this produces an object like `{"type": "long", "foo.bar": "baz"}`
    /// instead of the bare string `"long"`.
    AnnotatedPrimitive {
        kind: PrimitiveType,
        properties: IndexMap<std::string::String, Value>,
    },

    // =========================================================================
    // Logical types overlaying a primitive
    // =========================================================================
    Logical {
        logical_type: LogicalType,
        /// Extra properties on the underlying primitive (e.g., `@foo.bar("baz")` on a `long`).
        properties: IndexMap<std::string::String, Value>,
    },

    // =========================================================================
    // Forward reference placeholder, resolved after parsing completes.
    // =========================================================================
    Reference(std::string::String),
}

/// A field in a record schema.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: std::string::String,
    pub schema: AvroSchema,
    pub doc: Option<std::string::String>,
    pub default: Option<Value>,
    pub order: Option<FieldOrder>,
    pub aliases: Vec<std::string::String>,
    pub properties: IndexMap<std::string::String, Value>,
}

impl AvroSchema {
    /// Returns the full name of a named type (namespace.name), or `None` if not a named type.
    pub fn full_name(&self) -> Option<std::string::String> {
        match self {
            AvroSchema::Record {
                name, namespace, ..
            }
            | AvroSchema::Enum {
                name, namespace, ..
            }
            | AvroSchema::Fixed {
                name, namespace, ..
            } => Some(match namespace {
                Some(ns) => format!("{ns}.{name}"),
                None => name.clone(),
            }),
            _ => None,
        }
    }

    /// Returns the simple name of a named type, or `None` if not a named type.
    pub fn name(&self) -> Option<&str> {
        match self {
            AvroSchema::Record { name, .. }
            | AvroSchema::Enum { name, .. }
            | AvroSchema::Fixed { name, .. } => Some(name),
            _ => None,
        }
    }

    /// Returns true if this is a named type (record, enum, or fixed).
    pub fn is_named(&self) -> bool {
        matches!(
            self,
            AvroSchema::Record { .. } | AvroSchema::Enum { .. } | AvroSchema::Fixed { .. }
        )
    }
}
