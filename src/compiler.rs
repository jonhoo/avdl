// ==============================================================================
// Library API: Two Builders for Compiling Avro IDL
// ==============================================================================
//
// This module provides the public API for the avdl library. Two builders mirror
// the CLI's two subcommands:
//
//   - `Idl`          — compile `.avdl` → protocol JSON (.avpr) or schema JSON (.avsc)
//   - `Idl2Schemata` — extract individual `.avsc` schemas from `.avdl`
//
// Both follow the non-consuming `&mut self` builder pattern (C-BUILDER), so the
// same builder can be reused across multiple calls. All mutable compilation
// state (registry, import context) is created fresh per call.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use miette::Context;
use serde_json::Value;

use crate::error::ParseDiagnostic;
use crate::import::{ImportContext, import_protocol, import_schema};
use crate::model::json::{build_lookup, protocol_to_json, schema_to_json};
use crate::model::protocol::Message;
use crate::model::schema::validate_record_field_defaults;
use crate::reader::{DeclItem, IdlFile, ImportKind, parse_idl_named};
use crate::resolve::SchemaRegistry;

// ==============================================================================
// `Idl` Builder — mirrors `avdl idl`
// ==============================================================================

/// Builder for compiling Avro IDL to protocol (`.avpr`) or schema (`.avsc`) JSON.
///
/// Follows the non-consuming builder pattern (like [`std::process::Command`]):
/// configuration and terminal methods both take `&mut self`, enabling both
/// chained one-liners and multi-step configuration.
///
/// # Examples
///
/// ```no_run
/// use avdl::Idl;
///
/// // One-liner with chaining:
/// let output = Idl::new()
///     .import_dir("schemas/shared/")
///     .convert("schemas/service.avdl")?;
/// println!("{}", serde_json::to_string_pretty(&output.json)?);
///
/// // Multi-step configuration:
/// let mut idl = Idl::new();
/// idl.import_dir("schemas/shared/");
/// idl.import_dir("schemas/common/");
/// let output = idl.convert("schemas/service.avdl")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct Idl {
    import_dirs: Vec<PathBuf>,
}

/// Result of compiling an Avro IDL source.
///
/// The shape of [`json`](IdlOutput::json) depends on the IDL input:
/// - **Protocol** (`protocol Foo { ... }`) — a JSON object matching the `.avpr`
///   format, with `"protocol"`, `"types"`, and `"messages"` keys.
/// - **Standalone schema** (`schema int;`) — a single `.avsc` JSON value
///   (string, object, or array).
/// - **Multiple named schemas** (bare record/enum/fixed declarations) — a JSON
///   array of `.avsc` values.
pub struct IdlOutput {
    /// The compiled JSON (`.avpr` object, `.avsc` value, or JSON array).
    pub json: Value,
    /// Non-fatal warnings from parsing (e.g., orphaned doc comments).
    ///
    /// Each warning is a [`miette::Report`] with `Severity::Warning` set.
    /// Print with `eprintln!("{report:?}")` for rich diagnostic output
    /// including source spans and labels.
    pub warnings: Vec<miette::Report>,
}

/// Shows the JSON shape and warning count without dumping the full graphical
/// rendering of every `miette::Report` (whose `Debug` impl is verbose).
impl std::fmt::Debug for IdlOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdlOutput")
            .field("json", &self.json)
            .field("warnings", &format_args!("[{} warnings]", self.warnings.len()))
            .finish()
    }
}

impl Default for Idl {
    fn default() -> Self {
        Self::new()
    }
}

impl Idl {
    /// Create a new builder with no import directories.
    pub fn new() -> Self {
        Idl {
            import_dirs: Vec::new(),
        }
    }

    /// Add an import search directory. Searched in order added, after the input
    /// file's parent directory.
    pub fn import_dir(&mut self, dir: impl Into<PathBuf>) -> &mut Self {
        self.import_dirs.push(dir.into());
        self
    }

    /// Compile a `.avdl` file to JSON.
    pub fn convert(&mut self, path: impl AsRef<Path>) -> miette::Result<IdlOutput> {
        let path = path.as_ref();
        let source = fs::read_to_string(path)
            .map_err(|e| miette::miette!("{e}"))
            .with_context(|| format!("read {}", path.display()))?;

        let source_name = path.display().to_string();
        let dir = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let dir = dir.canonicalize().unwrap_or(dir);
        let canonical_path = path.canonicalize().ok();

        self.convert_impl(&source, &source_name, &dir, canonical_path)
    }

    /// Compile an IDL source string to JSON. Uses `"<input>"` as the source
    /// name in diagnostics.
    pub fn convert_str(&mut self, source: &str) -> miette::Result<IdlOutput> {
        self.convert_str_named(source, "<input>")
    }

    /// Compile an IDL source string to JSON with a custom source name for
    /// diagnostics.
    pub fn convert_str_named(&mut self, source: &str, name: &str) -> miette::Result<IdlOutput> {
        let cwd = std::env::current_dir()
            .map_err(|e| miette::miette!("{e}"))
            .context("determine current directory")?;
        self.convert_impl(source, name, &cwd, None)
    }

    /// Shared implementation for `convert` and `convert_str_named`.
    fn convert_impl(
        &self,
        source: &str,
        source_name: &str,
        input_dir: &Path,
        input_path: Option<PathBuf>,
    ) -> miette::Result<IdlOutput> {
        let mut ctx = CompileContext::new(&self.import_dirs);

        let (idl_file, registry, warnings) =
            parse_and_resolve(source, source_name, input_dir, input_path, &mut ctx)?;

        // Serialize the parsed IDL to JSON. Protocols become .avpr, standalone
        // schemas become .avsc.
        let json = match &idl_file {
            IdlFile::Protocol(protocol) => protocol_to_json(protocol),
            IdlFile::Schema(schema) => {
                let registry_schemas: Vec<_> = registry.schemas().cloned().collect();
                let lookup = build_lookup(&registry_schemas, None);
                schema_to_json(schema, &mut HashSet::new(), None, &lookup)
            }
            IdlFile::NamedSchemas(schemas) => {
                let registry_schemas: Vec<_> = registry.schemas().cloned().collect();
                let lookup = build_lookup(&registry_schemas, None);
                let json_schemas: Vec<Value> = schemas
                    .iter()
                    .map(|s| schema_to_json(s, &mut HashSet::new(), None, &lookup))
                    .collect();
                Value::Array(json_schemas)
            }
        };

        // Validate that all type references resolved. Unresolved references
        // indicate missing imports, undefined types, or cross-namespace
        // references that need fully-qualified names.
        validate_all_references(&idl_file, &registry, source, source_name)?;

        Ok(IdlOutput {
            json,
            warnings,
        })
    }
}

// ==============================================================================
// `Idl2Schemata` Builder — mirrors `avdl idl2schemata`
// ==============================================================================

/// A single named schema extracted from an Avro IDL file.
///
/// Each schema is fully self-contained: referenced types are inlined on first
/// occurrence, so the JSON value can be written directly to an `.avsc` file
/// without needing any other schema definitions.
#[derive(Debug)]
pub struct NamedSchema {
    /// Simple name of the schema (the `.avsc` filename stem).
    pub name: String,
    /// Self-contained JSON representation with all referenced types inlined on
    /// first occurrence.
    pub schema: Value,
}

/// Result of extracting individual schemas from Avro IDL.
///
/// Contains all named schemas (records, enums, fixed) from the IDL source,
/// in declaration order. Each schema is self-contained and suitable for
/// writing to its own `.avsc` file.
pub struct SchemataOutput {
    /// Named schemas in declaration order.
    pub schemas: Vec<NamedSchema>,
    /// Non-fatal warnings from parsing.
    ///
    /// Each warning is a [`miette::Report`] with `Severity::Warning` set.
    /// Print with `eprintln!("{report:?}")` for rich diagnostic output
    /// including source spans and labels.
    pub warnings: Vec<miette::Report>,
}

impl std::fmt::Debug for SchemataOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SchemataOutput")
            .field("schemas", &self.schemas)
            .field("warnings", &format_args!("[{} warnings]", self.warnings.len()))
            .finish()
    }
}

/// Builder for extracting individual `.avsc` schemas from Avro IDL.
///
/// # Examples
///
/// ```no_run
/// use avdl::Idl2Schemata;
///
/// let output = Idl2Schemata::new()
///     .extract("schemas/service.avdl")?;
/// for s in &output.schemas {
///     println!("{}.avsc: {}", s.name, s.schema);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct Idl2Schemata {
    import_dirs: Vec<PathBuf>,
}

impl Default for Idl2Schemata {
    fn default() -> Self {
        Self::new()
    }
}

impl Idl2Schemata {
    /// Create a new builder with no import directories.
    pub fn new() -> Self {
        Idl2Schemata {
            import_dirs: Vec::new(),
        }
    }

    /// Add an import search directory.
    pub fn import_dir(&mut self, dir: impl Into<PathBuf>) -> &mut Self {
        self.import_dirs.push(dir.into());
        self
    }

    /// Extract named schemas from a `.avdl` file or a directory of `.avdl`
    /// files. When given a directory, recursively walks it for `.avdl` files
    /// (using [`walkdir`]).
    pub fn extract(&mut self, path: impl AsRef<Path>) -> miette::Result<SchemataOutput> {
        let path = path.as_ref();

        if path.is_dir() {
            return self.extract_directory(path);
        }

        let source = fs::read_to_string(path)
            .map_err(|e| miette::miette!("{e}"))
            .with_context(|| format!("read {}", path.display()))?;

        let source_name = path.display().to_string();
        let dir = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let dir = dir.canonicalize().unwrap_or(dir);
        let canonical_path = path.canonicalize().ok();

        self.extract_impl(&source, &source_name, &dir, canonical_path)
    }

    /// Extract named schemas from an IDL source string.
    pub fn extract_str(&mut self, source: &str) -> miette::Result<SchemataOutput> {
        self.extract_str_named(source, "<input>")
    }

    /// Extract named schemas from an IDL source string with a custom source
    /// name for diagnostics.
    pub fn extract_str_named(
        &mut self,
        source: &str,
        name: &str,
    ) -> miette::Result<SchemataOutput> {
        let cwd = std::env::current_dir()
            .map_err(|e| miette::miette!("{e}"))
            .context("determine current directory")?;
        self.extract_impl(source, name, &cwd, None)
    }

    /// Recursively walk a directory for `.avdl` files and extract schemas from
    /// each. Each file is processed independently with its own registry.
    /// Results are concatenated.
    fn extract_directory(&self, dir: &Path) -> miette::Result<SchemataOutput> {
        let mut all_schemas = Vec::new();
        let mut all_warnings = Vec::new();

        let mut avdl_paths: Vec<PathBuf> = Vec::new();
        for entry in walkdir::WalkDir::new(dir)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("avdl") {
                avdl_paths.push(path.to_path_buf());
            }
        }

        for avdl_path in &avdl_paths {
            let source = fs::read_to_string(avdl_path)
                .map_err(|e| miette::miette!("{e}"))
                .with_context(|| format!("read {}", avdl_path.display()))?;

            let source_name = avdl_path.display().to_string();
            let file_dir = avdl_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            let file_dir = file_dir.canonicalize().unwrap_or(file_dir);
            let canonical_path = avdl_path.canonicalize().ok();

            let output = self.extract_impl(&source, &source_name, &file_dir, canonical_path)?;
            all_schemas.extend(output.schemas);
            all_warnings.extend(output.warnings);
        }

        Ok(SchemataOutput {
            schemas: all_schemas,
            warnings: all_warnings,
        })
    }

    /// Shared implementation for `extract` and `extract_str_named`.
    fn extract_impl(
        &self,
        source: &str,
        source_name: &str,
        input_dir: &Path,
        input_path: Option<PathBuf>,
    ) -> miette::Result<SchemataOutput> {
        let mut ctx = CompileContext::new(&self.import_dirs);

        let (idl_file, registry, warnings) =
            parse_and_resolve(source, source_name, input_dir, input_path, &mut ctx)?;

        // Build a lookup table from all registered schemas so that references
        // within each schema can be resolved and inlined.
        let registry_schemas: Vec<_> = registry.schemas().cloned().collect();
        let all_lookup = build_lookup(&registry_schemas, None);

        // Serialize each named schema independently with fresh `known_names`,
        // matching Java's `Schema.toString(true)` which creates a fresh
        // `HashSet` per call. This ensures each `.avsc` file is self-contained.
        let mut schemas = Vec::new();
        for schema in registry.schemas() {
            let simple_name = match schema.name() {
                Some(n) => n.to_string(),
                None => continue,
            };
            let mut known_names = HashSet::new();
            let json_value = schema_to_json(schema, &mut known_names, None, &all_lookup);
            schemas.push(NamedSchema {
                name: simple_name,
                schema: json_value,
            });
        }

        // Validate that all type references resolved.
        validate_all_references(&idl_file, &registry, source, source_name)?;

        Ok(SchemataOutput {
            schemas,
            warnings,
        })
    }
}

// ==============================================================================
// Shared: Parsing, Import Resolution, and Reference Validation
// ==============================================================================

/// Groups the mutable state threaded through `process_decl_items` and
/// `resolve_single_import`, replacing the long parameter lists in the
/// original code.
struct CompileContext {
    registry: SchemaRegistry,
    import_ctx: ImportContext,
    messages: HashMap<String, Message>,
    warnings: Vec<miette::Report>,
}

impl CompileContext {
    fn new(import_dirs: &[PathBuf]) -> Self {
        CompileContext {
            registry: SchemaRegistry::new(),
            import_ctx: ImportContext::new(import_dirs.to_vec()),
            messages: HashMap::new(),
            warnings: Vec::new(),
        }
    }
}

/// Parse IDL source and recursively resolve all imports.
///
/// Returns the parsed IDL file, schema registry, and any warnings. The key
/// insight for correct type ordering: `parse_idl_named` returns declaration items
/// (imports and local types) in source order, and we process them
/// sequentially, so the registry reflects declaration order.
fn parse_and_resolve(
    source: &str,
    source_name: &str,
    input_dir: &Path,
    input_path: Option<PathBuf>,
    ctx: &mut CompileContext,
) -> miette::Result<(IdlFile, SchemaRegistry, Vec<miette::Report>)> {
    let (idl_file, decl_items, local_warnings) =
        parse_idl_named(source, source_name).context("parse IDL source")?;

    // Pre-size the registry based on the number of type declarations in this
    // file. This avoids incremental reallocation of the backing IndexMap.
    // Imports may add more types, but pre-sizing for the local count handles
    // the common case and reduces overall reallocation pressure.
    let type_count = decl_items
        .iter()
        .filter(|item| matches!(item, DeclItem::Type(..)))
        .count();
    if type_count > 0 {
        ctx.registry.reserve(type_count);
    }

    // Mark the initial input file as "imported" so that self-imports are
    // detected as cycles and silently skipped.
    if let Some(path) = input_path {
        ctx.import_ctx.mark_imported(&path);
    }

    // Process declaration items in source order: resolve imports when
    // encountered, register local types when encountered.
    process_decl_items(
        &decl_items,
        &mut ctx.registry,
        &mut ctx.import_ctx,
        input_dir,
        &mut ctx.messages,
        &mut ctx.warnings,
        source,
        source_name,
    )?;

    // Convert the local `Warning` values from the top-level parse into
    // `miette::Report`s, then append any import-derived reports that
    // `process_decl_items` accumulated in `ctx.warnings`.
    let mut warnings: Vec<miette::Report> = local_warnings
        .into_iter()
        .map(miette::Report::new)
        .collect();
    warnings.append(&mut ctx.warnings);

    // For protocol files, rebuild the types list from the registry (which now
    // includes imported types in declaration order) and prepend imported
    // messages before the protocol's own messages.
    let idl_file = match idl_file {
        IdlFile::Protocol(mut protocol) => {
            protocol.types = ctx.registry.schemas().cloned().collect();
            let own_messages = std::mem::take(&mut protocol.messages);
            protocol.messages = std::mem::take(&mut ctx.messages);
            protocol.messages.extend(own_messages);
            IdlFile::Protocol(protocol)
        }
        other => other,
    };

    // Move the registry out; the caller owns it now. Replace with a fresh one
    // so `ctx` is left in a valid state (although typically not reused).
    let registry = std::mem::take(&mut ctx.registry);

    Ok((idl_file, registry, warnings))
}

/// Process declaration items (imports and local types) in source order.
#[allow(clippy::too_many_arguments)]
fn process_decl_items(
    decl_items: &[DeclItem],
    registry: &mut SchemaRegistry,
    import_ctx: &mut ImportContext,
    current_dir: &Path,
    messages: &mut HashMap<String, Message>,
    warnings: &mut Vec<miette::Report>,
    source: &str,
    source_name: &str,
) -> miette::Result<()> {
    for item in decl_items {
        match item {
            DeclItem::Import(import) => {
                resolve_single_import(
                    import,
                    registry,
                    import_ctx,
                    current_dir,
                    messages,
                    warnings,
                    source,
                    source_name,
                )?;
            }
            DeclItem::Type(schema, span) => {
                if let Err(msg) = registry.register(schema.clone()) {
                    if let Some(span) = span {
                        return Err(ParseDiagnostic {
                            src: miette::NamedSource::new(source_name, source.to_string()),
                            span: *span,
                            message: msg,
                        }
                        .into());
                    }
                    return Err(miette::miette!("{msg}"));
                }

                // Validate field defaults for Reference-typed fields now that
                // the registry contains all previously-registered types.
                let errors = validate_record_field_defaults(schema, |full_name| {
                    registry.lookup(full_name).cloned()
                });
                if let Some((field_name, reason)) = errors.into_iter().next() {
                    let type_name = schema.full_name().unwrap_or(Cow::Borrowed("<unknown>"));
                    let msg = format!(
                        "Invalid default for field `{field_name}` in `{type_name}`: {reason}"
                    );
                    if let Some(span) = span {
                        return Err(ParseDiagnostic {
                            src: miette::NamedSource::new(source_name, source.to_string()),
                            span: *span,
                            message: msg,
                        }
                        .into());
                    }
                    return Err(miette::miette!("{msg}"));
                }
            }
        }
    }

    Ok(())
}

/// Resolve a single import entry, registering schemas and merging messages
/// into the current protocol.
#[allow(clippy::too_many_arguments)]
fn resolve_single_import(
    import: &crate::reader::ImportEntry,
    registry: &mut SchemaRegistry,
    import_ctx: &mut ImportContext,
    current_dir: &Path,
    messages: &mut HashMap<String, Message>,
    warnings: &mut Vec<miette::Report>,
    source: &str,
    source_name: &str,
) -> miette::Result<()> {
    let resolved_path = match import_ctx.resolve_import(&import.path, current_dir) {
        Ok(p) => p,
        Err(e) => {
            if let Some(span) = import.span {
                return Err(ParseDiagnostic {
                    src: miette::NamedSource::new(source_name, source.to_string()),
                    span,
                    message: format!("{e}"),
                }
                .into());
            }
            return Err(e).with_context(|| format!("resolve import `{}`", import.path));
        }
    };

    // Skip files we've already imported (cycle prevention).
    if import_ctx.mark_imported(&resolved_path) {
        return Ok(());
    }

    let import_dir = resolved_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    match import.kind {
        ImportKind::Protocol => {
            let imported_messages = import_protocol(&resolved_path, registry).map_err(|e| {
                wrap_import_error(
                    e,
                    import.span,
                    source,
                    source_name,
                    &resolved_path,
                    "protocol",
                )
            })?;
            messages.extend(imported_messages);
        }
        ImportKind::Schema => {
            import_schema(&resolved_path, registry).map_err(|e| {
                wrap_import_error(
                    e,
                    import.span,
                    source,
                    source_name,
                    &resolved_path,
                    "schema",
                )
            })?;
        }
        ImportKind::Idl => {
            let imported_source = fs::read_to_string(&resolved_path)
                .map_err(|e| miette::miette!("{e}"))
                .with_context(|| format!("read imported IDL {}", resolved_path.display()))?;

            let imported_name = resolved_path.display().to_string();
            let (imported_idl, nested_decl_items, import_warnings) =
                parse_idl_named(&imported_source, &imported_name)
                    .with_context(|| format!("parse imported IDL {}", resolved_path.display()))?;

            // Propagate warnings from the imported file, wrapping each with the
            // import filename as context so the user knows where they originated.
            let import_file_name = resolved_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(import.path.as_str());
            for w in import_warnings {
                warnings.push(miette::Report::new(w).wrap_err(format!("{import_file_name}")));
            }

            // If the imported IDL is a protocol, merge its messages.
            if let IdlFile::Protocol(imported_protocol) = &imported_idl {
                messages.extend(imported_protocol.messages.clone());
            }

            // Recursively process declaration items from the imported file.
            process_decl_items(
                &nested_decl_items,
                registry,
                import_ctx,
                &import_dir,
                messages,
                warnings,
                &imported_source,
                &imported_name,
            )
            .with_context(|| {
                format!("resolve nested imports from `{}`", resolved_path.display())
            })?;
        }
    }

    Ok(())
}

/// Wrap an import error with the IDL source span of the import statement.
///
/// When the import statement's byte range (`span`) is available, the returned
/// error includes a `ParseDiagnostic` pointing at the import line in the IDL
/// source. This gives the user a direct pointer to which import triggered the
/// failure, even when the root cause is inside the imported JSON file.
fn wrap_import_error(
    error: miette::Report,
    span: Option<miette::SourceSpan>,
    source: &str,
    source_name: &str,
    resolved_path: &Path,
    kind: &str,
) -> miette::Report {
    if let Some(span) = span {
        let diag = ParseDiagnostic {
            src: miette::NamedSource::new(source_name, source.to_string()),
            span,
            message: format!("import {} {}", kind, resolved_path.display()),
        };
        error.context(diag)
    } else {
        error.context(format!("import {} {}", kind, resolved_path.display()))
    }
}

/// Validate that all type references in the IDL file and registry resolved.
///
/// Unresolved references indicate missing imports, undefined types, or
/// cross-namespace references that need fully-qualified names. Java's
/// `IdlReader` treats these as fatal errors.
///
/// When a reference carries a source span (from the parser), the error is
/// reported as a `ParseDiagnostic` with source highlighting. References
/// without spans (from JSON imports) fall back to a plain text message.
fn validate_all_references(
    idl_file: &IdlFile,
    registry: &SchemaRegistry,
    source: &str,
    source_name: &str,
) -> miette::Result<()> {
    let mut unresolved = registry.validate_references();

    // `SchemaFile` and `NamedSchemasFile` store their top-level schemas outside
    // the registry, so `validate_references` alone misses unresolved references
    // in them.
    match idl_file {
        IdlFile::Schema(schema) => {
            unresolved.extend(registry.validate_schema(schema));
        }
        IdlFile::NamedSchemas(schemas) => {
            for schema in schemas {
                unresolved.extend(registry.validate_schema(schema));
            }
        }
        IdlFile::Protocol(_) => {}
    }

    unresolved.sort_by(|a, b| a.0.cmp(&b.0));
    unresolved.dedup_by(|a, b| a.0 == b.0);

    if unresolved.is_empty() {
        return Ok(());
    }

    // Report the first unresolved reference that has a source span as a
    // ParseDiagnostic for rich source-highlighted output. If none have spans,
    // fall back to a plain message listing all unresolved names.
    if let Some((name, Some(span))) = unresolved.iter().find(|(_, s)| s.is_some()) {
        return Err(ParseDiagnostic {
            src: miette::NamedSource::new(source_name, source.to_string()),
            span: *span,
            message: format!("Undefined name: {name}"),
        }
        .into());
    }

    let names: Vec<&str> = unresolved.iter().map(|(name, _)| name.as_str()).collect();
    miette::bail!("Undefined name: {}", names.join(", "));
}

// ==============================================================================
// Unit Tests
// ==============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_str_simple_protocol() {
        let output = Idl::new()
            .convert_str(r#"protocol Empty { }"#)
            .expect("should parse empty protocol");
        assert_eq!(output.json["protocol"], "Empty");
        assert!(output.warnings.is_empty());
    }

    #[test]
    fn convert_str_with_record() {
        let output = Idl::new()
            .convert_str(
                r#"
                @namespace("org.example")
                protocol Svc {
                    record User { string name; }
                }
                "#,
            )
            .expect("should parse protocol with record");

        assert_eq!(output.json["protocol"], "Svc");
        assert_eq!(output.json["namespace"], "org.example");
        let types = output.json["types"].as_array().expect("should have types");
        assert_eq!(types.len(), 1);
        assert_eq!(types[0]["name"], "User");
    }

    #[test]
    fn convert_str_schema_mode() {
        let output = Idl::new()
            .convert_str("schema int;")
            .expect("should parse schema mode");
        assert_eq!(output.json, "int");
    }

    #[test]
    fn convert_str_undefined_type_error() {
        let result = Idl::new().convert_str(
            r#"
            @namespace("test")
            protocol P {
                record R { MissingType field; }
            }
            "#,
        );
        let err = result.unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("Undefined name"),
            "should report undefined name, got: {msg}"
        );
    }

    #[test]
    fn convert_str_named_custom_source_name() {
        let result = Idl::new().convert_str_named(r#"protocol { }"#, "my-test.avdl");
        // This should fail because protocol requires a name. The error should
        // reference the custom source name.
        assert!(result.is_err());
    }

    #[test]
    fn extract_str_simple_protocol() {
        let output = Idl2Schemata::new()
            .extract_str(
                r#"
                @namespace("test")
                protocol P {
                    record Foo { string name; }
                    enum Color { RED, GREEN, BLUE }
                }
                "#,
            )
            .expect("should extract schemas");

        assert_eq!(output.schemas.len(), 2);
        assert_eq!(output.schemas[0].name, "Foo");
        assert_eq!(output.schemas[0].schema["type"], "record");
        assert_eq!(output.schemas[1].name, "Color");
        assert_eq!(output.schemas[1].schema["type"], "enum");
    }

    #[test]
    fn extract_str_undefined_type_error() {
        let result = Idl2Schemata::new().extract_str(
            r#"
            @namespace("test")
            protocol P {
                record R { MissingType field; }
            }
            "#,
        );
        let err = result.unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("Undefined name"),
            "should report undefined name, got: {msg}"
        );
    }

    #[test]
    fn builder_reuse() {
        let mut idl = Idl::new();

        let out1 = idl
            .convert_str("protocol A { }")
            .expect("first call should succeed");
        assert_eq!(out1.json["protocol"], "A");

        let out2 = idl
            .convert_str("protocol B { }")
            .expect("second call should succeed");
        assert_eq!(out2.json["protocol"], "B");
    }

    #[test]
    fn default_trait() {
        // Verify Default is implemented.
        let _idl = Idl::default();
        let _schemata = Idl2Schemata::default();
    }
}
