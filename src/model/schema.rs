use miette::SourceSpan;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// Compute the fully-qualified name for an Avro named type.
///
/// When `namespace` is `Some` and non-empty, the result is `"namespace.name"`.
/// Otherwise, the bare `name` is returned without allocation.
pub(crate) fn make_full_name<'a>(name: &'a str, namespace: Option<&str>) -> Cow<'a, str> {
    match namespace {
        Some(ns) if !ns.is_empty() => Cow::Owned(format!("{ns}.{name}")),
        _ => Cow::Borrowed(name),
    }
}

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

    /// Convert this primitive type to its corresponding `AvroSchema` variant.
    pub fn to_schema(&self) -> AvroSchema {
        match self {
            PrimitiveType::Null => AvroSchema::Null,
            PrimitiveType::Boolean => AvroSchema::Boolean,
            PrimitiveType::Int => AvroSchema::Int,
            PrimitiveType::Long => AvroSchema::Long,
            PrimitiveType::Float => AvroSchema::Float,
            PrimitiveType::Double => AvroSchema::Double,
            PrimitiveType::Bytes => AvroSchema::Bytes,
            PrimitiveType::String => AvroSchema::String,
        }
    }
}

/// Error returned when parsing an unrecognized primitive type name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsePrimitiveTypeError {
    name: std::string::String,
}

impl fmt::Display for ParsePrimitiveTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown primitive type: {}", self.name)
    }
}

impl std::error::Error for ParsePrimitiveTypeError {}

impl FromStr for PrimitiveType {
    type Err = ParsePrimitiveTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "null" => Ok(PrimitiveType::Null),
            "boolean" => Ok(PrimitiveType::Boolean),
            "int" => Ok(PrimitiveType::Int),
            "long" => Ok(PrimitiveType::Long),
            "float" => Ok(PrimitiveType::Float),
            "double" => Ok(PrimitiveType::Double),
            "bytes" => Ok(PrimitiveType::Bytes),
            "string" => Ok(PrimitiveType::String),
            other => Err(ParsePrimitiveTypeError {
                name: other.to_string(),
            }),
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
        properties: HashMap<std::string::String, Value>,
    },
    Enum {
        name: std::string::String,
        namespace: Option<std::string::String>,
        doc: Option<std::string::String>,
        symbols: Vec<std::string::String>,
        default: Option<std::string::String>,
        aliases: Vec<std::string::String>,
        properties: HashMap<std::string::String, Value>,
    },
    Fixed {
        name: std::string::String,
        namespace: Option<std::string::String>,
        doc: Option<std::string::String>,
        size: u32,
        aliases: Vec<std::string::String>,
        properties: HashMap<std::string::String, Value>,
    },

    // =========================================================================
    // Complex types
    // =========================================================================
    Array {
        items: Box<AvroSchema>,
        properties: HashMap<std::string::String, Value>,
    },
    Map {
        values: Box<AvroSchema>,
        properties: HashMap<std::string::String, Value>,
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
        properties: HashMap<std::string::String, Value>,
    },

    // =========================================================================
    // Logical types overlaying a primitive
    // =========================================================================
    Logical {
        logical_type: LogicalType,
        /// Extra properties on the underlying primitive (e.g., `@foo.bar("baz")` on a `long`).
        properties: HashMap<std::string::String, Value>,
    },

    // =========================================================================
    // Forward reference placeholder, resolved after parsing completes.
    // =========================================================================
    Reference {
        name: std::string::String,
        namespace: Option<std::string::String>,
        properties: HashMap<std::string::String, Value>,
        /// Source location of this reference in the `.avdl` input, used for
        /// error diagnostics when the reference cannot be resolved. `None` for
        /// references created from JSON imports (no source location available).
        span: Option<SourceSpan>,
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
    pub properties: HashMap<std::string::String, Value>,
}

impl AvroSchema {
    /// If this is a primitive variant (`Null` through `String`), return its
    /// Avro type name. Returns `None` for all non-primitive variants.
    pub fn primitive_type_name(&self) -> Option<&'static str> {
        match self {
            AvroSchema::Null => Some("null"),
            AvroSchema::Boolean => Some("boolean"),
            AvroSchema::Int => Some("int"),
            AvroSchema::Long => Some("long"),
            AvroSchema::Float => Some("float"),
            AvroSchema::Double => Some("double"),
            AvroSchema::Bytes => Some("bytes"),
            AvroSchema::String => Some("string"),
            _ => None,
        }
    }

    /// Returns the full name of a named type (namespace.name), or `None` if not a named type.
    ///
    /// Returns `Cow::Borrowed` when there is no namespace (avoiding allocation),
    /// and `Cow::Owned` when a namespace prefix must be prepended.
    pub fn full_name(&self) -> Option<Cow<'_, str>> {
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
            } => Some(make_full_name(name, namespace.as_deref())),
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
        // Primitives: keyed by their type name.
        if let Some(name) = self.primitive_type_name() {
            return name.to_string();
        }

        match self {
            // Named types and references: keyed by fully qualified name.
            AvroSchema::Record { .. }
            | AvroSchema::Enum { .. }
            | AvroSchema::Fixed { .. }
            | AvroSchema::Reference { .. } => self
                .full_name()
                .expect("match arm restricts to Record/Enum/Fixed/Reference, all have full_name")
                .into_owned(),

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

            // Primitives are handled above by `primitive_type_name()`.
            _ => unreachable!("all AvroSchema variants are covered"),
        }
    }

    /// Returns a human-readable type description for use in error messages.
    pub fn type_description(&self) -> String {
        // Primitives: use their type name directly.
        if let Some(name) = self.primitive_type_name() {
            return name.to_string();
        }

        match self {
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

            // Primitives are handled above by `primitive_type_name()`.
            _ => unreachable!("all AvroSchema variants are covered"),
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
        AvroSchema::Int => {
            matches!(value, Value::Number(n) if n.is_i64()
                && n.as_i64().is_some_and(|v| (i32::MIN as i64..=i32::MAX as i64).contains(&v)))
        }
        AvroSchema::Long => {
            // `is_json_integer` already ensures the value fits in i64 or u64. Since
            // Avro `long` is a signed 64-bit integer, we additionally need `is_i64()`
            // to reject values in the u64-only range (i64::MAX+1 ..= u64::MAX).
            matches!(value, Value::Number(n) if n.is_i64())
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
            is_valid_default(value, &kind.to_schema())
        }

        // =====================================================================
        // Logical types: validate against the underlying physical type.
        // =====================================================================
        AvroSchema::Logical { logical_type, .. } => {
            let underlying = match logical_type {
                LogicalType::Date | LogicalType::TimeMillis => AvroSchema::Int,
                LogicalType::TimestampMillis | LogicalType::LocalTimestampMillis => {
                    AvroSchema::Long
                }
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
        return None;
    }

    // Produce a more specific message for integer values that are the right JSON
    // type but fall outside the schema's numeric range.
    if let Value::Number(n) = value
        && is_json_integer(n)
    {
        match schema {
            AvroSchema::Int => {
                return Some(format!(
                    "value {n} out of range for int (must be between {} and {})",
                    i32::MIN,
                    i32::MAX,
                ));
            }
            AvroSchema::Long => {
                return Some(format!(
                    "value {n} out of range for long (must be between {} and {})",
                    i64::MIN,
                    i64::MAX,
                ));
            }
            // Annotated primitives and logical types delegate to their underlying
            // type via is_valid_default, so range errors for them are caught above.
            // We fall through to the generic message for other schemas.
            _ => {}
        }
    }

    Some(format!(
        "expected {}, got {}",
        schema.type_description(),
        json_type_description(value),
    ))
}

/// Validate field defaults within a record schema, resolving `Reference` types
/// through the provided lookup function before checking.
///
/// At parse time, `validate_default` skips validation for `Reference` types
/// because the referenced schema is not yet available. This function runs
/// after type registration, when a resolver can look up previously-registered
/// types. If the reference resolves, the default is validated against the
/// resolved schema. If resolution fails (true forward reference), validation
/// is skipped, matching the existing behavior.
///
/// Returns a list of `(field_name, reason)` pairs for any invalid defaults
/// found.
pub fn validate_record_field_defaults<F>(schema: &AvroSchema, resolver: F) -> Vec<(String, String)>
where
    F: Fn(&str) -> Option<AvroSchema>,
{
    let fields = match schema {
        AvroSchema::Record { fields, .. } => fields,
        _ => return Vec::new(),
    };

    let mut errors = Vec::new();
    for field in fields {
        let default_val = match &field.default {
            Some(val) => val,
            None => continue,
        };

        // Only intervene for Reference types (and unions containing them).
        // Non-Reference types are already validated at parse time by
        // `walk_variable` in reader.rs.
        let resolved_schema = resolve_for_validation(&field.schema, &resolver);
        if let Some(ref resolved) = resolved_schema
            && let Some(reason) = validate_default(default_val, resolved)
        {
            errors.push((field.name.clone(), reason));
        }
        // If resolve_for_validation returns None, the reference could not be
        // resolved (true forward reference), so we skip validation.
    }

    errors
}

/// Attempt to resolve `Reference` types in a schema for default validation.
///
/// Returns `Some(resolved_schema)` if all references in the schema can be
/// resolved, or `None` if any reference is unresolvable (forward reference).
/// For non-Reference types, returns the schema unchanged.
fn resolve_for_validation<F>(schema: &AvroSchema, resolver: &F) -> Option<AvroSchema>
where
    F: Fn(&str) -> Option<AvroSchema>,
{
    match schema {
        AvroSchema::Reference {
            name, namespace, ..
        } => {
            let full_name = make_full_name(name, namespace.as_deref());
            resolver(&full_name)
        }
        AvroSchema::Union {
            types,
            is_nullable_type,
        } => {
            // Resolve any Reference branches within the union. If any branch
            // is an unresolvable forward reference, skip validation for the
            // entire union.
            let mut resolved_types = Vec::with_capacity(types.len());
            for branch in types {
                match resolve_for_validation(branch, resolver) {
                    Some(resolved) => resolved_types.push(resolved),
                    None => return None,
                }
            }
            Some(AvroSchema::Union {
                types: resolved_types,
                is_nullable_type: *is_nullable_type,
            })
        }
        // For all other types (primitives, records, enums, etc.), the schema
        // is already concrete and does not need resolution.
        other => Some(other.clone()),
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

    // =========================================================================
    // Int/Long: boundary values
    // =========================================================================

    #[test]
    fn int_accepts_i32_max() {
        assert!(is_valid_default(&json!(i32::MAX), &AvroSchema::Int));
    }

    #[test]
    fn int_accepts_i32_min() {
        assert!(is_valid_default(&json!(i32::MIN), &AvroSchema::Int));
    }

    #[test]
    fn int_rejects_above_i32_max() {
        assert!(!is_valid_default(
            &json!(i32::MAX as i64 + 1),
            &AvroSchema::Int
        ));
    }

    #[test]
    fn int_rejects_below_i32_min() {
        assert!(!is_valid_default(
            &json!(i32::MIN as i64 - 1),
            &AvroSchema::Int
        ));
    }

    #[test]
    fn int_rejects_large_positive() {
        // 9999999999 is the example from the issue.
        assert!(!is_valid_default(
            &json!(9_999_999_999_i64),
            &AvroSchema::Int
        ));
    }

    #[test]
    fn long_accepts_i64_max() {
        assert!(is_valid_default(&json!(i64::MAX), &AvroSchema::Long));
    }

    #[test]
    fn long_accepts_i64_min() {
        assert!(is_valid_default(&json!(i64::MIN), &AvroSchema::Long));
    }

    #[test]
    fn long_accepts_value_above_i32_max() {
        // Values that overflow int but fit in long should be valid for long.
        assert!(is_valid_default(
            &json!(i32::MAX as i64 + 1),
            &AvroSchema::Long
        ));
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
        assert!(!is_valid_default(
            &json!({"key": "value"}),
            &AvroSchema::Int
        ));
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
            properties: HashMap::new(),
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
            properties: HashMap::new(),
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
            properties: HashMap::new(),
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
            properties: HashMap::new(),
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
            properties: HashMap::new(),
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
            properties: HashMap::new(),
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
            properties: HashMap::new(),
        };
        assert!(is_valid_default(&json!([]), &schema));
    }

    #[test]
    fn array_accepts_non_empty_array() {
        let schema = AvroSchema::Array {
            items: Box::new(AvroSchema::Int),
            properties: HashMap::new(),
        };
        assert!(is_valid_default(&json!([1, 2, 3]), &schema));
    }

    #[test]
    fn array_rejects_string() {
        let schema = AvroSchema::Array {
            items: Box::new(AvroSchema::Int),
            properties: HashMap::new(),
        };
        assert!(!is_valid_default(&json!("not_an_array"), &schema));
    }

    #[test]
    fn map_accepts_object() {
        let schema = AvroSchema::Map {
            values: Box::new(AvroSchema::String),
            properties: HashMap::new(),
        };
        assert!(is_valid_default(&json!({}), &schema));
    }

    #[test]
    fn map_rejects_array() {
        let schema = AvroSchema::Map {
            values: Box::new(AvroSchema::String),
            properties: HashMap::new(),
        };
        assert!(!is_valid_default(&json!([1, 2]), &schema));
    }

    // =========================================================================
    // Union defaults: may match any branch (matching Java's relaxed behavior)
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
            properties: HashMap::new(),
        };
        assert!(is_valid_default(&json!(0), &schema));
    }

    #[test]
    fn annotated_long_rejects_string() {
        let schema = AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Long,
            properties: HashMap::new(),
        };
        assert!(!is_valid_default(&json!("hello"), &schema));
    }

    #[test]
    fn logical_date_accepts_integer() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::Date,
            properties: HashMap::new(),
        };
        assert!(is_valid_default(&json!(0), &schema));
    }

    #[test]
    fn logical_date_rejects_string() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::Date,
            properties: HashMap::new(),
        };
        assert!(!is_valid_default(&json!("2023-01-01"), &schema));
    }

    #[test]
    fn logical_uuid_accepts_string() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::Uuid,
            properties: HashMap::new(),
        };
        assert!(is_valid_default(
            &json!("550e8400-e29b-41d4-a716-446655440000"),
            &schema
        ));
    }

    #[test]
    fn logical_uuid_rejects_integer() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::Uuid,
            properties: HashMap::new(),
        };
        assert!(!is_valid_default(&json!(42), &schema));
    }

    #[test]
    fn logical_timestamp_millis_accepts_integer() {
        let schema = AvroSchema::Logical {
            logical_type: LogicalType::TimestampMillis,
            properties: HashMap::new(),
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
            properties: HashMap::new(),
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
            properties: HashMap::new(),
            span: None,
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

    #[test]
    fn validate_default_int_out_of_range_message() {
        let reason = validate_default(&json!(9_999_999_999_i64), &AvroSchema::Int);
        let msg = reason.expect("should have a reason for out-of-range int");
        assert!(msg.contains("out of range"), "message was: {msg}");
        assert!(msg.contains("9999999999"), "message was: {msg}");
        assert!(msg.contains(&i32::MIN.to_string()), "message was: {msg}");
        assert!(msg.contains(&i32::MAX.to_string()), "message was: {msg}");
    }

    #[test]
    fn validate_default_int_below_range_message() {
        let reason = validate_default(&json!(i32::MIN as i64 - 1), &AvroSchema::Int);
        let msg = reason.expect("should have a reason for below-range int");
        assert!(msg.contains("out of range"), "message was: {msg}");
    }

    #[test]
    fn validate_default_long_out_of_range_message() {
        // u64::MAX does not fit in i64, so serde_json stores it as u64-only.
        // We construct this via raw JSON parsing since json!(u64::MAX) might not
        // produce the exact representation we need.
        let big_val: Value = serde_json::from_str("18446744073709551615").expect("valid JSON");
        let reason = validate_default(&big_val, &AvroSchema::Long);
        let msg = reason.expect("should have a reason for out-of-range long");
        assert!(msg.contains("out of range"), "message was: {msg}");
    }

    // =========================================================================
    // validate_record_field_defaults: resolves references before validation
    // =========================================================================

    /// Helper: build a record schema with a single field using the given field
    /// schema and default value.
    fn make_record_with_default(
        field_name: &str,
        field_schema: AvroSchema,
        default: Value,
    ) -> AvroSchema {
        AvroSchema::Record {
            name: "Outer".to_string(),
            namespace: Some("org.test".to_string()),
            doc: None,
            fields: vec![Field {
                name: field_name.to_string(),
                schema: field_schema,
                doc: None,
                default: Some(default),
                order: None,
                aliases: vec![],
                properties: HashMap::new(),
            }],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        }
    }

    /// Resolver that maps "org.test.Inner" to a record schema.
    fn record_resolver(full_name: &str) -> Option<AvroSchema> {
        if full_name == "org.test.Inner" {
            Some(AvroSchema::Record {
                name: "Inner".to_string(),
                namespace: Some("org.test".to_string()),
                doc: None,
                fields: vec![Field {
                    name: "name".to_string(),
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
            })
        } else {
            None
        }
    }

    #[test]
    fn reference_field_rejects_string_default_for_record() {
        let schema = make_record_with_default(
            "inner",
            AvroSchema::Reference {
                name: "Inner".to_string(),
                namespace: Some("org.test".to_string()),
                properties: HashMap::new(),
                span: None,
            },
            json!("not a record"),
        );
        let errors = validate_record_field_defaults(&schema, record_resolver);
        assert_eq!(errors.len(), 1, "expected one error, got: {errors:?}");
        assert_eq!(errors[0].0, "inner");
        assert!(
            errors[0].1.contains("got string"),
            "reason was: {}",
            errors[0].1
        );
    }

    #[test]
    fn reference_field_rejects_integer_default_for_record() {
        let schema = make_record_with_default(
            "inner",
            AvroSchema::Reference {
                name: "Inner".to_string(),
                namespace: Some("org.test".to_string()),
                properties: HashMap::new(),
                span: None,
            },
            json!(42),
        );
        let errors = validate_record_field_defaults(&schema, record_resolver);
        assert_eq!(errors.len(), 1, "expected one error, got: {errors:?}");
        assert_eq!(errors[0].0, "inner");
        assert!(
            errors[0].1.contains("got number"),
            "reason was: {}",
            errors[0].1
        );
    }

    #[test]
    fn reference_field_accepts_object_default_for_record() {
        let schema = make_record_with_default(
            "inner",
            AvroSchema::Reference {
                name: "Inner".to_string(),
                namespace: Some("org.test".to_string()),
                properties: HashMap::new(),
                span: None,
            },
            json!({"name": "valid"}),
        );
        let errors = validate_record_field_defaults(&schema, record_resolver);
        assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    }

    #[test]
    fn forward_reference_skips_validation() {
        // When the reference cannot be resolved (forward reference), validation
        // should be skipped -- no errors reported even for clearly invalid defaults.
        let schema = make_record_with_default(
            "future_field",
            AvroSchema::Reference {
                name: "NotYetDefined".to_string(),
                namespace: Some("org.test".to_string()),
                properties: HashMap::new(),
                span: None,
            },
            json!("this would be invalid for a record, but we don't know that yet"),
        );
        // Resolver returns None for unknown types.
        let errors = validate_record_field_defaults(&schema, |_| None);
        assert!(
            errors.is_empty(),
            "forward references should skip validation, got: {errors:?}"
        );
    }

    #[test]
    fn reference_field_rejects_array_default_for_record() {
        let schema = make_record_with_default(
            "inner",
            AvroSchema::Reference {
                name: "Inner".to_string(),
                namespace: Some("org.test".to_string()),
                properties: HashMap::new(),
                span: None,
            },
            json!([1, 2, 3]),
        );
        let errors = validate_record_field_defaults(&schema, record_resolver);
        assert_eq!(errors.len(), 1, "expected one error, got: {errors:?}");
        assert_eq!(errors[0].0, "inner");
        assert!(
            errors[0].1.contains("got array"),
            "reason was: {}",
            errors[0].1
        );
    }

    #[test]
    fn reference_field_rejects_null_default_for_non_nullable_record() {
        let schema = make_record_with_default(
            "inner",
            AvroSchema::Reference {
                name: "Inner".to_string(),
                namespace: Some("org.test".to_string()),
                properties: HashMap::new(),
                span: None,
            },
            json!(null),
        );
        let errors = validate_record_field_defaults(&schema, record_resolver);
        assert_eq!(errors.len(), 1, "expected one error, got: {errors:?}");
        assert_eq!(errors[0].0, "inner");
        assert!(
            errors[0].1.contains("got null"),
            "reason was: {}",
            errors[0].1
        );
    }

    #[test]
    fn non_record_schema_returns_no_errors() {
        // validate_record_field_defaults should be a no-op for non-record schemas.
        let errors = validate_record_field_defaults(&AvroSchema::Int, record_resolver);
        assert!(errors.is_empty());
    }

    #[test]
    fn field_without_default_is_not_validated() {
        let schema = AvroSchema::Record {
            name: "Outer".to_string(),
            namespace: Some("org.test".to_string()),
            doc: None,
            fields: vec![Field {
                name: "inner".to_string(),
                schema: AvroSchema::Reference {
                    name: "Inner".to_string(),
                    namespace: Some("org.test".to_string()),
                    properties: HashMap::new(),
                    span: None,
                },
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
        let errors = validate_record_field_defaults(&schema, record_resolver);
        assert!(errors.is_empty());
    }

    #[test]
    fn nullable_reference_union_validates_resolved_type() {
        // `Inner? inner = null` should be valid (null matches the null branch).
        let schema = make_record_with_default(
            "inner",
            AvroSchema::Union {
                types: vec![
                    AvroSchema::Null,
                    AvroSchema::Reference {
                        name: "Inner".to_string(),
                        namespace: Some("org.test".to_string()),
                        properties: HashMap::new(),
                        span: None,
                    },
                ],
                is_nullable_type: true,
            },
            json!(null),
        );
        let errors = validate_record_field_defaults(&schema, record_resolver);
        assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    }

    #[test]
    fn nullable_reference_union_rejects_invalid_default() {
        // `Inner? inner = 42` should be invalid (42 matches neither null nor record).
        let schema = make_record_with_default(
            "inner",
            AvroSchema::Union {
                types: vec![
                    AvroSchema::Null,
                    AvroSchema::Reference {
                        name: "Inner".to_string(),
                        namespace: Some("org.test".to_string()),
                        properties: HashMap::new(),
                        span: None,
                    },
                ],
                is_nullable_type: true,
            },
            json!(42),
        );
        let errors = validate_record_field_defaults(&schema, record_resolver);
        assert_eq!(errors.len(), 1, "expected one error, got: {errors:?}");
    }
}
