// ==============================================================================
// IDL Reader: Recursive Parse Tree Walker
// ==============================================================================
//
// This module is the core of the Avro IDL parser. It takes a string containing
// Avro IDL source, lexes and parses it via ANTLR, then walks the resulting
// parse tree recursively to build our domain model (Protocol, AvroSchema, etc.).
//
// The generated parser defines token constants in lower_Camel_case (e.g.
// `Idl_Boolean`). We suppress the naming warning for the whole module since
// these constants appear extensively in match arms.
#![allow(non_upper_case_globals)]
//
// The Java reference implementation uses ANTLR's listener pattern with mutable
// stacks. That approach is awkward in Rust due to lifetime constraints on trait
// objects, so instead we set `build_parse_tree = true` and walk the tree with
// plain recursive functions that return values. This is simpler and more
// idiomatic Rust.

use std::rc::Rc;

use antlr4rust::common_token_stream::CommonTokenStream;
use antlr4rust::token::Token;
use antlr4rust::tree::ParseTree;
use antlr4rust::InputStream;
use indexmap::IndexMap;
use serde_json::Value;

use crate::doc_comments::extract_doc_comment;
use crate::error::{IdlError, Result};
use crate::generated::idllexer::IdlLexer;
use crate::generated::idlparser::*;
use crate::model::protocol::{Message, Protocol};
use crate::model::schema::{AvroSchema, Field, FieldOrder, LogicalType, PrimitiveType};
use crate::resolve::SchemaRegistry;

// ==========================================================================
// Public API
// ==========================================================================

/// The result of parsing an IDL file -- either a protocol or a standalone schema.
#[derive(Debug)]
pub enum IdlFile {
    ProtocolFile(Protocol),
    SchemaFile(AvroSchema),
}

/// Import type discovered during parsing. The actual import resolution is
/// deferred to the `import` module (not yet implemented).
#[derive(Debug, Clone)]
pub struct ImportEntry {
    pub kind: ImportKind,
    pub path: String,
}

/// The kind of import statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportKind {
    Idl,
    Protocol,
    Schema,
}

/// Parse an Avro IDL string into an `IdlFile` and a registry of named types.
///
/// The returned `SchemaRegistry` contains all named types (records, enums,
/// fixed) defined in the IDL, in definition order. The returned `Vec<ImportEntry>`
/// contains any import statements encountered but not yet resolved.
pub fn parse_idl(input: &str) -> Result<(IdlFile, SchemaRegistry, Vec<ImportEntry>)> {
    let input_stream = InputStream::new(input);
    let lexer = IdlLexer::new(input_stream);
    let token_stream = CommonTokenStream::new(lexer);
    let mut parser = IdlParser::new(token_stream);

    // Build a parse tree so we can walk it recursively. The `build_parse_trees`
    // field is on `BaseParser`, accessible through `Deref`.
    parser.build_parse_trees = true;

    let tree = parser
        .idlFile()
        .map_err(|e| IdlError::Parse(format!("{e:?}")))?;

    // The parser's `input` field (on `BaseParser`, accessible through `Deref`)
    // holds the token stream. We need it for doc comment extraction (scanning
    // backwards from a token index through hidden-channel tokens).
    let token_stream = &parser.input;

    let mut registry = SchemaRegistry::new();
    let mut namespace: Option<String> = None;
    let mut imports = Vec::new();

    let idl_file = walk_idl_file(
        &tree,
        token_stream,
        &mut registry,
        &mut namespace,
        &mut imports,
    )?;

    Ok((idl_file, registry, imports))
}

// ==========================================================================
// Token Stream Type Alias
// ==========================================================================

/// Concrete token stream type produced by our lexer. Every walk function
/// threads this through so it can extract doc comments from hidden tokens.
type TS<'input> = CommonTokenStream<'input, IdlLexer<'input, InputStream<&'input str>>>;

// ==========================================================================
// Schema Properties Helper
// ==========================================================================

/// Accumulated `@name(value)` annotations from the parse tree.
///
/// Schema properties like `@namespace`, `@aliases`, and `@order` are special:
/// they are consumed by the walker and not passed through as custom properties.
/// All other annotations end up in the `properties` map.
struct SchemaProperties {
    namespace: Option<String>,
    aliases: Vec<String>,
    order: Option<FieldOrder>,
    properties: IndexMap<String, Value>,
}

impl SchemaProperties {
    fn new() -> Self {
        SchemaProperties {
            namespace: None,
            aliases: Vec::new(),
            order: None,
            properties: IndexMap::new(),
        }
    }
}

/// Walk a list of `SchemaPropertyContext` nodes and accumulate them into a
/// `SchemaProperties` struct, interpreting well-known annotations like
/// `@namespace`, `@aliases`, and `@order` specially.
fn walk_schema_properties<'input>(
    props: &[Rc<SchemaPropertyContextAll<'input>>],
    token_stream: &TS<'input>,
) -> Result<SchemaProperties> {
    let mut result = SchemaProperties::new();

    for prop in props {
        let name_ctx = prop
            .identifier()
            .ok_or_else(|| IdlError::Other("missing property name".into()))?;
        let name = identifier_text(&name_ctx);

        let value_ctx = prop
            .jsonValue()
            .ok_or_else(|| IdlError::Other("missing property value".into()))?;
        let value = walk_json_value(&value_ctx, token_stream)?;

        match name.as_str() {
            "namespace" => {
                if let Value::String(s) = &value {
                    result.namespace = Some(s.clone());
                } else {
                    return Err(IdlError::Other(
                        "@namespace must contain a string value".into(),
                    ));
                }
            }
            "aliases" => {
                if let Value::Array(arr) = &value {
                    let mut aliases = Vec::new();
                    for elem in arr {
                        if let Value::String(s) = elem {
                            aliases.push(s.clone());
                        } else {
                            return Err(IdlError::Other(
                                "@aliases must contain an array of strings".into(),
                            ));
                        }
                    }
                    result.aliases = aliases;
                } else {
                    return Err(IdlError::Other(
                        "@aliases must contain an array of strings".into(),
                    ));
                }
            }
            "order" => {
                if let Value::String(s) = &value {
                    match s.to_uppercase().as_str() {
                        "ASCENDING" => result.order = Some(FieldOrder::Ascending),
                        "DESCENDING" => result.order = Some(FieldOrder::Descending),
                        "IGNORE" => result.order = Some(FieldOrder::Ignore),
                        _ => {
                            return Err(IdlError::Other(format!(
                                "@order must be ASCENDING, DESCENDING, or IGNORE, got: {s}"
                            )));
                        }
                    }
                } else {
                    return Err(IdlError::Other(
                        "@order must contain a string value".into(),
                    ));
                }
            }
            _ => {
                result.properties.insert(name, value);
            }
        }
    }

    Ok(result)
}

// ==========================================================================
// Tree Walking Functions
// ==========================================================================

/// Top-level dispatch: protocol mode vs. schema mode.
fn walk_idl_file<'input>(
    ctx: &IdlFileContextAll<'input>,
    token_stream: &TS<'input>,
    registry: &mut SchemaRegistry,
    namespace: &mut Option<String>,
    imports: &mut Vec<ImportEntry>,
) -> Result<IdlFile> {
    // Protocol mode: the IDL contains `protocol Name { ... }`.
    if let Some(protocol_ctx) = ctx.protocolDeclaration() {
        let protocol = walk_protocol(&protocol_ctx, token_stream, registry, namespace, imports)?;
        return Ok(IdlFile::ProtocolFile(protocol));
    }

    // Schema mode: optional `namespace`, optional `schema` declaration, plus
    // named type declarations.
    if let Some(ns_ctx) = ctx.namespaceDeclaration() {
        if let Some(id_ctx) = ns_ctx.identifier() {
            let id = identifier_text(&id_ctx);
            // In schema mode the namespace declaration sets the namespace
            // directly (there is no identifier-with-dots logic like for protocols).
            *namespace = compute_namespace(&id, &None);
        }
    }

    // Collect imports (schema mode can also have them).
    collect_imports(&ctx.importStatement_all(), imports);

    // Walk named schemas. In schema mode, we register them in the registry
    // but don't collect them into a types list (there's no protocol).
    for ns_ctx in ctx.namedSchemaDeclaration_all() {
        let _schema = walk_named_schema(&ns_ctx, token_stream, registry, namespace)?;
    }

    // The main schema declaration uses `schema <fullType>;`.
    if let Some(main_ctx) = ctx.mainSchemaDeclaration() {
        if let Some(ft_ctx) = main_ctx.fullType() {
            let schema = walk_full_type(&ft_ctx, token_stream, namespace)?;
            return Ok(IdlFile::SchemaFile(schema));
        }
    }

    // If there are named schemas but no explicit `schema <type>;` declaration,
    // return the last registered schema as the "main" schema. This handles IDL
    // files like `status_schema.avdl` that define named types without an explicit
    // schema declaration — the Java parser treats these as valid schema-mode files
    // where the named types are the schema output.
    if let Some(last_schema) = registry.last() {
        return Ok(IdlFile::SchemaFile(last_schema.clone()));
    }

    Err(IdlError::Other(
        "IDL file contains neither a protocol nor a schema declaration".into(),
    ))
}

/// Walk a protocol declaration and return a complete `Protocol`.
fn walk_protocol<'input>(
    ctx: &ProtocolDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    registry: &mut SchemaRegistry,
    namespace: &mut Option<String>,
    imports: &mut Vec<ImportEntry>,
) -> Result<Protocol> {
    // Extract doc comment by scanning hidden tokens before the context's start token.
    let doc = extract_doc_from_context(ctx, token_stream);

    // Process `@namespace(...)` and other schema properties on the protocol.
    let props = walk_schema_properties(&ctx.schemaProperty_all(), token_stream)?;

    // Get the protocol name from the identifier.
    let name_ctx = ctx
        .identifier()
        .ok_or_else(|| IdlError::Other("missing protocol name".into()))?;
    let raw_identifier = identifier_text(&name_ctx);

    // Determine namespace: explicit `@namespace` overrides, otherwise if the
    // identifier contains dots, the part before the last dot is the namespace.
    *namespace = compute_namespace(&raw_identifier, &props.namespace);
    let protocol_name = extract_name(&raw_identifier);

    // Build the protocol properties (custom annotations that aren't namespace/aliases/order).
    let protocol_properties = props.properties;

    // Walk the protocol body.
    let body = ctx
        .protocolDeclarationBody()
        .ok_or_else(|| IdlError::Other("missing protocol body".into()))?;

    // Collect imports from the body.
    collect_imports(&body.importStatement_all(), imports);

    // Walk named schemas in the body. We collect the top-level schemas
    // directly into `types` rather than pulling them from the registry, because
    // the registry flattens all named types (including those nested inside
    // records). The Java tools inline nested types at their first reference
    // point; only top-level declarations appear in the `types` array.
    let mut types = Vec::new();
    for ns_ctx in body.namedSchemaDeclaration_all() {
        let schema = walk_named_schema(&ns_ctx, token_stream, registry, namespace)?;
        types.push(schema);
    }

    // Walk messages in the body.
    let mut messages = IndexMap::new();
    for msg_ctx in body.messageDeclaration_all() {
        let (msg_name, message) = walk_message(&msg_ctx, token_stream, namespace)?;
        messages.insert(msg_name, message);
    }

    Ok(Protocol {
        name: protocol_name,
        namespace: namespace.clone(),
        doc,
        properties: protocol_properties,
        types,
        messages,
    })
}

/// Dispatch to record, enum, or fixed based on the named schema declaration.
fn walk_named_schema<'input>(
    ctx: &NamedSchemaDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    registry: &mut SchemaRegistry,
    namespace: &mut Option<String>,
) -> Result<AvroSchema> {
    let schema = if let Some(fixed_ctx) = ctx.fixedDeclaration() {
        walk_fixed(&fixed_ctx, token_stream, namespace)?
    } else if let Some(enum_ctx) = ctx.enumDeclaration() {
        walk_enum(&enum_ctx, token_stream, namespace)?
    } else if let Some(record_ctx) = ctx.recordDeclaration() {
        walk_record(&record_ctx, token_stream, registry, namespace)?
    } else {
        return Err(IdlError::Other(
            "unknown named schema declaration".into(),
        ));
    };
    registry
        .register(schema.clone())
        .map_err(|e| IdlError::Other(e))?;
    Ok(schema)
}

// ==========================================================================
// Record
// ==========================================================================

fn walk_record<'input>(
    ctx: &RecordDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    _registry: &mut SchemaRegistry,
    namespace: &mut Option<String>,
) -> Result<AvroSchema> {
    let doc = extract_doc_from_context(ctx, token_stream);
    let props = walk_schema_properties(&ctx.schemaProperty_all(), token_stream)?;

    let name_ctx = ctx
        .identifier()
        .ok_or_else(|| IdlError::Other("missing record name".into()))?;
    let raw_identifier = identifier_text(&name_ctx);

    // Determine if this is a record or an error type.
    let is_error = ctx.recordType.as_ref().map_or(false, |tok| {
        tok.get_token_type() == Idl_Error
    });

    // Compute namespace: `@namespace` on the record overrides; otherwise
    // the identifier may contain dots, or we fall back to the enclosing namespace.
    let record_namespace = compute_namespace(&raw_identifier, &props.namespace)
        .or_else(|| namespace.clone());
    let record_name = extract_name(&raw_identifier);

    // Save and set the current namespace for field type resolution inside the
    // record body, then restore it afterwards.
    let saved_namespace = namespace.clone();
    if record_namespace.is_some() {
        *namespace = record_namespace.clone();
    }

    // Walk the record body to get fields.
    let body = ctx
        .recordBody()
        .ok_or_else(|| IdlError::Other("missing record body".into()))?;

    let mut fields = Vec::new();
    for field_ctx in body.fieldDeclaration_all() {
        let mut field_fields = walk_field_declaration(&field_ctx, token_stream, namespace)?;
        fields.append(&mut field_fields);
    }

    // Restore namespace.
    *namespace = saved_namespace;

    Ok(AvroSchema::Record {
        name: record_name,
        namespace: record_namespace,
        doc,
        fields,
        is_error,
        aliases: props.aliases,
        properties: props.properties,
    })
}

// ==========================================================================
// Field Declaration
// ==========================================================================

/// Walk a field declaration, which has one fullType and one or more variable
/// declarations sharing that type.
fn walk_field_declaration<'input>(
    ctx: &FieldDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    namespace: &Option<String>,
) -> Result<Vec<Field>> {
    // The doc comment on the field declaration acts as a default for variables
    // that don't have their own doc comment.
    let default_doc = extract_doc_from_context(ctx, token_stream);

    // Walk the field type.
    let full_type_ctx = ctx
        .fullType()
        .ok_or_else(|| IdlError::Other("missing field type".into()))?;
    let field_type = walk_full_type(&full_type_ctx, token_stream, namespace)?;

    // Walk each variable declaration.
    let mut fields = Vec::new();
    for var_ctx in ctx.variableDeclaration_all() {
        let field = walk_variable(&var_ctx, &field_type, &default_doc, token_stream, namespace)?;
        fields.push(field);
    }

    Ok(fields)
}

/// Walk a single variable declaration and create a `Field`.
fn walk_variable<'input>(
    ctx: &VariableDeclarationContextAll<'input>,
    field_type: &AvroSchema,
    default_doc: &Option<String>,
    token_stream: &TS<'input>,
    _namespace: &Option<String>,
) -> Result<Field> {
    // Variable-specific doc comment overrides the field-level default.
    let var_doc = extract_doc_from_context(ctx, token_stream);
    let doc = var_doc.or_else(|| default_doc.clone());

    let name_ctx = ctx
        .identifier()
        .ok_or_else(|| IdlError::Other("missing variable name".into()))?;
    let field_name = identifier_text(&name_ctx);

    // Walk the variable-level schema properties (e.g. @order, @aliases on a
    // specific variable rather than on the field type).
    let props = walk_schema_properties(&ctx.schemaProperty_all(), token_stream)?;

    // Parse the default value if present.
    let default_value = if let Some(json_ctx) = ctx.jsonValue() {
        Some(walk_json_value(&json_ctx, token_stream)?)
    } else {
        None
    };

    // Apply fixOptionalSchema: if the type is a nullable union (from `type?`)
    // and the default is non-null, reorder to put the non-null type first.
    let final_type = fix_optional_schema(field_type.clone(), &default_value);

    Ok(Field {
        name: field_name,
        schema: final_type,
        doc,
        default: default_value,
        order: props.order,
        aliases: props.aliases,
        properties: props.properties,
    })
}

// ==========================================================================
// Enum
// ==========================================================================

fn walk_enum<'input>(
    ctx: &EnumDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    enclosing_namespace: &Option<String>,
) -> Result<AvroSchema> {
    let doc = extract_doc_from_context(ctx, token_stream);
    let props = walk_schema_properties(&ctx.schemaProperty_all(), token_stream)?;

    let name_ctx = ctx
        .identifier()
        .ok_or_else(|| IdlError::Other("missing enum name".into()))?;
    let raw_identifier = identifier_text(&name_ctx);

    // If compute_namespace returns None (no explicit @namespace and no dots
    // in the identifier), fall back to the enclosing namespace.
    let enum_namespace = compute_namespace(&raw_identifier, &props.namespace)
        .or_else(|| enclosing_namespace.clone());
    let enum_name = extract_name(&raw_identifier);

    // Collect enum symbols.
    let mut symbols = Vec::new();
    for sym_ctx in ctx.enumSymbol_all() {
        if let Some(sym_name_ctx) = sym_ctx.identifier() {
            symbols.push(identifier_text(&sym_name_ctx));
        }
    }

    // Get the default symbol if present (via `= symbolName;` after the closing brace).
    let default_symbol = ctx.enumDefault().and_then(|default_ctx| {
        default_ctx
            .identifier()
            .map(|id_ctx| identifier_text(&id_ctx))
    });

    Ok(AvroSchema::Enum {
        name: enum_name,
        namespace: enum_namespace,
        doc,
        symbols,
        default: default_symbol,
        aliases: props.aliases,
        properties: props.properties,
    })
}

// ==========================================================================
// Fixed
// ==========================================================================

fn walk_fixed<'input>(
    ctx: &FixedDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    enclosing_namespace: &Option<String>,
) -> Result<AvroSchema> {
    let doc = extract_doc_from_context(ctx, token_stream);
    let props = walk_schema_properties(&ctx.schemaProperty_all(), token_stream)?;

    let name_ctx = ctx
        .identifier()
        .ok_or_else(|| IdlError::Other("missing fixed name".into()))?;
    let raw_identifier = identifier_text(&name_ctx);

    // Fall back to enclosing namespace if no explicit namespace is given.
    let fixed_namespace = compute_namespace(&raw_identifier, &props.namespace)
        .or_else(|| enclosing_namespace.clone());
    let fixed_name = extract_name(&raw_identifier);

    // Parse the size from the IntegerLiteral token.
    let size_tok = ctx.size.as_ref().ok_or_else(|| {
        IdlError::Other("missing fixed size".into())
    })?;
    let size = parse_integer_as_u32(size_tok.get_text())?;

    Ok(AvroSchema::Fixed {
        name: fixed_name,
        namespace: fixed_namespace,
        doc,
        size,
        aliases: props.aliases,
        properties: props.properties,
    })
}

// ==========================================================================
// Type Walking
// ==========================================================================

/// Walk a `fullType` node: collect schema properties, walk the inner
/// `plainType`, then apply any custom properties to the resulting schema.
fn walk_full_type<'input>(
    ctx: &FullTypeContextAll<'input>,
    token_stream: &TS<'input>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    let props = walk_schema_properties(&ctx.schemaProperty_all(), token_stream)?;

    let plain_ctx = ctx
        .plainType()
        .ok_or_else(|| IdlError::Other("missing plain type in fullType".into()))?;

    let mut schema = walk_plain_type(&plain_ctx, token_stream, namespace)?;

    // Apply custom properties to the schema. For nullable unions we apply
    // properties to the non-null branch (matching the Java behavior).
    if !props.properties.is_empty() {
        schema = apply_properties(schema, props.properties);
    }

    Ok(schema)
}

/// Dispatch to array, map, union, or nullable type.
fn walk_plain_type<'input>(
    ctx: &PlainTypeContextAll<'input>,
    token_stream: &TS<'input>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    if let Some(array_ctx) = ctx.arrayType() {
        return walk_array_type(&array_ctx, token_stream, namespace);
    }
    if let Some(map_ctx) = ctx.mapType() {
        return walk_map_type(&map_ctx, token_stream, namespace);
    }
    if let Some(union_ctx) = ctx.unionType() {
        return walk_union_type(&union_ctx, token_stream, namespace);
    }
    if let Some(nullable_ctx) = ctx.nullableType() {
        return walk_nullable_type(&nullable_ctx, token_stream, namespace);
    }
    Err(IdlError::Other("unrecognized plain type".into()))
}

/// Walk a nullable type: either a primitive type or a named reference,
/// optionally followed by `?` to make it nullable.
fn walk_nullable_type<'input>(
    ctx: &NullableTypeContextAll<'input>,
    _token_stream: &TS<'input>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    let base_type = if let Some(prim_ctx) = ctx.primitiveType() {
        walk_primitive_type(&prim_ctx)?
    } else if let Some(ref_ctx) = ctx.identifier() {
        // Named type reference.
        let type_name = identifier_text(&ref_ctx);
        let full_name = compute_full_name(namespace, &type_name);
        AvroSchema::Reference(full_name)
    } else {
        return Err(IdlError::Other("nullable type has no inner type".into()));
    };

    // If the `?` token is present, wrap in a nullable union `[null, T]`.
    if ctx.optional.is_some() {
        Ok(AvroSchema::Union {
            types: vec![AvroSchema::Null, base_type],
            is_nullable_type: true,
        })
    } else {
        Ok(base_type)
    }
}

/// Walk a primitive type keyword and return the corresponding `AvroSchema`.
fn walk_primitive_type<'input>(
    ctx: &PrimitiveTypeContextAll<'input>,
) -> Result<AvroSchema> {
    let type_tok = ctx.typeName.as_ref().ok_or_else(|| {
        IdlError::Other("missing primitive type name".into())
    })?;
    let token_type = type_tok.get_token_type();

    let schema = match token_type {
        Idl_Boolean => AvroSchema::Boolean,
        Idl_Int => AvroSchema::Int,
        Idl_Long => AvroSchema::Long,
        Idl_Float => AvroSchema::Float,
        Idl_Double => AvroSchema::Double,
        Idl_Bytes => AvroSchema::Bytes,
        Idl_String => AvroSchema::String,
        Idl_Null => AvroSchema::Null,
        Idl_Date => AvroSchema::Logical {
            logical_type: LogicalType::Date,
            properties: IndexMap::new(),
        },
        Idl_Time => AvroSchema::Logical {
            logical_type: LogicalType::TimeMillis,
            properties: IndexMap::new(),
        },
        Idl_Timestamp => AvroSchema::Logical {
            logical_type: LogicalType::TimestampMillis,
            properties: IndexMap::new(),
        },
        Idl_LocalTimestamp => AvroSchema::Logical {
            logical_type: LogicalType::LocalTimestampMillis,
            properties: IndexMap::new(),
        },
        Idl_UUID => AvroSchema::Logical {
            logical_type: LogicalType::Uuid,
            properties: IndexMap::new(),
        },
        Idl_Decimal => {
            // decimal(precision [, scale])
            let precision_tok = ctx.precision.as_ref().ok_or_else(|| {
                IdlError::Other("decimal type missing precision".into())
            })?;
            let precision = parse_integer_as_u32(precision_tok.get_text())?;

            let scale = if let Some(scale_tok) = ctx.scale.as_ref() {
                parse_integer_as_u32(scale_tok.get_text())?
            } else {
                0
            };

            AvroSchema::Logical {
                logical_type: LogicalType::Decimal { precision, scale },
                properties: IndexMap::new(),
            }
        }
        _ => {
            return Err(IdlError::Other(format!(
                "unexpected primitive type token: {token_type}"
            )));
        }
    };

    Ok(schema)
}

/// Walk `array<fullType>`.
fn walk_array_type<'input>(
    ctx: &ArrayTypeContextAll<'input>,
    token_stream: &TS<'input>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    let element_ctx = ctx
        .fullType()
        .ok_or_else(|| IdlError::Other("array type missing element type".into()))?;
    let items = walk_full_type(&element_ctx, token_stream, namespace)?;
    Ok(AvroSchema::Array {
        items: Box::new(items),
        properties: IndexMap::new(),
    })
}

/// Walk `map<fullType>`.
fn walk_map_type<'input>(
    ctx: &MapTypeContextAll<'input>,
    token_stream: &TS<'input>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    let value_ctx = ctx
        .fullType()
        .ok_or_else(|| IdlError::Other("map type missing value type".into()))?;
    let values = walk_full_type(&value_ctx, token_stream, namespace)?;
    Ok(AvroSchema::Map {
        values: Box::new(values),
        properties: IndexMap::new(),
    })
}

/// Walk `union { fullType, fullType, ... }`.
fn walk_union_type<'input>(
    ctx: &UnionTypeContextAll<'input>,
    token_stream: &TS<'input>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    let mut types = Vec::new();
    for ft_ctx in ctx.fullType_all() {
        types.push(walk_full_type(&ft_ctx, token_stream, namespace)?);
    }
    Ok(AvroSchema::Union {
        types,
        is_nullable_type: false,
    })
}

// ==========================================================================
// Message Declaration
// ==========================================================================

fn walk_message<'input>(
    ctx: &MessageDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    namespace: &Option<String>,
) -> Result<(String, Message)> {
    let doc = extract_doc_from_context(ctx, token_stream);
    let props = walk_schema_properties(&ctx.schemaProperty_all(), token_stream)?;

    // Walk the result type. `void` maps to Null.
    let result_ctx = ctx
        .resultType()
        .ok_or_else(|| IdlError::Other("missing message return type".into()))?;
    let response = walk_result_type(&result_ctx, token_stream, namespace)?;

    // The message name is stored in the `name` field of the context ext.
    let name_ctx = ctx
        .name
        .as_ref()
        .ok_or_else(|| IdlError::Other("missing message name".into()))?;
    let message_name = identifier_text(name_ctx);

    // Walk formal parameters.
    let mut request_fields = Vec::new();
    for param_ctx in ctx.formalParameter_all() {
        let param_doc = extract_doc_from_context(&*param_ctx, token_stream);

        let ft_ctx = param_ctx
            .fullType()
            .ok_or_else(|| IdlError::Other("missing parameter type".into()))?;
        let param_type = walk_full_type(&ft_ctx, token_stream, namespace)?;

        let var_ctx = param_ctx
            .variableDeclaration()
            .ok_or_else(|| IdlError::Other("missing parameter variable".into()))?;
        let field = walk_variable(&var_ctx, &param_type, &param_doc, token_stream, namespace)?;
        request_fields.push(field);
    }

    // Check for oneway.
    let one_way = ctx.oneway.is_some();

    // Check for throws clause. The `errors` field on the context ext struct
    // contains only the error type identifiers (not the message name).
    let errors = if !ctx.errors.is_empty() {
        let mut error_schemas = Vec::new();
        for error_id_ctx in &ctx.errors {
            let error_name = identifier_text(error_id_ctx);
            let full_name = compute_full_name(namespace, &error_name);
            error_schemas.push(AvroSchema::Reference(full_name));
        }
        Some(error_schemas)
    } else if one_way {
        // One-way messages have no error declarations.
        None
    } else {
        // Non-throwing messages omit the errors key entirely in the JSON
        // output. The Java Avro tools only emit `"errors"` when the message
        // explicitly declares `throws`.
        None
    };

    Ok((
        message_name,
        Message {
            doc,
            properties: props.properties,
            request: request_fields,
            response,
            errors,
            one_way,
        },
    ))
}

/// Walk a `resultType`: either `void` (produces Null) or a `plainType`.
fn walk_result_type<'input>(
    ctx: &ResultTypeContextAll<'input>,
    token_stream: &TS<'input>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    // If there's a Void token, return Null.
    if ctx.Void().is_some() {
        return Ok(AvroSchema::Null);
    }
    // Otherwise walk the plainType child.
    if let Some(plain_ctx) = ctx.plainType() {
        return walk_plain_type(&plain_ctx, token_stream, namespace);
    }
    // Fallback: void.
    Ok(AvroSchema::Null)
}

// ==========================================================================
// JSON Value Walking
// ==========================================================================

fn walk_json_value<'input>(
    ctx: &JsonValueContextAll<'input>,
    token_stream: &TS<'input>,
) -> Result<Value> {
    if let Some(obj_ctx) = ctx.jsonObject() {
        return walk_json_object(&obj_ctx, token_stream);
    }
    if let Some(arr_ctx) = ctx.jsonArray() {
        return walk_json_array(&arr_ctx, token_stream);
    }
    if let Some(lit_ctx) = ctx.jsonLiteral() {
        return walk_json_literal(&lit_ctx);
    }
    Err(IdlError::Other("empty JSON value".into()))
}

fn walk_json_literal<'input>(ctx: &JsonLiteralContextAll<'input>) -> Result<Value> {
    let tok = ctx.literal.as_ref().ok_or_else(|| {
        IdlError::Other("missing JSON literal token".into())
    })?;
    let token_type = tok.get_token_type();
    let text = tok.get_text();

    match token_type {
        Idl_Null => Ok(Value::Null),
        Idl_BTrue => Ok(Value::Bool(true)),
        Idl_BFalse => Ok(Value::Bool(false)),
        Idl_StringLiteral => {
            let unescaped = get_string_from_literal(text);
            Ok(Value::String(unescaped))
        }
        Idl_IntegerLiteral => parse_integer_literal(text),
        Idl_FloatingPointLiteral => parse_floating_point_literal(text),
        _ => Err(IdlError::Other(format!(
            "unexpected JSON literal token type: {token_type}"
        ))),
    }
}

fn walk_json_object<'input>(
    ctx: &JsonObjectContextAll<'input>,
    token_stream: &TS<'input>,
) -> Result<Value> {
    let mut map = serde_json::Map::new();
    for pair_ctx in ctx.jsonPair_all() {
        let key_tok = pair_ctx.name.as_ref().ok_or_else(|| {
            IdlError::Other("missing JSON object key".into())
        })?;
        let key = get_string_from_literal(key_tok.get_text());

        let value_ctx = pair_ctx
            .jsonValue()
            .ok_or_else(|| IdlError::Other("missing JSON object value".into()))?;
        let value = walk_json_value(&value_ctx, token_stream)?;

        map.insert(key, value);
    }
    Ok(Value::Object(map))
}

fn walk_json_array<'input>(
    ctx: &JsonArrayContextAll<'input>,
    token_stream: &TS<'input>,
) -> Result<Value> {
    let mut elements = Vec::new();
    for val_ctx in ctx.jsonValue_all() {
        elements.push(walk_json_value(&val_ctx, token_stream)?);
    }
    Ok(Value::Array(elements))
}

// ==========================================================================
// Helper Functions
// ==========================================================================

/// Extract the text from an `IdentifierContext`, removing backtick escapes.
fn identifier_text<'input>(ctx: &IdentifierContextAll<'input>) -> String {
    // The generated parser stores the matched token in `ctx.word`.
    // We use `get_text()` on the context itself as a reliable fallback.
    let text = ctx.get_text();
    text.replace('`', "")
}

/// Strip surrounding quotes from a string literal and unescape Java-style
/// escape sequences.
fn get_string_from_literal(raw: &str) -> String {
    // Strip surrounding quotes (either `"..."` or `'...'`).
    if raw.len() < 2 {
        return raw.to_string();
    }
    let inner = &raw[1..raw.len() - 1];
    unescape_java(inner)
}

/// Unescape Java-style string escape sequences.
fn unescape_java(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('b') => result.push('\u{0008}'),
                Some('f') => result.push('\u{000C}'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                Some('/') => result.push('/'),
                Some('u') => {
                    // Unicode escape: \uXXXX
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(code_point) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code_point) {
                            result.push(ch);
                        } else {
                            // Invalid code point; emit the raw escape.
                            result.push_str("\\u");
                            result.push_str(&hex);
                        }
                    } else {
                        result.push_str("\\u");
                        result.push_str(&hex);
                    }
                }
                Some(c2) if c2.is_ascii_digit() && c2 < '8' => {
                    // Octal escape: \0, \00, \000
                    let mut octal = String::new();
                    octal.push(c2);
                    // Peek at up to 2 more octal digits.
                    // We consume by collecting -- but since we can't peek easily
                    // with a char iterator, we use a simpler approach.
                    // TODO: full octal escape handling with proper lookahead.
                    // For now, single-digit octal works for \0 (null).
                    if let Ok(val) = u32::from_str_radix(&octal, 8) {
                        if let Some(ch) = char::from_u32(val) {
                            result.push(ch);
                        } else {
                            result.push('\\');
                            result.push(c2);
                        }
                    } else {
                        result.push('\\');
                        result.push(c2);
                    }
                }
                Some(other) => {
                    // Unknown escape; keep as-is.
                    result.push('\\');
                    result.push(other);
                }
                None => {
                    // Trailing backslash.
                    result.push('\\');
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Parse an integer literal (from a JSON or schema context).
///
/// Handles: decimal, hex (`0x`/`0X`), octal (`0` prefix), `L`/`l` suffix,
/// and underscore separators. Returns `Value::Number` (i32 if it fits, i64
/// otherwise).
fn parse_integer_literal(text: &str) -> Result<Value> {
    let mut number = text.replace('_', "");

    // Check for long suffix.
    let coerce_to_long = if number.ends_with('l') || number.ends_with('L') {
        number.pop();
        true
    } else {
        false
    };

    // Parse the number. Java's `Long.decode` handles "0x", "0X", "#", and
    // octal (leading "0"). We replicate that logic.
    let long_value: i64 = if number.starts_with("0x") || number.starts_with("0X") {
        let hex = &number[2..];
        i64::from_str_radix(hex, 16)
            .map_err(|e| IdlError::Other(format!("invalid hex integer literal '{text}': {e}")))?
    } else if number.starts_with('-') && (number.starts_with("-0x") || number.starts_with("-0X")) {
        let hex = &number[3..];
        let abs = i64::from_str_radix(hex, 16)
            .map_err(|e| IdlError::Other(format!("invalid hex integer literal '{text}': {e}")))?;
        -abs
    } else if number.starts_with('0') && number.len() > 1 && !number.contains('.') {
        // Octal.
        i64::from_str_radix(&number, 8)
            .map_err(|e| IdlError::Other(format!("invalid octal integer literal '{text}': {e}")))?
    } else if number.starts_with("-0") && number.len() > 2 && !number.contains('.') {
        let oct = &number[1..];
        let abs = i64::from_str_radix(oct, 8)
            .map_err(|e| IdlError::Other(format!("invalid octal integer literal '{text}': {e}")))?;
        -abs
    } else {
        number.parse::<i64>().map_err(|e| {
            IdlError::Other(format!("invalid integer literal '{text}': {e}"))
        })?
    };

    let int_value = long_value as i32;
    if coerce_to_long || int_value as i64 != long_value {
        // Doesn't fit in i32 or explicitly long -- use i64.
        Ok(serde_json::to_value(long_value)
            .map_err(|e| IdlError::Other(format!("JSON number error: {e}")))?)
    } else {
        Ok(serde_json::to_value(int_value)
            .map_err(|e| IdlError::Other(format!("JSON number error: {e}")))?)
    }
}

/// Parse a floating point literal. NaN and Infinity become `Value::String`
/// because they are not valid JSON numbers.
fn parse_floating_point_literal(text: &str) -> Result<Value> {
    let val: f64 = text.parse().map_err(|e| {
        IdlError::Other(format!("invalid floating point literal '{text}': {e}"))
    })?;

    if val.is_nan() {
        Ok(Value::String("NaN".to_string()))
    } else if val.is_infinite() {
        if val.is_sign_positive() {
            Ok(Value::String("Infinity".to_string()))
        } else {
            Ok(Value::String("-Infinity".to_string()))
        }
    } else {
        Ok(serde_json::Number::from_f64(val)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(text.to_string())))
    }
}

/// Parse an integer literal text into a u32 (for fixed size, decimal precision/scale).
fn parse_integer_as_u32(text: &str) -> Result<u32> {
    let number = text.replace('_', "");
    let value: u32 = if number.starts_with("0x") || number.starts_with("0X") {
        u32::from_str_radix(&number[2..], 16)
            .map_err(|e| IdlError::Other(format!("invalid integer '{text}': {e}")))?
    } else if number.starts_with('0') && number.len() > 1 {
        u32::from_str_radix(&number, 8)
            .map_err(|e| IdlError::Other(format!("invalid integer '{text}': {e}")))?
    } else {
        number
            .parse()
            .map_err(|e| IdlError::Other(format!("invalid integer '{text}': {e}")))?
    };
    Ok(value)
}

/// Compute the full name for a type reference: if the name already contains
/// a dot, use it as-is; otherwise prepend the current namespace.
fn compute_full_name(namespace: &Option<String>, type_name: &str) -> String {
    if type_name.contains('.') {
        type_name.to_string()
    } else {
        match namespace {
            Some(ns) if !ns.is_empty() => format!("{ns}.{type_name}"),
            _ => type_name.to_string(),
        }
    }
}

/// Given an identifier (which may contain dots like `com.example.MyType`),
/// extract just the name part (after the last dot).
fn extract_name(identifier: &str) -> String {
    match identifier.rfind('.') {
        Some(pos) => identifier[pos + 1..].to_string(),
        None => identifier.to_string(),
    }
}

/// Compute the effective namespace for a named type.
///
/// Priority:
/// 1. Explicit `@namespace("...")` annotation (passed as `explicit_namespace`).
/// 2. Dots in the identifier (the part before the last dot).
/// 3. The enclosing namespace (inherited from context -- not passed here,
///    the caller should fall back to the enclosing namespace if this returns None).
fn compute_namespace(identifier: &str, explicit_namespace: &Option<String>) -> Option<String> {
    if let Some(ns) = explicit_namespace {
        if ns.is_empty() {
            return None;
        }
        return Some(ns.clone());
    }

    match identifier.rfind('.') {
        Some(pos) => {
            let ns = &identifier[..pos];
            if ns.is_empty() {
                None
            } else {
                Some(ns.to_string())
            }
        }
        None => None,
    }
}

/// When `type?` creates a union `[null, T]` and the field's default is non-null,
/// reorder the union to `[T, null]` so that the default value matches the first
/// branch. This matches the Java `fixOptionalSchema` behavior.
fn fix_optional_schema(schema: AvroSchema, default_value: &Option<Value>) -> AvroSchema {
    match &schema {
        AvroSchema::Union {
            types,
            is_nullable_type: true,
        } if types.len() == 2 => {
            let non_null_default = match default_value {
                Some(Value::Null) | None => false,
                Some(_) => true,
            };

            if non_null_default {
                // Reorder: put the non-null type first, null second.
                let null_schema = types[0].clone();
                let non_null_schema = types[1].clone();
                AvroSchema::Union {
                    types: vec![non_null_schema, null_schema],
                    is_nullable_type: true,
                }
            } else {
                schema
            }
        }
        _ => schema,
    }
}

/// Apply custom schema properties to a schema. For nullable unions, apply them
/// to the non-null branch (matching the Java behavior where properties go on
/// `type.getTypes().get(1)` for optional types).
fn apply_properties(schema: AvroSchema, properties: IndexMap<String, Value>) -> AvroSchema {
    match schema {
        AvroSchema::Union {
            types,
            is_nullable_type: true,
        } if types.len() == 2 => {
            // Apply properties to the non-null branch (index 1 in the
            // [null, T] representation before any reordering).
            let mut new_types = types;
            new_types[1] = apply_properties_to_schema(new_types[1].clone(), properties);
            AvroSchema::Union {
                types: new_types,
                is_nullable_type: true,
            }
        }
        other => apply_properties_to_schema(other, properties),
    }
}

/// Apply properties directly to a single schema node.
fn apply_properties_to_schema(schema: AvroSchema, properties: IndexMap<String, Value>) -> AvroSchema {
    match schema {
        AvroSchema::Record {
            name,
            namespace,
            doc,
            fields,
            is_error,
            aliases,
            properties: mut existing,
        } => {
            existing.extend(properties);
            AvroSchema::Record {
                name,
                namespace,
                doc,
                fields,
                is_error,
                aliases,
                properties: existing,
            }
        }
        AvroSchema::Enum {
            name,
            namespace,
            doc,
            symbols,
            default,
            aliases,
            properties: mut existing,
        } => {
            existing.extend(properties);
            AvroSchema::Enum {
                name,
                namespace,
                doc,
                symbols,
                default,
                aliases,
                properties: existing,
            }
        }
        AvroSchema::Fixed {
            name,
            namespace,
            doc,
            size,
            aliases,
            properties: mut existing,
        } => {
            existing.extend(properties);
            AvroSchema::Fixed {
                name,
                namespace,
                doc,
                size,
                aliases,
                properties: existing,
            }
        }
        AvroSchema::Array {
            items,
            properties: mut existing,
        } => {
            existing.extend(properties);
            AvroSchema::Array {
                items,
                properties: existing,
            }
        }
        AvroSchema::Map {
            values,
            properties: mut existing,
        } => {
            existing.extend(properties);
            AvroSchema::Map {
                values,
                properties: existing,
            }
        }
        AvroSchema::Logical {
            logical_type,
            properties: mut existing,
        } => {
            existing.extend(properties);
            AvroSchema::Logical {
                logical_type,
                properties: existing,
            }
        }
        AvroSchema::AnnotatedPrimitive {
            kind,
            properties: mut existing,
        } => {
            existing.extend(properties);
            AvroSchema::AnnotatedPrimitive {
                kind,
                properties: existing,
            }
        }
        // Wrap bare primitives in AnnotatedPrimitive when properties are present.
        AvroSchema::Null => AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Null,
            properties,
        },
        AvroSchema::Boolean => AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Boolean,
            properties,
        },
        AvroSchema::Int => AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Int,
            properties,
        },
        AvroSchema::Long => AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Long,
            properties,
        },
        AvroSchema::Float => AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Float,
            properties,
        },
        AvroSchema::Double => AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Double,
            properties,
        },
        AvroSchema::Bytes => AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::Bytes,
            properties,
        },
        AvroSchema::String => AvroSchema::AnnotatedPrimitive {
            kind: PrimitiveType::String,
            properties,
        },
        // References cannot carry properties in the Avro model.
        other => other,
    }
}

/// Extract the doc comment for a parse tree context by looking at its start
/// token index. Uses the `extract_doc_comment` function from `doc_comments`
/// which scans backwards through hidden tokens.
fn extract_doc_from_context<'input, T>(ctx: &T, token_stream: &TS<'input>) -> Option<String>
where
    T: antlr4rust::parser_rule_context::ParserRuleContext<'input>,
{
    let start = ctx.start();
    let token_index = start.get_token_index();
    extract_doc_comment(token_stream, token_index)
}

/// Collect import statements into the imports list.
fn collect_imports<'input>(
    import_ctxs: &[Rc<ImportStatementContextAll<'input>>],
    imports: &mut Vec<ImportEntry>,
) {
    for import_ctx in import_ctxs {
        let kind_tok = import_ctx.importType.as_ref();
        let location_tok = import_ctx.location.as_ref();

        if let (Some(kind), Some(loc)) = (kind_tok, location_tok) {
            let import_kind = match kind.get_token_type() {
                Idl_IDL => ImportKind::Idl,
                Idl_Protocol => ImportKind::Protocol,
                Idl_Schema => ImportKind::Schema,
                _ => continue,
            };

            imports.push(ImportEntry {
                kind: import_kind,
                path: get_string_from_literal(loc.get_text()),
            });
        }
    }
}
