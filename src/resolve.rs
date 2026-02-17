// ==============================================================================
// Schema Registry: Named Type Tracking and Forward Reference Validation
// ==============================================================================
//
// The Avro IDL allows forward references -- a record field can reference a type
// that hasn't been defined yet. During parsing, these unresolved type names are
// stored as `AvroSchema::Reference(full_name)`. After all types in the protocol
// are parsed, we validate that every `Reference` node points to a type that was
// actually registered.
//
// The `Reference` variant intentionally stays in the schema tree rather than
// being replaced inline with the full definition. The JSON serializer (in
// model/json.rs) renders `Reference` nodes as bare name strings, which is the
// correct Avro protocol JSON representation for subsequent occurrences of a
// named type.

use indexmap::IndexMap;
use miette::SourceSpan;

use crate::model::schema::{AvroSchema, make_full_name};

// ==============================================================================
// Avro Name Validation
// ==============================================================================
//
// The Avro specification requires that names match `[A-Za-z_][A-Za-z0-9_]*`.
// The ANTLR grammar's `IdentifierToken` is more permissive -- it allows dashes
// and dots between identifier parts. Java's `IdlReader.validateName` enforces
// the pattern `[_\p{L}][_\p{LD}]*` (Unicode-aware), rejecting dashed
// identifiers like `my-record` while accepting Unicode letters/digits.
// We replicate that validation here using Rust's Unicode-aware `char` methods
// rather than the `regex` crate.

/// Check whether a single name segment is a valid Avro name.
///
/// The Avro specification defines names as `[A-Za-z_][A-Za-z0-9_]*`, but the
/// Java reference implementation (`IdlReader.VALID_NAME`) uses the Unicode-aware
/// pattern `[_\p{L}][_\p{LD}]*`, which accepts Unicode letters and digits.
/// We match Java's behavior so that IDL files with Unicode identifiers (like
/// Cyrillic or CJK names) work correctly.
pub(crate) fn is_valid_avro_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        // First character must be a Unicode letter or underscore.
        Some(c) if c.is_alphabetic() || c == '_' => {}
        _ => return false,
    }
    // Remaining characters must be Unicode letters, digits, or underscores.
    chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// Validate that a schema's name and namespace segments are valid Avro names.
///
/// For namespaces, each dot-separated segment must independently satisfy the
/// name pattern. Returns `Ok(())` if valid, or an error message describing
/// which part is invalid.
fn validate_schema_name(name: &str, namespace: &Option<String>) -> Result<(), String> {
    if !is_valid_avro_name(name) {
        return Err(format!(
            "invalid Avro name: `{name}` \
             (names must start with a letter or underscore, \
             followed by letters, digits, or underscores)"
        ));
    }
    if let Some(ns) = namespace
        && !ns.is_empty()
    {
        for segment in ns.split('.') {
            if !is_valid_avro_name(segment) {
                return Err(format!(
                    "invalid Avro namespace segment: `{segment}` in `{ns}` \
                         (each segment must start with a letter or underscore, \
                         followed by letters, digits, or underscores)"
                ));
            }
        }
    }
    Ok(())
}

/// Registry of named Avro types, tracking definition order for output.
///
/// Named types (record, enum, fixed) are registered as they're parsed.
/// Forward references can then be validated against this registry.
pub struct SchemaRegistry {
    /// Named schemas indexed by full name (namespace.name), in registration order.
    schemas: IndexMap<String, AvroSchema>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        SchemaRegistry {
            schemas: IndexMap::new(),
        }
    }

    /// Reserve capacity for at least `additional` more named types.
    /// This avoids incremental reallocation of the backing `IndexMap` as types
    /// are registered, which profiling showed accounts for ~4.2% of total time
    /// on large inputs.
    pub fn reserve(&mut self, additional: usize) {
        self.schemas.reserve(additional);
    }

    /// Register a named schema. The schema must have a full name (i.e., be a
    /// record, enum, or fixed type). Returns an error if the name is already
    /// registered, if the schema is not a named type, or if the name/namespace
    /// contains characters invalid per the Avro specification.
    pub fn register(&mut self, schema: AvroSchema) -> Result<(), String> {
        let full_name = schema
            .full_name()
            .ok_or_else(|| "cannot register non-named schema".to_string())?
            .into_owned();

        // Validate that the name and namespace segments conform to the Avro
        // spec's name pattern before accepting the schema. This catches names
        // like `my-record` that the ANTLR grammar permits but the spec forbids.
        let (name, namespace) = match &schema {
            AvroSchema::Record {
                name, namespace, ..
            }
            | AvroSchema::Enum {
                name, namespace, ..
            }
            | AvroSchema::Fixed {
                name, namespace, ..
            } => (name.as_str(), namespace),
            _ => unreachable!("full_name() returned Some for a non-named type"),
        };
        validate_schema_name(name, namespace)?;

        if self.schemas.contains_key(&full_name) {
            return Err(format!("duplicate schema name: {full_name}"));
        }
        self.schemas.insert(full_name, schema);
        Ok(())
    }

    /// Look up a named schema by full name.
    pub fn lookup(&self, full_name: &str) -> Option<&AvroSchema> {
        self.schemas.get(full_name)
    }

    /// Return all registered schemas as a reference, in registration order.
    pub fn schemas(&self) -> impl Iterator<Item = &AvroSchema> {
        self.schemas.values()
    }

    /// Return all registered full names (e.g., `"org.example.Foo"`), in
    /// registration order. Used to suggest similar names for typos.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.schemas.keys().map(|k| k.as_str())
    }

    // -- Test-only helpers below this line --

    /// Check whether a name is registered.
    #[cfg(test)]
    pub fn contains(&self, full_name: &str) -> bool {
        self.schemas.contains_key(full_name)
    }

    /// Return all registered schemas in registration order, consuming the
    /// registry.
    #[cfg(test)]
    pub fn into_schemas(self) -> Vec<AvroSchema> {
        self.schemas.into_values().collect()
    }

    /// Merge schemas from another registry (used for imports).
    /// Schemas already present (by full name) are skipped, preserving the
    /// original definition.
    #[cfg(test)]
    pub fn merge(&mut self, other: SchemaRegistry) {
        for (name, schema) in other.schemas {
            self.schemas.entry(name).or_insert(schema);
        }
    }

    /// Validate that all `AvroSchema::Reference` nodes in registered schemas
    /// point to actually registered types. Returns a list of `(full_name,
    /// span)` pairs for unresolved references in tree-walk order (empty if
    /// everything resolves). The span allows callers to produce source-
    /// highlighted diagnostics.
    ///
    /// The caller is responsible for deduplication and ordering -- see
    /// `validate_all_references` in `compiler.rs`.
    pub fn validate_references(&self) -> Vec<(String, Option<SourceSpan>)> {
        let mut unresolved = Vec::new();
        for schema in self.schemas.values() {
            collect_unresolved_refs(schema, &self.schemas, &mut unresolved);
        }
        unresolved
    }

    /// Validate that all `AvroSchema::Reference` nodes in the given schema
    /// point to types registered in this registry. Returns a list of
    /// `(full_name, span)` pairs for unresolved references in tree-walk
    /// order (empty if everything resolves).
    ///
    /// This is used to validate schemas that are *not* themselves registered
    /// in the registry -- specifically, the top-level schema from
    /// `IdlFile::Schema`, which is stored separately and would otherwise
    /// escape validation by `validate_references`.
    ///
    /// The caller is responsible for deduplication and ordering -- see
    /// `validate_all_references` in `compiler.rs`.
    pub fn validate_schema(&self, schema: &AvroSchema) -> Vec<(String, Option<SourceSpan>)> {
        let mut unresolved = Vec::new();
        collect_unresolved_refs(schema, &self.schemas, &mut unresolved);
        unresolved
    }
}

/// Recursively walk a schema tree and collect any `Reference` names that
/// don't correspond to a known type in the provided name set.
///
/// Each unresolved entry is a `(full_name, span)` pair. The span comes
/// from the `Reference` variant and may be `None` for references created
/// from JSON imports (which have no source location).
///
/// This is the core validation logic shared by both `validate_references`
/// (which checks registered schemas) and `validate_schema` (which checks
/// an arbitrary schema against the registry).
fn collect_unresolved_refs(
    schema: &AvroSchema,
    known: &IndexMap<String, AvroSchema>,
    unresolved: &mut Vec<(String, Option<SourceSpan>)>,
) {
    match schema {
        AvroSchema::Reference {
            name,
            namespace,
            span,
            ..
        } => {
            let full_name = make_full_name(name, namespace.as_deref());
            if !known.contains_key(full_name.as_ref()) {
                unresolved.push((full_name.into_owned(), *span));
            }
        }
        AvroSchema::Record { fields, .. } => {
            for field in fields {
                collect_unresolved_refs(&field.schema, known, unresolved);
            }
        }
        AvroSchema::Array { items, .. } => {
            collect_unresolved_refs(items, known, unresolved);
        }
        AvroSchema::Map { values, .. } => {
            collect_unresolved_refs(values, known, unresolved);
        }
        AvroSchema::Union { types, .. } => {
            for t in types {
                collect_unresolved_refs(t, known, unresolved);
            }
        }
        // Primitives, logical types, enums, and fixed types contain no
        // nested schema references to validate.
        _ => {}
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Extract just the names from unresolved reference tuples, for concise
    /// test assertions.
    fn names(unresolved: Vec<(String, Option<SourceSpan>)>) -> Vec<String> {
        unresolved.into_iter().map(|(name, _)| name).collect()
    }

    #[test]
    fn test_register_and_lookup() {
        let mut reg = SchemaRegistry::new();
        let schema = AvroSchema::Record {
            name: "Ping".to_string(),
            namespace: Some("org.example".to_string()),
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        };
        reg.register(schema)
            .expect("registration of valid named schema succeeds");
        assert!(reg.contains("org.example.Ping"));
        assert!(reg.lookup("org.example.Ping").is_some());
        assert!(!reg.contains("Ping"));
    }

    #[test]
    fn test_duplicate_registration() {
        let mut reg = SchemaRegistry::new();
        let schema = AvroSchema::Enum {
            name: "Status".to_string(),
            namespace: None,
            doc: None,
            symbols: vec!["A".to_string()],
            default: None,
            aliases: vec![],
            properties: HashMap::new(),
        };
        reg.register(schema.clone())
            .expect("first registration of valid schema succeeds");
        assert!(reg.register(schema).is_err());
    }

    #[test]
    fn test_registration_order_preserved() {
        let mut reg = SchemaRegistry::new();
        for name in ["Alpha", "Beta", "Gamma"] {
            reg.register(AvroSchema::Fixed {
                name: name.to_string(),
                namespace: None,
                doc: None,
                size: 16,
                aliases: vec![],
                properties: HashMap::new(),
            })
            .expect("registration of distinct fixed schemas succeeds");
        }
        let names: Vec<_> = reg
            .schemas()
            .filter_map(|s| s.name().map(str::to_string))
            .collect();
        assert_eq!(names, vec!["Alpha", "Beta", "Gamma"]);
    }

    #[test]
    fn test_validate_references() {
        let mut reg = SchemaRegistry::new();
        reg.register(AvroSchema::Record {
            name: "Outer".to_string(),
            namespace: None,
            doc: None,
            fields: vec![crate::model::schema::Field {
                name: "inner".to_string(),
                schema: AvroSchema::Reference {
                    name: "Missing".to_string(),
                    namespace: None,
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
        })
        .expect("registration of record with reference field succeeds");
        let unresolved = reg.validate_references();
        assert_eq!(names(unresolved), vec!["Missing"]);
    }

    #[test]
    fn test_validate_references_resolves_known_types() {
        let mut reg = SchemaRegistry::new();
        // Register "Inner" first, then "Outer" which references it.
        reg.register(AvroSchema::Record {
            name: "Inner".to_string(),
            namespace: None,
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        })
        .expect("registration of Inner record succeeds");
        reg.register(AvroSchema::Record {
            name: "Outer".to_string(),
            namespace: None,
            doc: None,
            fields: vec![crate::model::schema::Field {
                name: "inner".to_string(),
                schema: AvroSchema::Reference {
                    name: "Inner".to_string(),
                    namespace: None,
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
        })
        .expect("registration of Outer record referencing Inner succeeds");
        let unresolved = reg.validate_references();
        assert!(unresolved.is_empty());
    }

    #[test]
    fn test_validate_nested_references() {
        let mut reg = SchemaRegistry::new();
        // A record with references nested inside array, map, and union types.
        reg.register(AvroSchema::Record {
            name: "Container".to_string(),
            namespace: None,
            doc: None,
            fields: vec![
                crate::model::schema::Field {
                    name: "items".to_string(),
                    schema: AvroSchema::Array {
                        items: Box::new(AvroSchema::Reference {
                            name: "MissingA".to_string(),
                            namespace: None,
                            properties: HashMap::new(),
                            span: None,
                        }),
                        properties: HashMap::new(),
                    },
                    doc: None,
                    default: None,
                    order: None,
                    aliases: vec![],
                    properties: HashMap::new(),
                },
                crate::model::schema::Field {
                    name: "lookup".to_string(),
                    schema: AvroSchema::Map {
                        values: Box::new(AvroSchema::Reference {
                            name: "MissingB".to_string(),
                            namespace: None,
                            properties: HashMap::new(),
                            span: None,
                        }),
                        properties: HashMap::new(),
                    },
                    doc: None,
                    default: None,
                    order: None,
                    aliases: vec![],
                    properties: HashMap::new(),
                },
                crate::model::schema::Field {
                    name: "choice".to_string(),
                    schema: AvroSchema::Union {
                        types: vec![
                            AvroSchema::Null,
                            AvroSchema::Reference {
                                name: "MissingC".to_string(),
                                namespace: None,
                                properties: HashMap::new(),
                                span: None,
                            },
                        ],
                        is_nullable_type: true,
                    },
                    doc: None,
                    default: None,
                    order: None,
                    aliases: vec![],
                    properties: HashMap::new(),
                },
            ],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        })
        .expect("registration of Container with nested references succeeds");
        let unresolved = reg.validate_references();
        assert_eq!(names(unresolved), vec!["MissingA", "MissingB", "MissingC"]);
    }

    #[test]
    fn test_merge_registries() {
        let mut reg1 = SchemaRegistry::new();
        reg1.register(AvroSchema::Fixed {
            name: "Hash".to_string(),
            namespace: None,
            doc: None,
            size: 32,
            aliases: vec![],
            properties: HashMap::new(),
        })
        .expect("registration of Hash into reg1 succeeds");

        let mut reg2 = SchemaRegistry::new();
        reg2.register(AvroSchema::Fixed {
            name: "Hash".to_string(),
            namespace: None,
            doc: None,
            size: 64, // Different size -- should be ignored since reg1 already has "Hash".
            aliases: vec![],
            properties: HashMap::new(),
        })
        .expect("registration of Hash into reg2 succeeds");
        reg2.register(AvroSchema::Fixed {
            name: "Token".to_string(),
            namespace: None,
            doc: None,
            size: 16,
            aliases: vec![],
            properties: HashMap::new(),
        })
        .expect("registration of Token into reg2 succeeds");

        reg1.merge(reg2);
        assert!(reg1.contains("Hash"));
        assert!(reg1.contains("Token"));

        // The original "Hash" (size 32) should be preserved, not overwritten.
        if let Some(AvroSchema::Fixed { size, .. }) = reg1.lookup("Hash") {
            assert_eq!(*size, 32);
        } else {
            panic!("expected Fixed schema for Hash");
        }
    }

    #[test]
    fn test_into_schemas_order() {
        let mut reg = SchemaRegistry::new();
        for name in ["X", "Y", "Z"] {
            reg.register(AvroSchema::Fixed {
                name: name.to_string(),
                namespace: None,
                doc: None,
                size: 8,
                aliases: vec![],
                properties: HashMap::new(),
            })
            .expect("registration of distinct fixed schemas succeeds");
        }
        let schemas = reg.into_schemas();
        let names: Vec<_> = schemas.iter().filter_map(|s| s.name()).collect();
        assert_eq!(names, vec!["X", "Y", "Z"]);
    }

    #[test]
    fn test_register_non_named_schema_fails() {
        let mut reg = SchemaRegistry::new();
        let err = reg.register(AvroSchema::Int).unwrap_err();
        insta::assert_snapshot!(err);
    }

    // =========================================================================
    // Name validation tests
    // =========================================================================

    #[test]
    fn test_is_valid_avro_name_accepts_valid_names() {
        assert!(is_valid_avro_name("Foo"));
        assert!(is_valid_avro_name("_private"));
        assert!(is_valid_avro_name("MyRecord123"));
        assert!(is_valid_avro_name("A"));
        assert!(is_valid_avro_name("_"));
        assert!(is_valid_avro_name("__double_underscore__"));
    }

    #[test]
    fn test_is_valid_avro_name_rejects_dashes() {
        assert!(!is_valid_avro_name("my-record"));
        assert!(!is_valid_avro_name("foo-bar-baz"));
    }

    #[test]
    fn test_is_valid_avro_name_rejects_leading_digit() {
        assert!(!is_valid_avro_name("1BadName"));
        assert!(!is_valid_avro_name("0x00"));
    }

    #[test]
    fn test_is_valid_avro_name_rejects_empty_and_special_chars() {
        assert!(!is_valid_avro_name(""));
        assert!(!is_valid_avro_name("has space"));
        assert!(!is_valid_avro_name("has.dot"));
        assert!(!is_valid_avro_name("has@at"));
    }

    #[test]
    fn test_is_valid_avro_name_accepts_unicode_letters() {
        // Java's VALID_NAME uses `\p{L}` and `\p{LD}`, which accepts Unicode
        // letters and digits. The golden test suite includes Cyrillic and CJK names.
        assert!(is_valid_avro_name("Структура"));
        assert!(is_valid_avro_name("文字列"));
        assert!(is_valid_avro_name("Протоколы"));
    }

    #[test]
    fn test_register_rejects_dashed_record_name() {
        let mut reg = SchemaRegistry::new();
        let result = reg.register(AvroSchema::Record {
            name: "my-record".to_string(),
            namespace: None,
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        });
        let err = result.unwrap_err();
        insta::assert_snapshot!(err);
    }

    #[test]
    fn test_register_rejects_dashed_enum_name() {
        let mut reg = SchemaRegistry::new();
        let result = reg.register(AvroSchema::Enum {
            name: "my-enum".to_string(),
            namespace: None,
            doc: None,
            symbols: vec!["A".to_string()],
            default: None,
            aliases: vec![],
            properties: HashMap::new(),
        });
        let err = result.unwrap_err();
        insta::assert_snapshot!(err);
    }

    #[test]
    fn test_register_rejects_invalid_namespace_segment() {
        let mut reg = SchemaRegistry::new();
        let result = reg.register(AvroSchema::Record {
            name: "ValidName".to_string(),
            namespace: Some("org.bad-segment.example".to_string()),
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        });
        let err = result.unwrap_err();
        insta::assert_snapshot!(err);
    }

    #[test]
    fn test_register_accepts_valid_namespaced_name() {
        let mut reg = SchemaRegistry::new();
        let result = reg.register(AvroSchema::Record {
            name: "MyRecord".to_string(),
            namespace: Some("org.apache.avro".to_string()),
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        });
        assert!(result.is_ok());
    }

    // =========================================================================
    // validate_schema tests (external schema validation)
    // =========================================================================

    #[test]
    fn test_validate_schema_detects_top_level_unresolved_reference() {
        // Simulates `schema DoesNotExist;` -- the top-level schema is a
        // Reference that is not in the registry.
        let reg = SchemaRegistry::new();
        let schema = AvroSchema::Reference {
            name: "DoesNotExist".to_string(),
            namespace: Some("com.example".to_string()),
            properties: HashMap::new(),
            span: None,
        };
        let unresolved = reg.validate_schema(&schema);
        assert_eq!(names(unresolved), vec!["com.example.DoesNotExist"]);
    }

    #[test]
    fn test_validate_schema_resolves_registered_reference() {
        // The schema references a type that IS in the registry -- no error.
        let mut reg = SchemaRegistry::new();
        reg.register(AvroSchema::Record {
            name: "MyRecord".to_string(),
            namespace: Some("com.example".to_string()),
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        })
        .expect("registration of MyRecord succeeds");

        let schema = AvroSchema::Reference {
            name: "MyRecord".to_string(),
            namespace: Some("com.example".to_string()),
            properties: HashMap::new(),
            span: None,
        };
        let unresolved = reg.validate_schema(&schema);
        assert!(unresolved.is_empty());
    }

    #[test]
    fn test_validate_schema_detects_nested_unresolved_in_array() {
        // Simulates `schema array<DoesNotExist>;`
        let reg = SchemaRegistry::new();
        let schema = AvroSchema::Array {
            items: Box::new(AvroSchema::Reference {
                name: "DoesNotExist".to_string(),
                namespace: None,
                properties: HashMap::new(),
                span: None,
            }),
            properties: HashMap::new(),
        };
        let unresolved = reg.validate_schema(&schema);
        assert_eq!(names(unresolved), vec!["DoesNotExist"]);
    }

    #[test]
    fn test_validate_schema_detects_nested_unresolved_in_map() {
        // Simulates `schema map<DoesNotExist>;`
        let reg = SchemaRegistry::new();
        let schema = AvroSchema::Map {
            values: Box::new(AvroSchema::Reference {
                name: "Missing".to_string(),
                namespace: Some("org.test".to_string()),
                properties: HashMap::new(),
                span: None,
            }),
            properties: HashMap::new(),
        };
        let unresolved = reg.validate_schema(&schema);
        assert_eq!(names(unresolved), vec!["org.test.Missing"]);
    }

    #[test]
    fn test_validate_schema_detects_nested_unresolved_in_union() {
        // Simulates `schema union { null, DoesNotExist };`
        let reg = SchemaRegistry::new();
        let schema = AvroSchema::Union {
            types: vec![
                AvroSchema::Null,
                AvroSchema::Reference {
                    name: "Missing".to_string(),
                    namespace: None,
                    properties: HashMap::new(),
                    span: None,
                },
            ],
            is_nullable_type: false,
        };
        let unresolved = reg.validate_schema(&schema);
        assert_eq!(names(unresolved), vec!["Missing"]);
    }

    #[test]
    fn test_validate_schema_passes_for_primitives() {
        // Primitives have no references to resolve.
        let reg = SchemaRegistry::new();
        assert!(reg.validate_schema(&AvroSchema::Int).is_empty());
        assert!(reg.validate_schema(&AvroSchema::String).is_empty());
        assert!(reg.validate_schema(&AvroSchema::Null).is_empty());
    }

    #[test]
    fn test_validate_schema_namespace_mismatch() {
        // Simulates `schema MyRecord;` where MyRecord is registered under a
        // different namespace than the reference expects.
        let mut reg = SchemaRegistry::new();
        reg.register(AvroSchema::Record {
            name: "MyRecord".to_string(),
            namespace: Some("com.other".to_string()),
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        })
        .expect("registration of MyRecord under com.other succeeds");

        // The reference resolves to `com.example.MyRecord`, but the registry
        // only has `com.other.MyRecord`.
        let schema = AvroSchema::Reference {
            name: "MyRecord".to_string(),
            namespace: Some("com.example".to_string()),
            properties: HashMap::new(),
            span: None,
        };
        let unresolved = reg.validate_schema(&schema);
        assert_eq!(names(unresolved), vec!["com.example.MyRecord"]);
    }
}
