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
    Reference {
        name: std::string::String,
        namespace: Option<std::string::String>,
        properties: IndexMap<std::string::String, Value>,
    },
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
            }
            | AvroSchema::Reference {
                name, namespace, ..
            } => Some(match namespace {
                Some(ns) if !ns.is_empty() => format!("{ns}.{name}"),
                _ => name.clone(),
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

    /// Returns the key used for duplicate detection within a union.
    ///
    /// The Avro specification requires that unions not contain more than one
    /// schema with the same type. For anonymous types (primitives, arrays,
    /// maps), the key is the type name (e.g., `"null"`, `"array"`). For named
    /// types (record, enum, fixed) and references, the key is the fully
    /// qualified name.
    ///
    /// This mirrors Java's `Schema.getFullName()` behavior used in
    /// `UnionSchema`'s constructor for duplicate checking.
    pub fn union_type_key(&self) -> String {
        match self {
            // Primitives: keyed by their type name.
            AvroSchema::Null => "null".to_string(),
            AvroSchema::Boolean => "boolean".to_string(),
            AvroSchema::Int => "int".to_string(),
            AvroSchema::Long => "long".to_string(),
            AvroSchema::Float => "float".to_string(),
            AvroSchema::Double => "double".to_string(),
            AvroSchema::Bytes => "bytes".to_string(),
            AvroSchema::String => "string".to_string(),

            // Named types and references: keyed by fully qualified name.
            AvroSchema::Record { .. }
            | AvroSchema::Enum { .. }
            | AvroSchema::Fixed { .. }
            | AvroSchema::Reference { .. } => {
                self.full_name()
                    .expect("match arm restricts to Record/Enum/Fixed/Reference, all have full_name")
            }

            // Complex anonymous types: keyed by their structural type name.
            AvroSchema::Array { .. } => "array".to_string(),
            AvroSchema::Map { .. } => "map".to_string(),
            AvroSchema::Union { .. } => "union".to_string(),

            // Annotated primitives: keyed by the underlying primitive type.
            AvroSchema::AnnotatedPrimitive { kind, .. } => kind.as_str().to_string(),

            // Logical types: keyed by the underlying primitive type name.
            // Java treats logical types as their underlying type for union
            // duplicate checking (e.g., `date` is `int`, `uuid` is `string`).
            AvroSchema::Logical { logical_type, .. } => match logical_type {
                LogicalType::Date => "int".to_string(),
                LogicalType::TimeMillis => "int".to_string(),
                LogicalType::TimestampMillis => "long".to_string(),
                LogicalType::LocalTimestampMillis => "long".to_string(),
                LogicalType::Uuid => "string".to_string(),
                LogicalType::Decimal { .. } => "bytes".to_string(),
            },
        }
    }

    /// Returns a human-readable type description for use in error messages.
    pub fn type_description(&self) -> String {
        match self {
            AvroSchema::Null => "null".to_string(),
            AvroSchema::Boolean => "boolean".to_string(),
            AvroSchema::Int => "int".to_string(),
            AvroSchema::Long => "long".to_string(),
            AvroSchema::Float => "float".to_string(),
            AvroSchema::Double => "double".to_string(),
            AvroSchema::Bytes => "bytes".to_string(),
            AvroSchema::String => "string".to_string(),
            AvroSchema::Record { name, .. } => format!("record {name}"),
            AvroSchema::Enum { name, .. } => format!("enum {name}"),
            AvroSchema::Fixed { name, .. } => format!("fixed {name}"),
            AvroSchema::Array { .. } => "array".to_string(),
            AvroSchema::Map { .. } => "map".to_string(),
            AvroSchema::Union { .. } => "union".to_string(),
            AvroSchema::AnnotatedPrimitive { kind, .. } => kind.as_str().to_string(),
            AvroSchema::Logical { logical_type, .. } => match logical_type {
                LogicalType::Date => "date".to_string(),
                LogicalType::TimeMillis => "time_ms".to_string(),
                LogicalType::TimestampMillis => "timestamp_ms".to_string(),
                LogicalType::LocalTimestampMillis => "local_timestamp_ms".to_string(),
                LogicalType::Uuid => "uuid".to_string(),
                LogicalType::Decimal { .. } => "decimal".to_string(),
            },
            AvroSchema::Reference { name, .. } => name.clone(),
        }
    }
}

// ==============================================================================
// Default Value Validation
// ==============================================================================

/// Returns a human-readable description of a JSON value's type, for use in
/// error messages.
fn json_type_description(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Check whether a `serde_json::Value` is a valid integer (fits in i64/u64
/// without a fractional part). This is the correct check for Avro `int` and
/// `long` defaults, which must be JSON integers, not floats.
fn is_json_integer(n: &serde_json::Number) -> bool {
    n.is_i64() || n.is_u64()
}

/// Validate that a JSON default value is compatible with the given Avro schema
/// type, per the Avro specification's default value rules.
///
/// Returns `true` if the value is valid for the schema, `false` otherwise.
///
/// For `Reference` types (forward references not yet resolved), validation is
/// skipped and `true` is returned, because the referenced type is not available
/// for inspection at parse time.
pub fn is_valid_default(value: &Value, schema: &AvroSchema) -> bool {
    match schema {
        // =====================================================================
        // Primitives: each has exactly one valid JSON type.
        // =====================================================================
        AvroSchema::Null => value.is_null(),
        AvroSchema::Boolean => value.is_boolean(),
        AvroSchema::Int | AvroSchema::Long => {
            matches!(value, Value::Number(n) if is_json_integer(n))
        }
        AvroSchema::Float | AvroSchema::Double => {
            // JSON numbers are always valid. Additionally, the special string
            // values "NaN", "Infinity", and "-Infinity" are valid because JSON
            // cannot represent these IEEE 754 values natively — both Java and
            // our tool serialize them as strings in the JSON output.
            value.is_number()
                || matches!(value, Value::String(s) if s == "NaN" || s == "Infinity" || s == "-Infinity")
        }
        AvroSchema::String => value.is_string(),
        AvroSchema::Bytes => value.is_string(),

        // =====================================================================
        // Named types
        // =====================================================================
        AvroSchema::Record { .. } => value.is_object(),
        AvroSchema::Enum { .. } => value.is_string(),
        AvroSchema::Fixed { .. } => value.is_string(),

        // =====================================================================
        // Complex types
        // =====================================================================
        AvroSchema::Array { .. } => value.is_array(),
        AvroSchema::Map { .. } => value.is_object(),

        // Java's `Schema.isValidDefault` checks whether the default matches
        // *any* branch of the union, not just the first. The Avro spec says
        // "the default must correspond to the first schema", but Java relaxes
        // this, and we match Java's behavior.
        AvroSchema::Union { types, .. } => {
            if types.is_empty() {
                false
            } else {
                types.iter().any(|branch| is_valid_default(value, branch))
            }
        }

        // =====================================================================
        // Annotated primitives: validate against the underlying primitive type.
        // =====================================================================
        AvroSchema::AnnotatedPrimitive { kind, .. } => {
            let underlying = match kind {
                PrimitiveType::Null => AvroSchema::Null,
                PrimitiveType::Boolean => AvroSchema::Boolean,
                PrimitiveType::Int => AvroSchema::Int,
                PrimitiveType::Long => AvroSchema::Long,
                PrimitiveType::Float => AvroSchema::Float,
                PrimitiveType::Double => AvroSchema::Double,
                PrimitiveType::Bytes => AvroSchema::Bytes,
                PrimitiveType::String => AvroSchema::String,
            };
            is_valid_default(value, &underlying)
        }

        // =====================================================================
        // Logical types: validate against the underlying physical type.
        // =====================================================================
        AvroSchema::Logical { logical_type, .. } => {
            let underlying = match logical_type {
                LogicalType::Date | LogicalType::TimeMillis => AvroSchema::Int,
                LogicalType::TimestampMillis | LogicalType::LocalTimestampMillis => AvroSchema::Long,
                LogicalType::Uuid => AvroSchema::String,
                LogicalType::Decimal { .. } => AvroSchema::Bytes,
            };
            is_valid_default(value, &underlying)
        }

        // =====================================================================
        // Forward references: skip validation because the referenced type is
        // not yet resolved at parse time.
        // =====================================================================
        AvroSchema::Reference { .. } => true,
    }
}

/// Returns a human-readable description of why a default value is invalid,
/// or `None` if the value is valid for the schema.
pub fn validate_default(value: &Value, schema: &AvroSchema) -> Option<String> {
    if is_valid_default(value, schema) {
        None
    } else {
        Some(format!(
            "expected {}, got {}",
            schema.type_description(),
            json_type_description(value),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // =========================================================================
    // Primitives: valid defaults
    // =========================================================================

    #[test]
    fn null_accepts_null() {
        assert!(is_valid_default(&json!(null), &AvroSchema::Null));
    }

    #[test]
    fn boolean_accepts_true() {
        assert!(is_valid_default(&json!(true), &AvroSchema::Boolean));
    }

    #[test]
    fn boolean_accepts_false() {
        assert!(is_valid_default(&json!(false), &AvroSchema::Boolean));
    }

    #[test]
    fn int_accepts_integer() {
        assert!(is_valid_default(&json!(42), &AvroSchema::Int));
    }

    #[test]
    fn int_accepts_zero() {
        assert!(is_valid_default(&json!(0), &AvroSchema::Int));
    }

    #[test]
    fn int_accepts_negative() {
        assert!(is_valid_default(&json!(-1), &AvroSchema::Int));
    }

    #[test]
    fn long_accepts_integer() {
        assert!(is_valid_default(&json!(100), &AvroSchema::Long));
    }

    #[test]
    fn float_accepts_number() {
        assert!(is_valid_default(&json!(3.14), &AvroSchema::Float));
    }

    #[test]
    fn float_accepts_integer() {
        assert!(is_valid_default(&json!(0), &AvroSchema::Float));
    }

    #[test]
    fn double_accepts_number() {
        assert!(is_valid_default(&json!(2.718), &AvroSchema::Double));
    }

    #[test]
    fn double_accepts_nan_string() {
        assert!(is_valid_default(&json!("NaN"), &AvroSchema::Double));
    }

    #[test]
    fn double_accepts_infinity_string() {
        assert!(is_valid_default(&json!("Infinity"), &AvroSchema::Double));
    }

    #[test]
    fn double_accepts_neg_infinity_string() {
        assert!(is_valid_default(&json!("-Infinity"), &AvroSchema::Double));
    }

    #[test]
    fn float_accepts_nan_string() {
        assert!(is_valid_default(&json!("NaN"), &AvroSchema::Float));
    }

    #[test]
    fn string_accepts_string() {
        assert!(is_valid_default(&json!("hello"), &AvroSchema::String));
    }

    #[test]
    fn bytes_accepts_string() {
        assert!(is_valid_default(&json!("\\u0000"), &AvroSchema::Bytes));
    }

    // =========================================================================
    // Primitives: invalid defaults
    // =========================================================================

    #[test]
    fn null_rejects_integer() {
        assert!(!is_valid_default(&json!(42), &AvroSchema::Null));
    }

    #[test]
    fn boolean_rejects_integer() {
        assert!(!is_valid_default(&json!(42), &AvroSchema::Boolean));
    }

    #[test]
    fn boolean_rejects_string() {
        assert!(!is_valid_default(&json!("true"), &AvroSchema::Boolean));
    }

    #[test]
    fn int_rejects_string() {
        assert!(!is_valid_default(&json!("hello"), &AvroSchema::Int));
    }

    #[test]
    fn int_rejects_float() {
        assert!(!is_valid_default(&json!(3.14), &AvroSchema::Int));
    }

    #[test]
    fn int_rejects_null() {
        assert!(!is_valid_default(&json!(null), &AvroSchema::Int));
    }

    #[test]
    fn int_rejects_boolean() {
        assert!(!is_valid_default(&json!(true), &AvroSchema::Int));
    }

    #[test]
    fn int_rejects_array() {
        assert!(!is_valid_default(&json!([1, 2, 3]), &AvroSchema::Int));
    }

    #[test]
    fn int_rejects_object() {
        assert!(!is_valid_default(&json!({"key": "value"}), &AvroSchema::Int));
    }

    #[test]
    fn long_rejects_float() {
        assert!(!is_valid_default(&json!(3.14), &AvroSchema::Long));
    }

    #[test]
    fn string_rejects_integer() {
        assert!(!is_valid_default(&json!(42), &AvroSchema::String));
    }

    #[test]
    fn string_rejects_array() {
        assert!(!is_valid_default(&json!([1, 2, 3]), &AvroSchema::String));
    }

    #[test]
    fn bytes_rejects_integer() {
        assert!(!is_valid_default(&json!(42), &AvroSchema::Bytes));
    }

    #[test]
    fn double_rejects_string() {
        // Regular strings are not valid; only "NaN", "Infinity", "-Infinity".
        assert!(!is_valid_default(&json!("hello"), &AvroSchema::Double));
    }

    #[test]
    fn float_rejects_regular_string() {
        assert!(!is_valid_default(&json!("3.14"), &AvroSchema::Float));
    }

    // =========================================================================
    // Named types
    // =========================================================================

    #[test]
    fn record_accepts_object() {
        let schema = AvroSchema::Record {
            name: "TestRecord".to_string(),
            namespace: None,
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: IndexMap::new(),
        };
        assert!(is_valid_default(&json!({"name": "bar"}), &schema));
    }

    #[test]
    fn record_rejects_string() {
        let schema = AvroSchema::Record {
            name: "TestRecord".to_string(),
            namespace: None,
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: IndexMap::new(),
        };
        assert!(!is_valid_default(&json!("not_an_object"), &schema));
    }

    #[test]
    fn enum_accepts_string() {
        let schema = AvroSchema::Enum {
            name: "Suit".to_string(),
            namespace: None,
            doc: None,
            symbols: vec!["HEARTS".to_string(), "DIAMONDS".to_string()],
            default: None,
            aliases: vec![],
            properties: IndexMap::new(),
        };
        assert!(is_valid_default(&json!("HEARTS"), &schema));
    }

    #[test]
    fn enum_rejects_integer() {
        let schema = AvroSchema::Enum {
            name: "Suit".to_string(),
            namespace: None,
            doc: None,
            symbols: vec!["HEARTS".to_string()],
            default: None,
            aliases: vec![],
            properties: IndexMap::new(),
        };
        assert!(!is_valid_default(&json!(0), &schema));
    }

    #[test]
    fn fixed_accepts_string() {
        let schema = AvroSchema::Fixed {
            name: "MD5".to_string(),
            namespace: None,
            doc: None,
            size: 16,
            aliases: vec![],
            properties: IndexMap::new(),
        };
        assert!(is_valid_default(&json!("0000000000000000"), &schema));
    }

    #[test]
    fn fixed_rejects_integer() {
        let schema = AvroSchema::Fixed {
            name: "MD5".to_string(),
            namespace: None,
            doc: None,
            size: 16,
            aliases: vec![],
            properties: IndexMap::new(),
        };
        assert!(!is_valid_default(&json!(42), &schema));
    }

    // =========================================================================
    // Complex types
    // =========================================================================

    #[test]
    fn array_accepts_array() {
        let schema = AvroSchema::Array {
            items: Box::new(AvroSchema::Int),
            properties: IndexMap::new(),
        };
        assert!(is_valid_default(&json!([]), &schema));
    }

    #[test]
    fn array_accepts_non_empty_array() {
        let schema = AvroSchema::Array {
            items: Box::new(AvroSchema::Int),
            properties: IndexMap::new(),
        };
        assert!(is_valid_default(&json!([1, 2, 3]), &schema));
    }

    #[test]
    fn array_rejects_string() {
        let schema = AvroSchema::Array {
            items: Box::new(AvroSchema::Int),
            properties: IndexMap::new(),
        };
        assert!(!is_valid_default(&json!("not_an_array"), &schema));
    }

    #[test]
    fn map_accepts_object() {
        let schema = AvroSchema::Map {
            values: Box::new(AvroSchema::String),
            properties: IndexMap::new(),
        };
        assert!(is_valid_default(&json!({}), &schema));
    }

    #[test]
    fn map_rejects_array() {
        let schema = AvroSchema::Map {
            values: Box::new(AvroSchema::String),
            properties: IndexMap::new(),
        };
        assert!(!is_valid_default(&json!([1, 2]), &schema));
    }

    // =========================================================================
    // Union defaults: must match the first type in the union
    // =========================================================================

    #[test]
    fn union_null_first_accepts_null() {
        let schema = AvroSchema::Union {
            types: vec![AvroSchema::Null, AvroSchema::String],
            is_nullable_type: true,
        };
        assert!(is_valid_default(&json!(null), &schema));
    }

    #[test]
    fn union_null_first_accepts_string_from_second_branch() {
        let schema = AvroSchema::Union {
            types: vec![AvroSchema::Null, AvroSchema::String],
            is_nullable_type: true,
        };
        // Java validates against any branch, not just the first.
        assert!(is_valid_default(&json!("hello"), &schema));
    }

    #[test]
    fn union_null_first_rejects_integer() {
        let schema = AvroSchema::Union {
            types: vec![AvroSchema::Null, AvroSchema::String],
            is_nullable_type: true,
        };
        // Integer does not match either null or string.
        assert!(!is_valid_default(&json!(42), &schema));
    }

    #[test]
    fn union_string_first_accepts_string() {
        let schema = AvroSchema::Union {
            types: vec![AvroSchema::String, AvroSchema::Null],
            is_nullable_type: true,
        };
        assert!(is_valid_default(&json!("hello"), &schema));
    }

    #[test]
    fn union_string_first_accepts_null_from_second_branch() {
        let schema = AvroSchema::Union {
            types: vec![AvroSchema::String, AvroSchema::Null],
            is_nullable_type: true,
        };
        // Null matches the second branch.
        assert!(is_valid_default(&json!(null), &schema));
    }

    #[test]
    fn union_int_string_accepts_either() {
        let schema = AvroSchema::Union {
            types: vec![AvroSchema::Int, AvroSchema::String],
            is_nullable_type: false,
        };
        assert!(is_valid_default(&json!(42), &schema));
        assert!(is_valid_default(&json!("hello"), &schema));
        assert!(!is_valid_default(&json!(true), &schema));
    }

    // =========================================================================
    // Annotated primitives and logical types
    // =========================================================================

    #[test]
    fn annotated_long_accepts_integer() {
        let schema = AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Long,
            properties: IndexMap::new(),
        };
        assert!(is_valid_default(&json!(0), &schema));
    }

    #[test]
    fn annotated_long_rejects_string() {
        let schema = AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Long,
            properties: IndexMap::new(),
        };
        assert!(!is_valid_default(&json!("hello"), &schema));
    }

    #[test]
    fn logical_date_accepts_integer() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::Date,
            properties: IndexMap::new(),
        };
        assert!(is_valid_default(&json!(0), &schema));
    }

    #[test]
    fn logical_date_rejects_string() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::Date,
            properties: IndexMap::new(),
        };
        assert!(!is_valid_default(&json!("2023-01-01"), &schema));
    }

    #[test]
    fn logical_uuid_accepts_string() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::Uuid,
            properties: IndexMap::new(),
        };
        assert!(is_valid_default(&json!("550e8400-e29b-41d4-a716-446655440000"), &schema));
    }

    #[test]
    fn logical_uuid_rejects_integer() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::Uuid,
            properties: IndexMap::new(),
        };
        assert!(!is_valid_default(&json!(42), &schema));
    }

    #[test]
    fn logical_timestamp_millis_accepts_integer() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::TimestampMillis,
            properties: IndexMap::new(),
        };
        assert!(is_valid_default(&json!(1609459200000i64), &schema));
    }

    #[test]
    fn logical_decimal_accepts_string() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::Decimal {
                precision: 10,
                scale: 2,
            },
            properties: IndexMap::new(),
        };
        // Decimal's underlying type is bytes, which is serialized as a string.
        assert!(is_valid_default(&json!("\\u0000"), &schema));
    }

    // =========================================================================
    // Forward references: always pass validation
    // =========================================================================

    #[test]
    fn reference_accepts_any_value() {
        let schema = AvroSchema::Reference {
            name: "SomeType".to_string(),
            namespace: None,
            properties: IndexMap::new(),
        };
        // References skip validation because the type is not yet resolved.
        assert!(is_valid_default(&json!(42), &schema));
        assert!(is_valid_default(&json!("string"), &schema));
        assert!(is_valid_default(&json!(null), &schema));
        assert!(is_valid_default(&json!({}), &schema));
    }

    // =========================================================================
    // validate_default returns descriptive messages
    // =========================================================================

    #[test]
    fn validate_default_returns_none_for_valid() {
        assert!(validate_default(&json!(42), &AvroSchema::Int).is_none());
    }

    #[test]
    fn validate_default_returns_reason_for_invalid() {
        let reason = validate_default(&json!("hello"), &AvroSchema::Int);
        assert!(reason.is_some());
        let msg = reason.expect("should have a reason");
        assert!(msg.contains("expected int"), "message was: {msg}");
        assert!(msg.contains("got string"), "message was: {msg}");
    }
}
