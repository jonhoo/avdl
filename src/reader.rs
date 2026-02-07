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

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use antlr4rust::common_token_stream::CommonTokenStream;
use antlr4rust::error_listener::ErrorListener;
use antlr4rust::parser::Parser;
use antlr4rust::recognizer::Recognizer;
use antlr4rust::token::Token;
use antlr4rust::token_factory::TokenFactory;
use antlr4rust::tree::{ParseTree, Tree};
use antlr4rust::{InputStream, TidExt};
use indexmap::IndexMap;
use serde_json::Value;

use crate::doc_comments::extract_doc_comment;
use crate::error::{IdlError, ParseDiagnostic, Result};
use crate::generated::idllexer::IdlLexer;
use crate::generated::idlparser::*;
use crate::model::protocol::{Message, Protocol};
use crate::model::schema::{AvroSchema, Field, FieldOrder, LogicalType, PrimitiveType};

// ==============================================================================
// ANTLR Error Collection
// ==============================================================================
//
// Java's IdlReader installs a custom `BaseErrorListener` that throws
// `SchemaParseException` on any syntax error, causing the tool to fail
// immediately. ANTLR's default `ConsoleErrorListener` only prints to stderr
// and lets error recovery continue, which silently produces incorrect output.
//
// We replace the default listener with `CollectingErrorListener`, which records
// each syntax error's line/column/message. After parsing, we check the
// collected errors and return an `IdlError` if any were found.

/// An ANTLR error listener that collects syntax errors into a shared `Vec`
/// instead of printing them to stderr. This lets us detect parse errors after
/// `parser.idlFile()` returns and fail with a proper error.
struct CollectingErrorListener {
    errors: Rc<RefCell<Vec<String>>>,
}

impl<'a, T: Recognizer<'a>> ErrorListener<'a, T> for CollectingErrorListener {
    fn syntax_error(
        &self,
        _recognizer: &T,
        _offending_symbol: Option<&<T::TF as TokenFactory<'a>>::Inner>,
        line: isize,
        column: isize,
        msg: &str,
        _error: Option<&antlr4rust::errors::ANTLRError>,
    ) {
        self.errors
            .borrow_mut()
            .push(format!("line {line}:{column} {msg}"));
    }
}

/// Type names that collide with Avro built-in types. Matches Java's
/// `IdlReader.INVALID_TYPE_NAMES`.
const INVALID_TYPE_NAMES: &[&str] = &[
    "boolean", "int", "long", "float", "double",
    "bytes", "string", "null", "date", "time_ms",
    "timestamp_ms", "localtimestamp_ms", "uuid",
];

// ==========================================================================
// Public API
// ==========================================================================

/// The result of parsing an IDL file -- either a protocol or a standalone schema.
#[derive(Debug)]
pub enum IdlFile {
    ProtocolFile(Protocol),
    /// A file with an explicit `schema <type>;` declaration. Serialized as a
    /// single JSON schema object.
    SchemaFile(AvroSchema),
    /// A file with bare named type declarations (no `schema` keyword and no
    /// `protocol`). Serialized as a JSON array of all named schemas, matching
    /// the Java `IdlFile.outputString()` behavior.
    NamedSchemasFile(Vec<AvroSchema>),
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

/// A declaration item in source order. Captures both import statements and
/// local type definitions interleaved exactly as they appear in the IDL file.
/// This preserves the declaration order so that the caller can register types
/// and resolve imports in the correct sequence.
#[derive(Debug, Clone)]
pub enum DeclItem {
    /// An import statement to be resolved later.
    Import(ImportEntry),
    /// A locally-defined named type (record, enum, or fixed).
    Type(AvroSchema),
}

/// Parse an Avro IDL string into an `IdlFile` and a list of declaration items.
///
/// The returned `Vec<DeclItem>` contains all imports and locally-defined types
/// in source order. The caller is responsible for processing these items in
/// order (resolving imports and registering types) to produce a correctly
/// ordered `SchemaRegistry`.
pub fn parse_idl(input: &str) -> Result<(IdlFile, Vec<DeclItem>)> {
    parse_idl_named(input, "<input>")
}

/// Parse an Avro IDL string, attaching `source_name` to any error diagnostics
/// so that error messages identify the originating file.
pub fn parse_idl_named(
    input: &str,
    source_name: &str,
) -> Result<(IdlFile, Vec<DeclItem>)> {
    let input_stream = InputStream::new(input);
    let lexer = IdlLexer::new(input_stream);
    let token_stream = CommonTokenStream::new(lexer);
    let mut parser = IdlParser::new(token_stream);

    // Build a parse tree so we can walk it recursively. The `build_parse_trees`
    // field is on `BaseParser`, accessible through `Deref`.
    parser.build_parse_trees = true;

    // Replace the default ConsoleErrorListener with a CollectingErrorListener
    // that records syntax errors. Java's IdlReader does the same thing -- it
    // removes the default listener and installs one that throws on any error.
    // We collect instead of throwing because ANTLR's error recovery may still
    // produce a usable parse tree, but we'll fail after parsing completes.
    let syntax_errors: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    parser.remove_error_listeners();
    parser.add_error_listener(Box::new(CollectingErrorListener {
        errors: Rc::clone(&syntax_errors),
    }));

    let tree = parser
        .idlFile()
        .map_err(|e| IdlError::Parse(format!("{e:?}")))?;

    // Check for ANTLR syntax errors collected during parsing. Any syntax error
    // means the input is malformed, even if ANTLR's error recovery produced a
    // parse tree. This matches Java's behavior of throwing on the first error.
    let collected_errors = syntax_errors.borrow();
    if !collected_errors.is_empty() {
        let error_msg = if collected_errors.len() == 1 {
            format!("{}: {}", source_name, collected_errors[0])
        } else {
            let mut msg = format!(
                "{}: {} syntax errors:\n",
                source_name,
                collected_errors.len()
            );
            for err in collected_errors.iter() {
                msg.push_str("  ");
                msg.push_str(err);
                msg.push('\n');
            }
            msg
        };
        return Err(IdlError::Parse(error_msg));
    }
    drop(collected_errors);

    // The parser's `input` field (on `BaseParser`, accessible through `Deref`)
    // holds the token stream. We need it for doc comment extraction (scanning
    // backwards from a token index through hidden-channel tokens).
    let token_stream = &parser.input;

    let src = SourceInfo {
        source: input,
        name: source_name,
    };

    let mut namespace: Option<String> = None;
    let mut decl_items = Vec::new();

    let idl_file = walk_idl_file(
        &tree,
        token_stream,
        &src,
        &mut namespace,
        &mut decl_items,
    )?;

    Ok((idl_file, decl_items))
}

// ==========================================================================
// Token Stream Type Alias
// ==========================================================================

/// Concrete token stream type produced by our lexer. Every walk function
/// threads this through so it can extract doc comments from hidden tokens.
type TS<'input> = CommonTokenStream<'input, IdlLexer<'input, InputStream<&'input str>>>;

// ==========================================================================
// Source Location Diagnostic Helpers
// ==========================================================================

/// Carries the original source text and a display name through the tree walk
/// so that error messages can include source location context via miette.
struct SourceInfo<'a> {
    source: &'a str,
    name: &'a str,
}

/// Construct an `IdlError::Diagnostic` with source location extracted from
/// an ANTLR parse tree context's start token.
///
/// The start token gives us a byte offset into the original source text. We
/// use the token's `get_start()` and `get_stop()` to compute a byte-level
/// `SourceSpan` that miette can render as an underlined region in the error
/// output.
fn make_diagnostic<'input>(
    src: &SourceInfo<'_>,
    ctx: &impl antlr4rust::parser_rule_context::ParserRuleContext<'input>,
    message: impl Into<String>,
) -> IdlError {
    let start_token = ctx.start();
    let offset = start_token.get_start();
    let stop = start_token.get_stop();

    // Compute a span covering at least one character. ANTLR byte offsets are
    // inclusive on both ends, so length = stop - start + 1.
    let (offset, length) = if offset >= 0 && stop >= offset {
        (offset as usize, (stop - offset + 1) as usize)
    } else if offset >= 0 {
        (offset as usize, 1)
    } else {
        // No valid position available; point at the start of the file.
        (0, 0)
    };

    let message = message.into();
    IdlError::Diagnostic(ParseDiagnostic {
        src: miette::NamedSource::new(src.name, src.source.to_string()),
        span: miette::SourceSpan::new(offset.into(), length),
        message,
    })
}

/// Like `make_diagnostic` but takes a raw `Token` reference instead of a
/// context node. Useful when the error relates to a specific token field
/// (e.g. `ctx.size`, `ctx.typeName`) rather than the whole context.
fn make_diagnostic_from_token(
    src: &SourceInfo<'_>,
    token: &impl Token,
    message: impl Into<String>,
) -> IdlError {
    let offset = token.get_start();
    let stop = token.get_stop();

    let (offset, length) = if offset >= 0 && stop >= offset {
        (offset as usize, (stop - offset + 1) as usize)
    } else if offset >= 0 {
        (offset as usize, 1)
    } else {
        (0, 0)
    };

    let message = message.into();
    IdlError::Diagnostic(ParseDiagnostic {
        src: miette::NamedSource::new(src.name, src.source.to_string()),
        span: miette::SourceSpan::new(offset.into(), length),
        message,
    })
}

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

// ==========================================================================
// Context-Sensitive Property Handling
// ==========================================================================
//
// Java's `SchemaProperties` class controls which annotation names are
// intercepted as special (`@namespace`, `@aliases`, `@order`) vs treated
// as custom properties, using boolean flags that vary per parse context.
// When a flag is false, that annotation name falls through to the custom
// properties map instead of being intercepted. This struct mirrors those
// flags.

/// Controls which annotations are intercepted as special fields vs custom
/// properties. Matches the Java `SchemaProperties(contextNamespace,
/// withNamespace, withAliases, withOrder)` constructor flags.
#[derive(Clone, Copy)]
struct PropertyContext {
    with_namespace: bool,
    with_aliases: bool,
    with_order: bool,
}

// TODO: Add reserved property name validation (issue ee3a2bca). Java's
// `JsonProperties.addProp()` rejects annotations whose names collide with
// structural JSON keys (e.g., `@doc`, `@type`, `@fields`). The reserved
// sets vary by target type (Schema vs Field vs Protocol vs Message). This
// was deferred because the exact reserved sets in avro-tools 1.12.1 don't
// always match what the git submodule source shows.

/// Context for protocol declarations: namespace is intercepted, but aliases
/// and order are treated as custom properties.
const PROTOCOL_PROPS: PropertyContext = PropertyContext {
    with_namespace: true,
    with_aliases: false,
    with_order: false,
};

/// Context for record/fixed declarations: namespace and aliases are
/// intercepted, order is a custom property.
const NAMED_TYPE_PROPS: PropertyContext = PropertyContext {
    with_namespace: true,
    with_aliases: true,
    with_order: false,
};

/// Context for enum declarations: same interception as record/fixed.
const ENUM_PROPS: PropertyContext = PropertyContext {
    with_namespace: true,
    with_aliases: true,
    with_order: false,
};

/// Context for variable declarations (field names): aliases and order are
/// intercepted, namespace is a custom property.
const VARIABLE_PROPS: PropertyContext = PropertyContext {
    with_namespace: false,
    with_aliases: true,
    with_order: true,
};

/// Context for fullType: nothing is intercepted (all annotations become
/// custom properties).
const BARE_PROPS: PropertyContext = PropertyContext {
    with_namespace: false,
    with_aliases: false,
    with_order: false,
};

/// Context for message declarations: nothing is intercepted.
const MESSAGE_PROPS: PropertyContext = PropertyContext {
    with_namespace: false,
    with_aliases: false,
    with_order: false,
};

/// Walk a list of `SchemaPropertyContext` nodes and accumulate them into a
/// `SchemaProperties` struct. Which annotations are intercepted as special
/// fields (`namespace`, `aliases`, `order`) depends on the `pctx` flags,
/// matching Java's context-sensitive `SchemaProperties` behavior.
fn walk_schema_properties<'input>(
    props: &[Rc<SchemaPropertyContextAll<'input>>],
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    pctx: PropertyContext,
) -> Result<SchemaProperties> {
    let mut result = SchemaProperties::new();

    for prop in props {
        let name_ctx = prop
            .identifier()
            .ok_or_else(|| make_diagnostic(src, &**prop, "missing property name"))?;
        let name = identifier_text(&name_ctx);

        let value_ctx = prop
            .jsonValue()
            .ok_or_else(|| make_diagnostic(src, &**prop, "missing property value"))?;
        let value = walk_json_value(&value_ctx, token_stream, src)?;

        // Intercept well-known annotations only when the context flags allow it.
        // When a flag is false, that name falls through to the custom properties
        // path (and may be rejected as reserved there).
        if pctx.with_namespace && name == "namespace" {
            if let Value::String(s) = &value {
                if result.namespace.is_some() {
                    return Err(make_diagnostic(
                        src,
                        &**prop,
                        "duplicate @namespace annotation",
                    ));
                }
                result.namespace = Some(s.clone());
            } else {
                return Err(make_diagnostic(
                    src,
                    &**prop,
                    "@namespace must contain a string value",
                ));
            }
        } else if pctx.with_aliases && name == "aliases" {
            if let Value::Array(arr) = &value {
                let mut aliases = Vec::new();
                for elem in arr {
                    if let Value::String(s) = elem {
                        aliases.push(s.clone());
                    } else {
                        return Err(make_diagnostic(
                            src,
                            &**prop,
                            "@aliases must contain an array of strings",
                        ));
                    }
                }
                result.aliases = aliases;
            } else {
                return Err(make_diagnostic(
                    src,
                    &**prop,
                    "@aliases must contain an array of strings",
                ));
            }
        } else if pctx.with_order && name == "order" {
            if let Value::String(s) = &value {
                match s.to_uppercase().as_str() {
                    "ASCENDING" => result.order = Some(FieldOrder::Ascending),
                    "DESCENDING" => result.order = Some(FieldOrder::Descending),
                    "IGNORE" => result.order = Some(FieldOrder::Ignore),
                    _ => {
                        return Err(make_diagnostic(
                            src,
                            &**prop,
                            format!(
                                "@order must be ASCENDING, DESCENDING, or IGNORE, got: {s}"
                            ),
                        ));
                    }
                }
            } else {
                return Err(make_diagnostic(
                    src,
                    &**prop,
                    "@order must contain a string value",
                ));
            }
        } else {
            result.properties.insert(name, value);
        }
    }

    Ok(result)
}

// ==========================================================================
// Tree Walking Functions
// ==========================================================================

/// Top-level dispatch: protocol mode vs. schema mode.
///
/// Instead of registering types in a SchemaRegistry during parsing, this
/// function collects all imports and local type definitions into `decl_items`
/// in source order. The caller processes these items sequentially to build a
/// correctly ordered registry.
fn walk_idl_file<'input>(
    ctx: &IdlFileContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &mut Option<String>,
    decl_items: &mut Vec<DeclItem>,
) -> Result<IdlFile> {
    // Protocol mode: the IDL contains `protocol Name { ... }`.
    if let Some(protocol_ctx) = ctx.protocolDeclaration() {
        let protocol =
            walk_protocol(&protocol_ctx, token_stream, src, namespace, decl_items)?;
        return Ok(IdlFile::ProtocolFile(protocol));
    }

    // Schema mode: optional `namespace`, optional `schema` declaration, plus
    // named type declarations.
    if let Some(ns_ctx) = ctx.namespaceDeclaration()
        && let Some(id_ctx) = ns_ctx.identifier()
    {
        let id = identifier_text(&id_ctx);
        // In schema mode, `namespace foo.bar;` sets the enclosing namespace
        // directly. Unlike protocol/record identifiers (where dots in the
        // name imply a namespace prefix), here the entire identifier IS the
        // namespace value.
        *namespace = Some(id);
    }

    // Walk the body children in source order, interleaving imports and named
    // schema declarations. The grammar rule is:
    //   (imports+=importStatement | namedSchemas+=namedSchemaDeclaration)*
    // We iterate all children to preserve the original declaration order.
    let mut local_schemas = Vec::new();
    for child in ctx.get_children() {
        if let Ok(import_ctx) = child.clone().downcast_rc::<ImportStatementContextAll<'input>>() {
            collect_single_import(&import_ctx, decl_items);
        } else if let Ok(ns_ctx) = child.downcast_rc::<NamedSchemaDeclarationContextAll<'input>>() {
            let schema = walk_named_schema_no_register(&ns_ctx, token_stream, src, namespace)?;
            local_schemas.push(schema.clone());
            decl_items.push(DeclItem::Type(schema));
        }
    }

    // The main schema declaration uses `schema <fullType>;`.
    if let Some(main_ctx) = ctx.mainSchemaDeclaration()
        && let Some(ft_ctx) = main_ctx.fullType()
    {
        let schema = walk_full_type(&ft_ctx, token_stream, src, namespace)?;
        return Ok(IdlFile::SchemaFile(schema));
    }

    // If there are named schemas but no explicit `schema <type>;` declaration,
    // return all registered schemas. This handles IDL files like
    // `status_schema.avdl` that define named types without an explicit schema
    // declaration. The Java `IdlFile.outputString()` serializes these as a JSON
    // array of all named schemas.
    if !local_schemas.is_empty() {
        return Ok(IdlFile::NamedSchemasFile(local_schemas));
    }

    Err(make_diagnostic(
        src,
        ctx,
        "IDL file contains neither a protocol nor a schema declaration",
    ))
}

/// Walk a protocol declaration and return a complete `Protocol`.
///
/// Instead of registering types immediately, this function iterates the
/// protocol body's children in source order, appending `DeclItem::Import`
/// and `DeclItem::Type` entries to `decl_items`. Messages are collected
/// directly into the protocol since they don't affect type ordering.
fn walk_protocol<'input>(
    ctx: &ProtocolDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &mut Option<String>,
    decl_items: &mut Vec<DeclItem>,
) -> Result<Protocol> {
    // Extract doc comment by scanning hidden tokens before the context's start token.
    let doc = extract_doc_from_context(ctx, token_stream);

    // Process `@namespace(...)` and other schema properties on the protocol.
    let props = walk_schema_properties(&ctx.schemaProperty_all(), token_stream, src, PROTOCOL_PROPS)?;

    // Get the protocol name from the identifier.
    let name_ctx = ctx
        .identifier()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing protocol name"))?;
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
        .ok_or_else(|| make_diagnostic(src, ctx, "missing protocol body"))?;

    // Walk the protocol body children in source order. The ANTLR grammar
    // interleaves imports, named schema declarations, and message declarations:
    //   protocolDeclarationBody: '{' (import | namedSchema | message)* '}'
    // We iterate all children and dispatch based on type, preserving the
    // original declaration order for imports and types.
    let mut messages = IndexMap::new();
    for child in body.get_children() {
        if let Ok(import_ctx) = child.clone().downcast_rc::<ImportStatementContextAll<'input>>() {
            collect_single_import(&import_ctx, decl_items);
        } else if let Ok(ns_ctx) = child.clone().downcast_rc::<NamedSchemaDeclarationContextAll<'input>>() {
            let schema = walk_named_schema_no_register(&ns_ctx, token_stream, src, namespace)?;
            decl_items.push(DeclItem::Type(schema));
        } else if let Ok(msg_ctx) = child.downcast_rc::<MessageDeclarationContextAll<'input>>() {
            let (msg_name, message) = walk_message(&msg_ctx, token_stream, src, namespace)?;
            messages.insert(msg_name, message);
        }
    }

    // The types list in the Protocol is initially empty; the caller will
    // populate it from the registry after processing all DeclItems in order.
    Ok(Protocol {
        name: protocol_name,
        namespace: namespace.clone(),
        doc,
        properties: protocol_properties,
        types: Vec::new(),
        messages,
    })
}

/// Dispatch to record, enum, or fixed based on the named schema declaration.
///
/// This function parses the named schema but does NOT register it in a
/// SchemaRegistry. The caller is responsible for registration, which allows
/// imports and local types to be registered in source order.
fn walk_named_schema_no_register<'input>(
    ctx: &NamedSchemaDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &mut Option<String>,
) -> Result<AvroSchema> {
    if let Some(fixed_ctx) = ctx.fixedDeclaration() {
        walk_fixed(&fixed_ctx, token_stream, src, namespace)
    } else if let Some(enum_ctx) = ctx.enumDeclaration() {
        walk_enum(&enum_ctx, token_stream, src, namespace)
    } else if let Some(record_ctx) = ctx.recordDeclaration() {
        walk_record(&record_ctx, token_stream, src, namespace)
    } else {
        Err(make_diagnostic(
            src,
            ctx,
            "unknown named schema declaration",
        ))
    }
}

// ==========================================================================
// Record
// ==========================================================================

// NOTE: The ANTLR grammar's `recordBody` rule only permits `fieldDeclaration`
// children — it does not include `namedSchemaDeclaration`. Therefore
// `walk_record` does not need access to the schema registry. If the grammar
// is ever extended to allow nested named schema declarations inside records,
// a `registry: &mut SchemaRegistry` parameter would need to be added back.
fn walk_record<'input>(
    ctx: &RecordDeclarationContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &mut Option<String>,
) -> Result<AvroSchema> {
    let doc = extract_doc_from_context(ctx, token_stream);
    let props = walk_schema_properties(&ctx.schemaProperty_all(), token_stream, src, NAMED_TYPE_PROPS)?;

    let name_ctx = ctx
        .identifier()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing record name"))?;
    let raw_identifier = identifier_text(&name_ctx);

    // Determine if this is a record or an error type.
    let is_error = ctx.recordType.as_ref().is_some_and(|tok| {
        tok.get_token_type() == Idl_Error
    });

    // Compute namespace: `@namespace` on the record overrides; otherwise
    // the identifier may contain dots, or we fall back to the enclosing namespace.
    let record_namespace = compute_namespace(&raw_identifier, &props.namespace)
        .or_else(|| namespace.clone());
    let record_name = extract_name(&raw_identifier);

    if INVALID_TYPE_NAMES.contains(&record_name.as_str()) {
        return Err(make_diagnostic(
            src,
            &*name_ctx,
            format!("Illegal name: {record_name}"),
        )
        .into());
    }

    // Save and set the current namespace for field type resolution inside the
    // record body, then restore it afterwards.
    let saved_namespace = namespace.clone();
    if record_namespace.is_some() {
        *namespace = record_namespace.clone();
    }

    // Walk the record body to get fields.
    let body = ctx
        .recordBody()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing record body"))?;

    let mut fields = Vec::new();
    let mut seen_field_names: HashSet<String> = HashSet::new();
    for field_ctx in body.fieldDeclaration_all() {
        let mut field_fields =
            walk_field_declaration(&field_ctx, token_stream, src, namespace)?;
        for field in &field_fields {
            if !seen_field_names.insert(field.name.clone()) {
                // Restore namespace before returning error.
                *namespace = saved_namespace;
                return Err(make_diagnostic(
                    src,
                    &*field_ctx,
                    format!(
                        "duplicate field '{}' in record '{}'",
                        field.name, record_name
                    ),
                )
                .into());
            }
        }
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
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<Vec<Field>> {
    // The doc comment on the field declaration acts as a default for variables
    // that don't have their own doc comment.
    let default_doc = extract_doc_from_context(ctx, token_stream);

    // Walk the field type.
    let full_type_ctx = ctx
        .fullType()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing field type"))?;
    let field_type = walk_full_type(&full_type_ctx, token_stream, src, namespace)?;

    // Walk each variable declaration.
    let mut fields = Vec::new();
    for var_ctx in ctx.variableDeclaration_all() {
        let field =
            walk_variable(&var_ctx, &field_type, &default_doc, token_stream, src, namespace)?;
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
    src: &SourceInfo<'_>,
    _namespace: &Option<String>,
) -> Result<Field> {
    // Variable-specific doc comment overrides the field-level default.
    let var_doc = extract_doc_from_context(ctx, token_stream);
    let doc = var_doc.or_else(|| default_doc.clone());

    let name_ctx = ctx
        .identifier()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing variable name"))?;
    let field_name = identifier_text(&name_ctx);

    // Walk the variable-level schema properties (e.g. @order, @aliases on a
    // specific variable rather than on the field type).
    let props = walk_schema_properties(&ctx.schemaProperty_all(), token_stream, src, VARIABLE_PROPS)?;

    // Parse the default value if present.
    let default_value = if let Some(json_ctx) = ctx.jsonValue() {
        Some(walk_json_value(&json_ctx, token_stream, src)?)
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
    src: &SourceInfo<'_>,
    enclosing_namespace: &Option<String>,
) -> Result<AvroSchema> {
    let doc = extract_doc_from_context(ctx, token_stream);
    let props = walk_schema_properties(&ctx.schemaProperty_all(), token_stream, src, ENUM_PROPS)?;

    let name_ctx = ctx
        .identifier()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing enum name"))?;
    let raw_identifier = identifier_text(&name_ctx);

    // If compute_namespace returns None (no explicit @namespace and no dots
    // in the identifier), fall back to the enclosing namespace.
    let enum_namespace = compute_namespace(&raw_identifier, &props.namespace)
        .or_else(|| enclosing_namespace.clone());
    let enum_name = extract_name(&raw_identifier);

    if INVALID_TYPE_NAMES.contains(&enum_name.as_str()) {
        return Err(make_diagnostic(
            src,
            &*name_ctx,
            format!("Illegal name: {enum_name}"),
        )
        .into());
    }

    // Collect enum symbols, rejecting duplicates.
    let mut symbols = Vec::new();
    let mut seen_symbols: HashSet<String> = HashSet::new();
    for sym_ctx in ctx.enumSymbol_all() {
        if let Some(sym_name_ctx) = sym_ctx.identifier() {
            let sym_name = identifier_text(&sym_name_ctx);
            if !seen_symbols.insert(sym_name.clone()) {
                return Err(make_diagnostic(
                    src,
                    &*sym_ctx,
                    format!("duplicate enum symbol: {sym_name}"),
                )
                .into());
            }
            symbols.push(sym_name);
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
    src: &SourceInfo<'_>,
    enclosing_namespace: &Option<String>,
) -> Result<AvroSchema> {
    let doc = extract_doc_from_context(ctx, token_stream);
    let props = walk_schema_properties(&ctx.schemaProperty_all(), token_stream, src, NAMED_TYPE_PROPS)?;

    let name_ctx = ctx
        .identifier()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing fixed name"))?;
    let raw_identifier = identifier_text(&name_ctx);

    // Fall back to enclosing namespace if no explicit namespace is given.
    let fixed_namespace = compute_namespace(&raw_identifier, &props.namespace)
        .or_else(|| enclosing_namespace.clone());
    let fixed_name = extract_name(&raw_identifier);

    if INVALID_TYPE_NAMES.contains(&fixed_name.as_str()) {
        return Err(make_diagnostic(
            src,
            &*name_ctx,
            format!("Illegal name: {fixed_name}"),
        )
        .into());
    }

    // Parse the size from the IntegerLiteral token.
    let size_tok = ctx.size.as_ref().ok_or_else(|| {
        make_diagnostic(src, ctx, "missing fixed size")
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
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    let props = walk_schema_properties(&ctx.schemaProperty_all(), token_stream, src, BARE_PROPS)?;

    let plain_ctx = ctx
        .plainType()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing plain type in fullType"))?;

    let schema = walk_plain_type(&plain_ctx, token_stream, src, namespace)?;

    // Type references may not be annotated. When the resolved type is a bare
    // reference to a previously-defined named type, any accumulated schema
    // properties (from annotations like `@foo("bar")`) are semantically invalid
    // -- the annotation is ambiguous (does it apply to the field or the type?).
    // The Java implementation checks this in exitNullableType (IdlReader.java
    // lines 776-777) and throws "Type references may not be annotated".
    if !props.properties.is_empty() && is_type_reference(&schema) {
        return Err(make_diagnostic(
            src,
            ctx,
            "Type references may not be annotated",
        )
        .into());
    }

    // Apply custom properties to the schema. For nullable unions we apply
    // properties to the non-null branch (matching the Java behavior).
    let schema = if !props.properties.is_empty() {
        apply_properties(schema, props.properties)
    } else {
        schema
    };

    Ok(schema)
}

/// Dispatch to array, map, union, or nullable type.
fn walk_plain_type<'input>(
    ctx: &PlainTypeContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    if let Some(array_ctx) = ctx.arrayType() {
        return walk_array_type(&array_ctx, token_stream, src, namespace);
    }
    if let Some(map_ctx) = ctx.mapType() {
        return walk_map_type(&map_ctx, token_stream, src, namespace);
    }
    if let Some(union_ctx) = ctx.unionType() {
        return walk_union_type(&union_ctx, token_stream, src, namespace);
    }
    if let Some(nullable_ctx) = ctx.nullableType() {
        return walk_nullable_type(&nullable_ctx, token_stream, src, namespace);
    }
    Err(make_diagnostic(src, ctx, "unrecognized plain type"))
}

/// Walk a nullable type: either a primitive type or a named reference,
/// optionally followed by `?` to make it nullable.
fn walk_nullable_type<'input>(
    ctx: &NullableTypeContextAll<'input>,
    _token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    let base_type = if let Some(prim_ctx) = ctx.primitiveType() {
        walk_primitive_type(&prim_ctx, src)?
    } else if let Some(ref_ctx) = ctx.identifier() {
        // Named type reference. Split the identifier into name and namespace
        // so the Reference carries them separately, enabling correct namespace
        // shortening during JSON serialization.
        let type_name = identifier_text(&ref_ctx);
        if type_name.contains('.') {
            let pos = type_name.rfind('.').expect("dot presence checked above");
            AvroSchema::Reference {
                name: type_name[pos + 1..].to_string(),
                namespace: Some(type_name[..pos].to_string()),
                properties: IndexMap::new(),
            }
        } else {
            AvroSchema::Reference {
                name: type_name.to_string(),
                namespace: namespace.clone(),
                properties: IndexMap::new(),
            }
        }
    } else {
        return Err(make_diagnostic(src, ctx, "nullable type has no inner type"));
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
    src: &SourceInfo<'_>,
) -> Result<AvroSchema> {
    let type_tok = ctx.typeName.as_ref().ok_or_else(|| {
        make_diagnostic(src, ctx, "missing primitive type name")
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
                make_diagnostic(src, ctx, "decimal type missing precision")
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
            return Err(make_diagnostic_from_token(
                src,
                type_tok.as_ref(),
                format!("unexpected primitive type token: {token_type}"),
            ));
        }
    };

    Ok(schema)
}

/// Walk `array<fullType>`.
fn walk_array_type<'input>(
    ctx: &ArrayTypeContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    let element_ctx = ctx
        .fullType()
        .ok_or_else(|| make_diagnostic(src, ctx, "array type missing element type"))?;
    let items = walk_full_type(&element_ctx, token_stream, src, namespace)?;
    Ok(AvroSchema::Array {
        items: Box::new(items),
        properties: IndexMap::new(),
    })
}

/// Walk `map<fullType>`.
fn walk_map_type<'input>(
    ctx: &MapTypeContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    let value_ctx = ctx
        .fullType()
        .ok_or_else(|| make_diagnostic(src, ctx, "map type missing value type"))?;
    let values = walk_full_type(&value_ctx, token_stream, src, namespace)?;
    Ok(AvroSchema::Map {
        values: Box::new(values),
        properties: IndexMap::new(),
    })
}

/// Walk `union { fullType, fullType, ... }`.
fn walk_union_type<'input>(
    ctx: &UnionTypeContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    let mut types = Vec::new();
    for ft_ctx in ctx.fullType_all() {
        types.push(walk_full_type(&ft_ctx, token_stream, src, namespace)?);
    }

    // Reject nested unions (Avro spec: "Unions may not immediately contain
    // other unions").
    let ft_ctxs = ctx.fullType_all();
    for (i, t) in types.iter().enumerate() {
        if matches!(t, AvroSchema::Union { .. }) {
            return Err(make_diagnostic(
                src,
                &*ft_ctxs[i],
                "Unions may not immediately contain other unions",
            ));
        }
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
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<(String, Message)> {
    let doc = extract_doc_from_context(ctx, token_stream);
    let props = walk_schema_properties(&ctx.schemaProperty_all(), token_stream, src, MESSAGE_PROPS)?;

    // Walk the result type. `void` maps to Null.
    let result_ctx = ctx
        .resultType()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing message return type"))?;
    let response = walk_result_type(&result_ctx, token_stream, src, namespace)?;

    // The message name is stored in the `name` field of the context ext.
    let name_ctx = ctx
        .name
        .as_ref()
        .ok_or_else(|| make_diagnostic(src, ctx, "missing message name"))?;
    let message_name = identifier_text(name_ctx);

    // Walk formal parameters.
    let mut request_fields = Vec::new();
    let mut seen_param_names: HashSet<String> = HashSet::new();
    for param_ctx in ctx.formalParameter_all() {
        let param_doc = extract_doc_from_context(&*param_ctx, token_stream);

        let ft_ctx = param_ctx
            .fullType()
            .ok_or_else(|| make_diagnostic(src, &*param_ctx, "missing parameter type"))?;
        let param_type = walk_full_type(&ft_ctx, token_stream, src, namespace)?;

        let var_ctx = param_ctx
            .variableDeclaration()
            .ok_or_else(|| make_diagnostic(src, &*param_ctx, "missing parameter variable"))?;
        let field =
            walk_variable(&var_ctx, &param_type, &param_doc, token_stream, src, namespace)?;
        if !seen_param_names.insert(field.name.clone()) {
            return Err(make_diagnostic(
                src,
                &*param_ctx,
                format!(
                    "duplicate parameter '{}' in message '{}'",
                    field.name, message_name
                ),
            )
            .into());
        }
        request_fields.push(field);
    }

    // Check for oneway.
    let one_way = ctx.oneway.is_some();

    // One-way messages must return void (AvroSchema::Null). The Avro specification
    // requires one-way messages to have a null response and no errors. The Java
    // implementation checks this in exitMessageDeclaration (IdlReader.java line 715).
    if one_way && response != AvroSchema::Null {
        return Err(make_diagnostic(
            src,
            ctx,
            &format!("One-way message '{}' must return void", message_name),
        )
        .into());
    }

    // Check for throws clause. The `errors` field on the context ext struct
    // contains only the error type identifiers (not the message name).
    let errors = if !ctx.errors.is_empty() {
        let mut error_schemas = Vec::new();
        for error_id_ctx in &ctx.errors {
            let error_name = identifier_text(error_id_ctx);
            if error_name.contains('.') {
                let pos = error_name.rfind('.').expect("dot presence checked above");
                error_schemas.push(AvroSchema::Reference {
                    name: error_name[pos + 1..].to_string(),
                    namespace: Some(error_name[..pos].to_string()),
                    properties: IndexMap::new(),
                });
            } else {
                error_schemas.push(AvroSchema::Reference {
                    name: error_name.to_string(),
                    namespace: namespace.clone(),
                    properties: IndexMap::new(),
                });
            }
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
    src: &SourceInfo<'_>,
    namespace: &Option<String>,
) -> Result<AvroSchema> {
    // If there's a Void token, return Null.
    if ctx.Void().is_some() {
        return Ok(AvroSchema::Null);
    }
    // Otherwise walk the plainType child.
    if let Some(plain_ctx) = ctx.plainType() {
        return walk_plain_type(&plain_ctx, token_stream, src, namespace);
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
    src: &SourceInfo<'_>,
) -> Result<Value> {
    if let Some(obj_ctx) = ctx.jsonObject() {
        return walk_json_object(&obj_ctx, token_stream, src);
    }
    if let Some(arr_ctx) = ctx.jsonArray() {
        return walk_json_array(&arr_ctx, token_stream, src);
    }
    if let Some(lit_ctx) = ctx.jsonLiteral() {
        return walk_json_literal(&lit_ctx, src);
    }
    Err(make_diagnostic(src, ctx, "empty JSON value"))
}

fn walk_json_literal<'input>(
    ctx: &JsonLiteralContextAll<'input>,
    src: &SourceInfo<'_>,
) -> Result<Value> {
    let tok = ctx.literal.as_ref().ok_or_else(|| {
        make_diagnostic(src, ctx, "missing JSON literal token")
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
        _ => Err(make_diagnostic_from_token(
            src,
            tok.as_ref(),
            format!("unexpected JSON literal token type: {token_type}"),
        )),
    }
}

fn walk_json_object<'input>(
    ctx: &JsonObjectContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
) -> Result<Value> {
    let mut map = serde_json::Map::new();
    for pair_ctx in ctx.jsonPair_all() {
        let key_tok = pair_ctx.name.as_ref().ok_or_else(|| {
            make_diagnostic(src, &*pair_ctx, "missing JSON object key")
        })?;
        let key = get_string_from_literal(key_tok.get_text());

        let value_ctx = pair_ctx
            .jsonValue()
            .ok_or_else(|| make_diagnostic(src, &*pair_ctx, "missing JSON object value"))?;
        let value = walk_json_value(&value_ctx, token_stream, src)?;

        map.insert(key, value);
    }
    Ok(Value::Object(map))
}

fn walk_json_array<'input>(
    ctx: &JsonArrayContextAll<'input>,
    token_stream: &TS<'input>,
    src: &SourceInfo<'_>,
) -> Result<Value> {
    let mut elements = Vec::new();
    for val_ctx in ctx.jsonValue_all() {
        elements.push(walk_json_value(&val_ctx, token_stream, src)?);
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
    let mut chars = s.chars().peekable();

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
                Some('u') => {
                    // Unicode escape: \u+XXXX (one or more 'u' characters
                    // followed by exactly four hex digits). The extra 'u'
                    // characters are a Java-ism that some IDL files use.
                    while chars.peek() == Some(&'u') {
                        chars.next();
                    }
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
                Some(c2) if ('0'..='7').contains(&c2) => {
                    // Octal escape: 1-3 octal digits. The grammar allows:
                    //   OctDigit OctDigit?          (1-2 digits, any octal)
                    //   [0-3] OctDigit OctDigit     (3 digits, first must be 0-3)
                    // This means a 3-digit sequence is only valid if the first
                    // digit is 0-3 (keeping the value <= \377 = 255).
                    let mut octal = String::new();
                    octal.push(c2);
                    if let Some(&next) = chars.peek()
                        && ('0'..='7').contains(&next)
                    {
                        octal.push(next);
                        chars.next();
                        // Only consume a third digit if the first was 0-3.
                        if c2 <= '3'
                            && let Some(&next2) = chars.peek()
                            && ('0'..='7').contains(&next2)
                        {
                            octal.push(next2);
                            chars.next();
                        }
                    }
                    if let Ok(val) = u32::from_str_radix(&octal, 8) {
                        if let Some(ch) = char::from_u32(val) {
                            result.push(ch);
                        } else {
                            result.push('\\');
                            result.push_str(&octal);
                        }
                    } else {
                        result.push('\\');
                        result.push_str(&octal);
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
    // Java priority: dots in the identifier always take precedence over
    // an explicit `@namespace` annotation. Only when the identifier has
    // no dots do we fall back to `@namespace`.
    if let Some(pos) = identifier.rfind('.') {
        let ns = &identifier[..pos];
        return Some(ns.to_string());
    }

    explicit_namespace.clone()
}

/// Check whether a schema is a type reference (a bare name referring to a
/// previously-defined named type) or a nullable union wrapping a type reference.
/// Used to reject annotations on type references, matching the Java behavior.
fn is_type_reference(schema: &AvroSchema) -> bool {
    match schema {
        AvroSchema::Reference { .. } => true,
        // A nullable type reference (`MD5?`) wraps the reference in a union.
        AvroSchema::Union {
            types,
            is_nullable_type: true,
        } => types.iter().any(|t| matches!(t, AvroSchema::Reference { .. })),
        _ => false,
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
            // Apply properties to the non-null branch. We find it by type
            // rather than hardcoding index 1, because nullable unions may be
            // reordered to `[T, null]` when the field has a non-null default.
            let mut new_types = types;
            let non_null_idx = if matches!(new_types[0], AvroSchema::Null) { 1 } else { 0 };
            new_types[non_null_idx] = apply_properties_to_schema(
                new_types[non_null_idx].clone(),
                properties,
            );
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
        AvroSchema::Reference {
            name,
            namespace,
            properties: mut existing,
        } => {
            existing.extend(properties);
            AvroSchema::Reference {
                name,
                namespace,
                properties: existing,
            }
        }
        // Union and other types that don't carry top-level properties.
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

/// Parse a single import statement and append it as a `DeclItem::Import` to
/// the declaration items list.
fn collect_single_import<'input>(
    import_ctx: &ImportStatementContextAll<'input>,
    decl_items: &mut Vec<DeclItem>,
) {
    let kind_tok = import_ctx.importType.as_ref();
    let location_tok = import_ctx.location.as_ref();

    if let (Some(kind), Some(loc)) = (kind_tok, location_tok) {
        let import_kind = match kind.get_token_type() {
            Idl_IDL => ImportKind::Idl,
            Idl_Protocol => ImportKind::Protocol,
            Idl_Schema => ImportKind::Schema,
            _ => return,
        };

        decl_items.push(DeclItem::Import(ImportEntry {
            kind: import_kind,
            path: get_string_from_literal(loc.get_text()),
        }));
    }
}

// ==========================================================================
// Tests
// ==========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // Octal escapes (issue #5)
    // ------------------------------------------------------------------

    #[test]
    fn octal_single_digit() {
        // \7 is octal 7 = BEL (U+0007).
        assert_eq!(unescape_java(r"\7"), "\u{0007}");
    }

    #[test]
    fn octal_two_digits() {
        // \77 is octal 77 = 63 = '?'.
        assert_eq!(unescape_java(r"\77"), "?");
    }

    #[test]
    fn octal_three_digits_newline() {
        // \012 is octal 012 = 10 = '\n'.
        assert_eq!(unescape_java(r"\012"), "\n");
    }

    #[test]
    fn octal_three_digits_uppercase_a() {
        // \101 is octal 101 = 65 = 'A'.
        assert_eq!(unescape_java(r"\101"), "A");
    }

    #[test]
    fn octal_three_digits_max() {
        // \377 is octal 377 = 255 = U+00FF (latin small letter y with diaeresis).
        assert_eq!(unescape_java(r"\377"), "\u{00FF}");
    }

    #[test]
    fn octal_high_first_digit_limits_to_two() {
        // \477 -- first digit is 4 (> 3), so only two digits are consumed:
        // \47 = octal 47 = 39 = '\'' and '7' is literal.
        assert_eq!(unescape_java(r"\477"), "'7");
    }

    #[test]
    fn octal_zero() {
        // \0 is the null character.
        assert_eq!(unescape_java(r"\0"), "\0");
    }

    // ------------------------------------------------------------------
    // Unicode escapes (multi-u support)
    // ------------------------------------------------------------------

    #[test]
    fn unicode_single_u() {
        assert_eq!(unescape_java(r"\u0041"), "A");
    }

    #[test]
    fn unicode_multi_u() {
        // \uu0041 and \uuu0041 should both produce 'A'.
        assert_eq!(unescape_java(r"\uu0041"), "A");
        assert_eq!(unescape_java(r"\uuu0041"), "A");
    }

    // ------------------------------------------------------------------
    // Slash escape removal (issue #16)
    // ------------------------------------------------------------------

    #[test]
    fn slash_is_not_unescaped() {
        // \/ is not a valid escape in the grammar. The backslash should be
        // preserved as-is, producing the two-character sequence "\/".
        assert_eq!(unescape_java(r"\/"), "\\/");
    }

    // ------------------------------------------------------------------
    // Standard escapes (regression)
    // ------------------------------------------------------------------

    #[test]
    fn standard_escapes() {
        assert_eq!(unescape_java(r"\n"), "\n");
        assert_eq!(unescape_java(r"\r"), "\r");
        assert_eq!(unescape_java(r"\t"), "\t");
        assert_eq!(unescape_java(r"\b"), "\u{0008}");
        assert_eq!(unescape_java(r"\f"), "\u{000C}");
        assert_eq!(unescape_java(r"\\"), "\\");
        assert_eq!(unescape_java(r#"\""#), "\"");
        assert_eq!(unescape_java(r"\'"), "'");
    }

    #[test]
    fn mixed_escapes() {
        assert_eq!(unescape_java(r"hello\012world"), "hello\nworld");
        assert_eq!(unescape_java(r"\101\102\103"), "ABC");
    }

    // ------------------------------------------------------------------
    // One-way messages must return void (issue #877f0e96)
    // ------------------------------------------------------------------

    #[test]
    fn oneway_nonvoid_return_is_rejected() {
        let idl = r#"
            @namespace("test")
            protocol OneWayTest {
                record Msg { string text; }
                Msg send(Msg m) oneway;
            }
        "#;
        let result = parse_idl(idl);
        assert!(result.is_err(), "expected error for one-way message with non-void return");
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("must return void"),
            "error should mention 'must return void', got: {err_msg}"
        );
    }

    #[test]
    fn oneway_void_return_is_accepted() {
        let idl = r#"
            @namespace("test")
            protocol OneWayTest {
                record Msg { string text; }
                void send(Msg m) oneway;
            }
        "#;
        let result = parse_idl(idl);
        assert!(result.is_ok(), "one-way void message should be accepted");
    }

    // ------------------------------------------------------------------
    // Annotations on type references must be rejected (issue #caeb40b1)
    // ------------------------------------------------------------------

    #[test]
    fn annotation_on_type_reference_is_rejected() {
        let idl = r#"
            @namespace("test")
            protocol P {
                fixed MD5(16);
                record R {
                    @foo("bar") MD5 hash = "0000000000000000";
                }
            }
        "#;
        let result = parse_idl(idl);
        assert!(
            result.is_err(),
            "expected error for annotation on type reference"
        );
        let err_msg = format!("{}", result.unwrap_err());
        assert!(
            err_msg.contains("may not be annotated"),
            "error should mention 'may not be annotated', got: {err_msg}"
        );
    }

    #[test]
    fn annotation_on_primitive_type_is_accepted() {
        // Annotations on primitive types are fine -- only type references
        // (bare names referring to previously-defined types) are rejected.
        let idl = r#"
            @namespace("test")
            protocol P {
                record R {
                    @foo("bar") string name;
                }
            }
        "#;
        let result = parse_idl(idl);
        assert!(result.is_ok(), "annotation on primitive type should be accepted");
    }

    // ------------------------------------------------------------------
    // ANTLR parse errors must be fatal (issue #1b49abf1)
    // ------------------------------------------------------------------

    #[test]
    fn missing_semicolon_is_rejected() {
        // A missing semicolon after a field declaration is a syntax error.
        // ANTLR can recover and produce output, but we must detect the error
        // and fail. Java exits 1 with SchemaParseException.
        let idl = r#"
            @namespace("test")
            protocol P {
                record R { string name }
            }
        "#;
        let result = parse_idl(idl);
        assert!(
            result.is_err(),
            "expected error for missing semicolon in record field"
        );
    }

    #[test]
    fn valid_protocol_still_accepted() {
        // Sanity check: valid IDL with correct syntax must still parse.
        let idl = r#"
            @namespace("test")
            protocol P {
                record R { string name; }
            }
        "#;
        let result = parse_idl(idl);
        assert!(result.is_ok(), "valid protocol should be accepted, got: {:?}", result.err());
    }
}
