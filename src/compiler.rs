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
//
// Internally, both delegate to a shared `IdlCompiler` struct that owns the
// common builder state (import directories, accumulated warnings) and provides
// the shared compilation preamble (file reading, path resolution, parsing, and
// import resolution). The type-specific serialization logic lives in each
// builder's `*_impl` method.

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
// Shared `IdlCompiler` — common builder state and compilation preamble
// ==============================================================================

/// Shared inner struct that owns the builder state common to both [`Idl`] and
/// [`Idl2Schemata`]: import directories and accumulated warnings.
///
/// This is intentionally private — the public API surface is through `Idl` and
/// `Idl2Schemata`, which wrap this struct and add their type-specific
/// serialization logic.
struct IdlCompiler {
    import_dirs: Vec<PathBuf>,
    /// Warnings accumulated during the most recent compilation call. Populated
    /// even when the call returns `Err`, so the CLI can emit warnings before
    /// propagating the error.
    accumulated_warnings: Vec<miette::Report>,
}

/// The result of a successful compilation preamble: the parsed IDL file and
/// schema registry, plus any non-fatal warnings. Passed to the type-specific
/// serialization logic in `Idl::convert_impl` and `Idl2Schemata::extract_impl`.
///
/// Includes the source text and source name so that type-specific logic (e.g.,
/// `Idl::convert_impl`'s `NamedSchemas` rejection) can produce rich
/// `ParseDiagnostic` errors with source spans.
struct CompileOutput {
    idl_file: IdlFile,
    registry: SchemaRegistry,
    warnings: Vec<miette::Report>,
    /// Original source text, retained for error diagnostics in type-specific
    /// serialization logic.
    source: String,
    /// Name used for the source in diagnostics (e.g., file path or `"<input>"`).
    source_name: String,
}

impl IdlCompiler {
    fn new() -> Self {
        IdlCompiler {
            import_dirs: Vec::new(),
            accumulated_warnings: Vec::new(),
        }
    }

    fn import_dir(&mut self, dir: PathBuf) {
        self.import_dirs.push(dir);
    }

    fn drain_warnings(&mut self) -> Vec<miette::Report> {
        std::mem::take(&mut self.accumulated_warnings)
    }

    /// Read a `.avdl` file and resolve its path components, then compile it.
    ///
    /// This is the shared implementation behind `Idl::convert(path)` and
    /// `Idl2Schemata::extract(path)`. It reads the file, determines the parent
    /// directory and canonical path, then delegates to [`compile`](Self::compile).
    fn compile_file(&mut self, path: &Path) -> miette::Result<CompileOutput> {
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

        self.compile(&source, &source_name, &dir, canonical_path)
    }

    /// Compile an IDL source string using the current working directory as the
    /// import base. This is the shared implementation behind
    /// `Idl::convert_str_named` and `Idl2Schemata::extract_str_named`.
    fn compile_str(&mut self, source: &str, name: &str) -> miette::Result<CompileOutput> {
        let cwd = std::env::current_dir()
            .map_err(|e| miette::miette!("{e}"))
            .context("determine current directory")?;
        self.compile(source, name, &cwd, None)
    }

    /// Core compilation preamble shared by both `Idl` and `Idl2Schemata`.
    ///
    /// Clears accumulated warnings, creates a fresh `CompileContext`, runs
    /// `parse_and_resolve`, and validates all type references. On success,
    /// returns the parsed IDL file, schema registry, and non-fatal warnings.
    /// On failure, stores warnings in `self.accumulated_warnings` so they
    /// are available via [`drain_warnings`](Self::drain_warnings).
    fn compile(
        &mut self,
        source: &str,
        source_name: &str,
        input_dir: &Path,
        input_path: Option<PathBuf>,
    ) -> miette::Result<CompileOutput> {
        self.accumulated_warnings.clear();

        let mut ctx = CompileContext::new(&self.import_dirs);

        let (idl_file, registry) =
            match parse_and_resolve(source, source_name, input_dir, input_path, &mut ctx) {
                Ok((idl_file, registry)) => (idl_file, registry),
                Err(e) => {
                    self.accumulated_warnings = std::mem::take(&mut ctx.warnings);
                    return Err(e);
                }
            };

        // Validate that all type references resolved. Unresolved references
        // indicate missing imports, undefined types, or cross-namespace
        // references that need fully-qualified names.
        if let Err(e) = validate_all_references(
            &idl_file,
            &registry,
            source,
            source_name,
            &ctx.json_import_spans,
        ) {
            self.accumulated_warnings = std::mem::take(&mut ctx.warnings);
            return Err(e);
        }

        let warnings = std::mem::take(&mut ctx.warnings);
        Ok(CompileOutput {
            idl_file,
            registry,
            warnings,
            source: source.to_string(),
            source_name: source_name.to_string(),
        })
    }
}

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
    inner: IdlCompiler,
}

/// Result of compiling an Avro IDL source.
///
/// The shape of [`json`](IdlOutput::json) depends on the IDL input:
/// - **Protocol** (`protocol Foo { ... }`) — a JSON object matching the `.avpr`
///   format, with `"protocol"`, `"types"`, and `"messages"` keys.
/// - **Standalone schema** (`schema int;`) — a single `.avsc` JSON value
///   (string, object, or array).
///
/// Files with only bare named type declarations (no `schema` keyword, no
/// `protocol`) are rejected, matching Java's `IdlTool` behavior. Use
/// [`Idl2Schemata`] to extract schemas from such files.
pub struct IdlOutput {
    /// The compiled JSON (`.avpr` object or `.avsc` value).
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
            .field(
                "warnings",
                &format_args!("[{} warnings]", self.warnings.len()),
            )
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
            inner: IdlCompiler::new(),
        }
    }

    /// Add an import search directory. Searched in order added, after the input
    /// file's parent directory.
    pub fn import_dir(&mut self, dir: impl Into<PathBuf>) -> &mut Self {
        self.inner.import_dir(dir.into());
        self
    }

    /// Drain warnings accumulated during the most recent `convert*` call.
    ///
    /// When `convert` or `convert_str_named` returns `Ok`, the warnings are
    /// also available in [`IdlOutput::warnings`]. When they return `Err`,
    /// this method is the only way to retrieve warnings that were collected
    /// before the error occurred (e.g., orphaned doc-comment warnings from
    /// parsing that precede a later type-resolution failure).
    ///
    /// Each call drains the internal buffer, so a second call returns an
    /// empty `Vec`.
    pub fn drain_warnings(&mut self) -> Vec<miette::Report> {
        self.inner.drain_warnings()
    }

    /// Compile a `.avdl` file to JSON.
    pub fn convert(&mut self, path: impl AsRef<Path>) -> miette::Result<IdlOutput> {
        let compiled = self.inner.compile_file(path.as_ref())?;
        self.convert_impl(compiled)
    }

    /// Compile an IDL source string to JSON. Uses `"<input>"` as the source
    /// name in diagnostics.
    pub fn convert_str(&mut self, source: &str) -> miette::Result<IdlOutput> {
        self.convert_str_named(source, "<input>")
    }

    /// Compile an IDL source string to JSON with a custom source name for
    /// diagnostics.
    pub fn convert_str_named(&mut self, source: &str, name: &str) -> miette::Result<IdlOutput> {
        let compiled = self.inner.compile_str(source, name)?;
        self.convert_impl(compiled)
    }

    /// Type-specific serialization: serialize the parsed IDL to a single JSON
    /// value (protocol or schema).
    ///
    /// This is the only logic that differs from `Idl2Schemata`. It rejects
    /// `NamedSchemas` (bare declarations without a `schema` keyword or
    /// `protocol`), matching Java's `IdlTool` behavior.
    fn convert_impl(&mut self, compiled: CompileOutput) -> miette::Result<IdlOutput> {
        let CompileOutput {
            idl_file,
            registry,
            warnings,
            source,
            source_name,
        } = compiled;

        // The `idl` subcommand requires either a protocol or a `schema` keyword.
        // Schema-mode files with only bare named type declarations (records, enums,
        // fixed) but no `schema` keyword are rejected — Java's `IdlTool.run()`
        // checks `if (m == null && p == null)` and errors with "the IDL file does
        // not contain a schema nor a protocol." The `idl2schemata` path
        // intentionally omits this check so it can extract named schemas.
        if let IdlFile::NamedSchemas(_) = &idl_file {
            // Stash warnings before returning the error so they're available
            // via `drain_warnings()`.
            self.inner.accumulated_warnings = warnings;

            let span_len = source.len().min(1);
            return Err(ParseDiagnostic {
                src: miette::NamedSource::new(source_name, source),
                span: (0, span_len).into(),
                message: "IDL file contains neither a protocol nor a schema declaration"
                    .to_string(),
                label: Some("this file".to_string()),
                help: Some(
                    "wrap declarations in `protocol MyProto { ... }` or prefix with `schema <type>;`"
                        .to_string(),
                ),
                related: Vec::new(),
            }
            .into());
        }

        // Serialize the parsed IDL to JSON. Protocols become .avpr, standalone
        // schemas become .avsc.
        let json = match &idl_file {
            IdlFile::Protocol(protocol) => protocol_to_json(protocol),
            IdlFile::Schema(schema) => {
                let registry_schemas: Vec<_> = registry.schemas().cloned().collect();
                let lookup = build_lookup(&registry_schemas, None);
                schema_to_json(schema, &mut HashSet::new(), None, &lookup)
            }
            // `NamedSchemas` is rejected above — this arm is unreachable.
            IdlFile::NamedSchemas(_) => unreachable!("NamedSchemas rejected earlier"),
        };

        Ok(IdlOutput { json, warnings })
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
            .field(
                "warnings",
                &format_args!("[{} warnings]", self.warnings.len()),
            )
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
    inner: IdlCompiler,
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
            inner: IdlCompiler::new(),
        }
    }

    /// Add an import search directory.
    pub fn import_dir(&mut self, dir: impl Into<PathBuf>) -> &mut Self {
        self.inner.import_dir(dir.into());
        self
    }

    /// Drain warnings accumulated during the most recent `extract*` call.
    ///
    /// When `extract` or `extract_str_named` returns `Ok`, the warnings are
    /// also available in [`SchemataOutput::warnings`]. When they return `Err`,
    /// this method is the only way to retrieve warnings that were collected
    /// before the error occurred (e.g., orphaned doc-comment warnings from
    /// parsing that precede a later type-resolution failure).
    ///
    /// Each call drains the internal buffer, so a second call returns an
    /// empty `Vec`.
    pub fn drain_warnings(&mut self) -> Vec<miette::Report> {
        self.inner.drain_warnings()
    }

    /// Extract named schemas from a `.avdl` file or a directory of `.avdl`
    /// files. When given a directory, recursively walks it for `.avdl` files
    /// (using [`walkdir`]).
    pub fn extract(&mut self, path: impl AsRef<Path>) -> miette::Result<SchemataOutput> {
        let path = path.as_ref();

        if path.is_dir() {
            return self.extract_directory(path);
        }

        let compiled = self.inner.compile_file(path)?;
        Self::extract_impl(compiled)
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
        let compiled = self.inner.compile_str(source, name)?;
        Self::extract_impl(compiled)
    }

    /// Recursively walk a directory for `.avdl` files and extract schemas from
    /// each. Each file is processed independently with its own registry.
    /// Results are concatenated.
    fn extract_directory(&mut self, dir: &Path) -> miette::Result<SchemataOutput> {
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
            let compiled = self.inner.compile_file(avdl_path)?;
            let output = Self::extract_impl(compiled)?;
            all_schemas.extend(output.schemas);
            all_warnings.extend(output.warnings);
        }

        Ok(SchemataOutput {
            schemas: all_schemas,
            warnings: all_warnings,
        })
    }

    /// Type-specific serialization: serialize each named schema independently
    /// as a self-contained `.avsc` JSON value.
    ///
    /// This is the only logic that differs from `Idl`. Unlike `Idl::convert_impl`,
    /// this accepts `NamedSchemas` (bare declarations without `schema` keyword or
    /// `protocol`), matching Java's `IdlToSchemataTool` behavior.
    fn extract_impl(compiled: CompileOutput) -> miette::Result<SchemataOutput> {
        let CompileOutput {
            registry, warnings, ..
        } = compiled;

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

        Ok(SchemataOutput { schemas, warnings })
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
    /// Maps JSON-imported file display names to their import statement spans
    /// in the IDL source. Used to enrich error messages for unresolved
    /// references from `.avsc`/`.avpr` imports, which lack source spans of
    /// their own.
    json_import_spans: Vec<(String, Option<miette::SourceSpan>)>,
}

impl CompileContext {
    fn new(import_dirs: &[PathBuf]) -> Self {
        CompileContext {
            registry: SchemaRegistry::new(),
            import_ctx: ImportContext::new(import_dirs.to_vec()),
            messages: HashMap::new(),
            warnings: Vec::new(),
            json_import_spans: Vec::new(),
        }
    }
}

/// Parse IDL source and recursively resolve all imports.
///
/// Returns the parsed IDL file and schema registry. Warnings are accumulated
/// in `ctx.warnings` rather than returned directly, so the caller can always
/// access them — even when this function returns `Err`. This design ensures
/// that orphaned doc-comment warnings from parsing are preserved when a
/// later compilation step (import resolution, type registration) fails.
///
/// The key insight for correct type ordering: `parse_idl_named` returns
/// declaration items (imports and local types) in source order, and we
/// process them sequentially, so the registry reflects declaration order.
fn parse_and_resolve(
    source: &str,
    source_name: &str,
    input_dir: &Path,
    input_path: Option<PathBuf>,
    ctx: &mut CompileContext,
) -> miette::Result<(IdlFile, SchemaRegistry)> {
    let (idl_file, decl_items, local_warnings) =
        parse_idl_named(source, source_name).context("parse IDL source")?;

    // Immediately convert local warnings into `miette::Report`s and store
    // them in `ctx.warnings`. This must happen before any fallible operation
    // so that warnings survive even if a later step returns `Err`.
    let local_reports: Vec<miette::Report> = local_warnings
        .into_iter()
        .map(miette::Report::new)
        .collect();
    ctx.warnings.extend(local_reports);

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
    // encountered, register local types when encountered. Any import-derived
    // warnings are appended to `ctx.warnings` by `process_decl_items`.
    process_decl_items(
        &decl_items,
        &mut ctx.registry,
        &mut ctx.import_ctx,
        input_dir,
        &mut ctx.messages,
        &mut ctx.warnings,
        &mut ctx.json_import_spans,
        source,
        source_name,
    )?;

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

    Ok((idl_file, registry))
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
    json_import_spans: &mut Vec<(String, Option<miette::SourceSpan>)>,
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
                    json_import_spans,
                    source,
                    source_name,
                )?;
            }
            DeclItem::Type(schema, span, field_spans) => {
                if let Err(msg) = registry.register(schema.clone()) {
                    if let Some(span) = span {
                        return Err(ParseDiagnostic {
                            src: miette::NamedSource::new(source_name, source.to_string()),
                            span: *span,
                            message: msg,
                            label: None,
                            help: None,
                            related: Vec::new(),
                        }
                        .into());
                    }
                    return Err(miette::miette!("{msg}"));
                }

                // Validate field defaults for Reference-typed fields now that
                // the registry contains all previously-registered types.
                // All validation errors are reported at once so users can fix
                // multiple bad defaults in one edit cycle.
                let errors = validate_record_field_defaults(schema, |full_name| {
                    registry.lookup(full_name).cloned()
                });
                if errors.is_empty() {
                    continue;
                }
                let type_name = schema.full_name().unwrap_or(Cow::Borrowed("<unknown>"));
                let mut error_iter = errors.into_iter();
                let (first_field, first_reason) = error_iter.next().expect("errors is non-empty");

                // Build related diagnostics from subsequent errors.
                let related: Vec<ParseDiagnostic> = error_iter
                    .filter_map(|(field_name, reason)| {
                        let msg = format!(
                            "Invalid default for field `{field_name}` in `{type_name}`: {reason}"
                        );
                        let effective_span = field_spans.get(&field_name).copied().or(*span);
                        effective_span.map(|span| ParseDiagnostic {
                            src: miette::NamedSource::new(source_name, source.to_string()),
                            span,
                            message: msg,
                            label: None,
                            help: None,
                            related: Vec::new(),
                        })
                    })
                    .collect();

                let first_msg = format!(
                    "Invalid default for field `{first_field}` in `{type_name}`: {first_reason}"
                );
                // Prefer the per-field span (from the variable declaration)
                // over the type-level span (from the record keyword), so the
                // diagnostic highlights the offending field, not the record.
                let effective_span = field_spans.get(&first_field).copied().or(*span);
                if let Some(span) = effective_span {
                    return Err(ParseDiagnostic {
                        src: miette::NamedSource::new(source_name, source.to_string()),
                        span,
                        message: first_msg,
                        label: None,
                        help: None,
                        related,
                    }
                    .into());
                }
                return Err(miette::miette!("{first_msg}"));
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
    json_import_spans: &mut Vec<(String, Option<miette::SourceSpan>)>,
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
                    label: None,
                    help: None,
                    related: Vec::new(),
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

            // Track the import so unresolved references from this .avpr can
            // be attributed to the import statement in error diagnostics.
            json_import_spans.push((resolved_path.display().to_string(), import.span));
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

            // Track the import so unresolved references from this .avsc can
            // be attributed to the import statement in error diagnostics.
            json_import_spans.push((resolved_path.display().to_string(), import.span));
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
                warnings.push(miette::Report::new(w).wrap_err(import_file_name.to_string()));
            }

            // If the imported IDL is a protocol, merge its messages.
            if let IdlFile::Protocol(imported_protocol) = &imported_idl {
                messages.extend(imported_protocol.messages.clone());
            }

            // Recursively process declaration items from the imported file.
            // IDL imports use their own source text for span tracking, so
            // json_import_spans is passed through to capture any nested
            // JSON imports within the imported IDL file.
            process_decl_items(
                &nested_decl_items,
                registry,
                import_ctx,
                &import_dir,
                messages,
                warnings,
                json_import_spans,
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
/// error places the `ParseDiagnostic` (which carries `source_code()` and
/// `labels()`) as the **root** diagnostic, and attaches the downstream error
/// as context. This ordering is important because miette's
/// `GraphicalReportHandler` only renders source spans from the root
/// diagnostic -- context layers are shown as plain text.
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
            label: None,
            help: None,
            related: Vec::new(),
        };
        // Place ParseDiagnostic as root so its source span is rendered,
        // and attach the downstream error (e.g., JSON parse failure) as
        // context text above.
        miette::Report::new(diag).wrap_err(format!("{error}"))
    } else {
        error.context(format!("import {} {}", kind, resolved_path.display()))
    }
}

// ==============================================================================
// "Did you mean?" Suggestions for Undefined Type Names
// ==============================================================================
//
// When a type name is misspelled, the error message can suggest similar names
// that exist in the registry or among Avro primitives. We use Levenshtein edit
// distance to find close matches.

use crate::model::schema::PRIMITIVE_TYPE_NAMES;

/// Compute the Levenshtein edit distance between two strings.
///
/// Uses the standard dynamic programming algorithm with a single-row buffer
/// (O(min(m, n)) space). This is sufficient for type names, which are short.
fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    // `row[j]` holds the edit distance between `a[..i]` and `b[..j]`.
    let mut row: Vec<usize> = (0..=n).collect();

    for i in 1..=m {
        let mut prev = row[0];
        row[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            let val = (row[j] + 1) // deletion
                .min(row[j - 1] + 1) // insertion
                .min(prev + cost); // substitution
            prev = row[j];
            row[j] = val;
        }
    }

    row[n]
}

/// Maximum edit distance for a suggestion to be considered "close enough."
///
/// For short names (length <= 4), we require distance <= 1 to avoid noisy
/// suggestions. For longer names, we allow distance <= 2.
fn max_distance(name_len: usize) -> usize {
    if name_len <= 4 {
        1
    } else {
        2
    }
}

/// Build a "did you mean?" help string for an unresolved type name.
///
/// Checks the unresolved name against:
/// 1. Avro primitive type names (`string`, `int`, `boolean`, etc.)
/// 2. Registered type names in the schema registry (both full names and
///    simple/unqualified names)
///
/// When the unresolved name differs from a primitive only in casing (e.g.,
/// `String` vs `string`), the hint includes a note that Avro primitives are
/// lowercase.
///
/// Returns `None` when no sufficiently close match is found.
fn suggest_similar_name(unresolved: &str, registry: &SchemaRegistry) -> Option<String> {
    // The unresolved name may be fully qualified (e.g., "test.stiring"). We
    // compare the unqualified (simple) part against primitives and the simple
    // parts of registered names, because typos almost always affect the simple
    // name, not the namespace.
    let simple = unresolved
        .rsplit('.')
        .next()
        .expect("rsplit always yields at least one element");

    let mut best: Option<(String, usize, bool)> = None; // (suggestion, distance, is_primitive)

    // Check against Avro primitive type names.
    for &prim in PRIMITIVE_TYPE_NAMES {
        let dist = levenshtein(simple, prim);
        let threshold = max_distance(simple.len().min(prim.len()));
        if dist <= threshold {
            if best.as_ref().map_or(true, |(_, d, _)| dist < *d) {
                best = Some((prim.to_string(), dist, true));
            }
        }
    }

    // Check against registered type names. We compare both the full name
    // and the simple (unqualified) name to handle cases where the user
    // omitted the namespace or misspelled just the type part.
    for registered_full in registry.names() {
        // Compare unresolved full name against registered full name.
        let dist_full = levenshtein(unresolved, registered_full);
        let threshold_full = max_distance(unresolved.len().min(registered_full.len()));
        if dist_full <= threshold_full {
            if best.as_ref().map_or(true, |(_, d, _)| dist_full < *d) {
                best = Some((registered_full.to_string(), dist_full, false));
            }
        }

        // Also compare the simple parts, in case the namespace is correct
        // but the type name has a typo.
        let registered_simple = registered_full
            .rsplit('.')
            .next()
            .expect("rsplit always yields at least one element");
        let dist_simple = levenshtein(simple, registered_simple);
        let threshold_simple = max_distance(simple.len().min(registered_simple.len()));
        if dist_simple <= threshold_simple {
            // Suggest the full registered name so the user gets the right
            // fully-qualified form.
            if best.as_ref().map_or(true, |(_, d, _)| dist_simple < *d) {
                best = Some((registered_full.to_string(), dist_simple, false));
            }
        }
    }

    best.map(|(suggestion, _, is_primitive)| {
        let case_mismatch = is_primitive && simple.eq_ignore_ascii_case(&suggestion);
        if case_mismatch {
            format!(
                "did you mean `{suggestion}`? (note: Avro primitives are lowercase)"
            )
        } else {
            format!("did you mean `{suggestion}`?")
        }
    })
}

/// Validate that all type references in the IDL file and registry resolved.
///
/// Unresolved references indicate missing imports, undefined types, or
/// cross-namespace references that need fully-qualified names. Java's
/// `IdlReader` treats these as fatal errors.
///
/// When a reference carries a source span (from the parser), the error is
/// reported as a `ParseDiagnostic` with source highlighting. References
/// without spans (from JSON imports) are reported using the import
/// statement's span and a help message naming the imported file, so the
/// user can identify which import brought in the undefined type.
///
/// When an unresolved name is similar to a primitive or registered type,
/// the error includes a "did you mean?" suggestion.
fn validate_all_references(
    idl_file: &IdlFile,
    registry: &SchemaRegistry,
    source: &str,
    source_name: &str,
    json_import_spans: &[(String, Option<miette::SourceSpan>)],
) -> miette::Result<()> {
    let mut unresolved = registry.validate_references();

    // `Schema` and `NamedSchemas` store their top-level schemas outside
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
        IdlFile::Protocol(protocol) => {
            // Message return types, parameter types, and error types are stored
            // in the `Protocol` but never registered in the `SchemaRegistry`, so
            // `validate_references()` alone does not see them. We must validate
            // them explicitly here. Without this, undefined types in messages
            // silently pass through (Java rejects them with "Undefined schema").
            for msg in protocol.messages.values() {
                unresolved.extend(registry.validate_schema(&msg.response));
                for field in &msg.request {
                    unresolved.extend(registry.validate_schema(&field.schema));
                }
                if let Some(errors) = &msg.errors {
                    for err_schema in errors {
                        unresolved.extend(registry.validate_schema(err_schema));
                    }
                }
            }
        }
    }

    // Deduplicate by name while preserving source order (first occurrence
    // wins). We use a `HashSet` to track which names we've already seen,
    // retaining the entry whose span appears earliest in the file.
    {
        let mut seen = HashSet::new();
        unresolved.retain(|(name, _)| seen.insert(name.clone()));
    }

    // Sort by source span offset so the first error in the file is reported
    // first. References without a span (from JSON imports) sort to the end.
    unresolved.sort_by_key(|(_, span)| span.map_or(usize::MAX, |s| s.offset()));

    if unresolved.is_empty() {
        return Ok(());
    }

    // Partition into those with source spans (can produce rich diagnostics)
    // and those without (from JSON imports, fall back to plain text).
    let (with_span, without_span): (Vec<_>, Vec<_>) =
        unresolved.into_iter().partition(|(_, s)| s.is_some());

    // Build a help message listing the JSON-imported files that may contain
    // the undefined type, for use in spanless reference diagnostics.
    let import_file_names: Vec<&str> = json_import_spans
        .iter()
        .map(|(path, _)| path.as_str())
        .collect();

    if with_span.is_empty() {
        // All unresolved references come from JSON imports (no IDL source
        // spans). Use the first available import statement span to point
        // the user at the import line, with a help message naming the
        // imported file(s).
        let first_import_span = json_import_spans.iter().find_map(|(_, s)| *s);

        let names: Vec<&str> = without_span.iter().map(|(name, _)| name.as_str()).collect();
        let message = format!("Undefined name: {}", names.join(", "));

        let help = if import_file_names.is_empty() {
            None
        } else {
            Some(format!(
                "the undefined type(s) may be referenced in imported file(s): {}",
                import_file_names.join(", ")
            ))
        };

        if let Some(span) = first_import_span {
            return Err(ParseDiagnostic {
                src: miette::NamedSource::new(source_name, source.to_string()),
                span,
                message,
                label: Some("this import contains undefined type references".to_string()),
                help,
                related: Vec::new(),
            }
            .into());
        }

        // No import span available either (e.g., import from string input
        // without span tracking). Fall back to plain message with help.
        if let Some(help) = help {
            miette::bail!("{message}\n  help: {help}");
        }
        miette::bail!("{message}");
    }

    // The first spanned reference becomes the primary diagnostic; the rest
    // are attached as related diagnostics so users see all undefined names
    // in one error report.
    let mut span_iter = with_span.into_iter();
    let (first_name, first_span) = span_iter.next().expect("with_span is non-empty");
    let first_span = first_span.expect("partitioned into Some");

    let mut related: Vec<ParseDiagnostic> = span_iter
        .map(|(name, span)| {
            let span = span.expect("partitioned into Some");
            let help = suggest_similar_name(&name, registry);
            ParseDiagnostic {
                src: miette::NamedSource::new(source_name, source.to_string()),
                span,
                message: format!("Undefined name: {name}"),
                label: None,
                help,
                related: Vec::new(),
            }
        })
        .collect();

    // Append spanless references as related diagnostics, using the import
    // statement spans so the user can see which import brought them in.
    // Fall back to a zero-length span at offset 0 if no import span is
    // available. Include "did you mean?" suggestions where applicable.
    let fallback_span: miette::SourceSpan = (0, 0).into();
    for (name, _) in &without_span {
        let (span, label) =
            if let Some((path, Some(import_span))) = json_import_spans.first() {
                (
                    *import_span,
                    Some(format!("type `{name}` referenced in imported file `{path}`")),
                )
            } else {
                (fallback_span, None)
            };

        let help = if import_file_names.is_empty() {
            suggest_similar_name(name, registry)
        } else {
            Some(format!(
                "the undefined type may be referenced in imported file(s): {}",
                import_file_names.join(", ")
            ))
        };

        related.push(ParseDiagnostic {
            src: miette::NamedSource::new(source_name, source.to_string()),
            span,
            message: format!("Undefined name: {name}"),
            label,
            help,
            related: Vec::new(),
        });
    }

    let first_help = suggest_similar_name(&first_name, registry);
    Err(ParseDiagnostic {
        src: miette::NamedSource::new(source_name, source.to_string()),
        span: first_span,
        message: format!("Undefined name: {first_name}"),
        label: None,
        help: first_help,
        related,
    }
    .into())
}

// ==============================================================================
// Unit Tests
// ==============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::schema::AvroSchema;
    use pretty_assertions::assert_eq;

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

    // =========================================================================
    // Undefined types in protocol messages
    // =========================================================================
    //
    // Java's IdlReader rejects undefined types in message return types,
    // parameter types, and throws clauses with "Undefined schema" errors.
    // We verify that our validation catches these cases too.

    #[test]
    fn undefined_message_return_type_is_rejected() {
        let result = Idl::new().convert_str(
            r#"
            @namespace("test")
            protocol P {
                DoesNotExist getUnknown();
            }
            "#,
        );
        let err = result.expect_err("undefined return type should be rejected");
        let msg = format!("{err}");
        assert!(
            msg.contains("Undefined name"),
            "should report undefined name for return type, got: {msg}"
        );
    }

    #[test]
    fn undefined_message_param_type_is_rejected() {
        let result = Idl::new().convert_str(
            r#"
            @namespace("test")
            protocol P {
                void process(DoesNotExist arg);
            }
            "#,
        );
        let err = result.expect_err("undefined param type should be rejected");
        let msg = format!("{err}");
        assert!(
            msg.contains("Undefined name"),
            "should report undefined name for param type, got: {msg}"
        );
    }

    #[test]
    fn undefined_message_error_type_is_rejected() {
        let result = Idl::new().convert_str(
            r#"
            @namespace("test")
            protocol P {
                void doThing() throws DoesNotExist;
            }
            "#,
        );
        let err = result.expect_err("undefined error type should be rejected");
        let msg = format!("{err}");
        assert!(
            msg.contains("Undefined name"),
            "should report undefined name for error type, got: {msg}"
        );
    }

    #[test]
    fn defined_message_types_are_accepted() {
        // Verify that messages referencing defined types still work correctly.
        let output = Idl::new()
            .convert_str(
                r#"
                @namespace("test")
                protocol P {
                    record Request { string query; }
                    record Response { string answer; }
                    error ServiceError { string message; }
                    Response search(Request req) throws ServiceError;
                }
                "#,
            )
            .expect("messages with defined types should be accepted");
        assert_eq!(output.json["protocol"], "P");
        assert!(output.json["messages"]["search"].is_object());
    }

    #[test]
    fn extract_str_undefined_message_return_type_is_rejected() {
        // idl2schemata should also reject undefined message types.
        let result = Idl2Schemata::new().extract_str(
            r#"
            @namespace("test")
            protocol P {
                DoesNotExist getUnknown();
            }
            "#,
        );
        let err = result.expect_err("idl2schemata should reject undefined return type");
        let msg = format!("{err}");
        assert!(
            msg.contains("Undefined name"),
            "should report undefined name for return type, got: {msg}"
        );
    }

    // =========================================================================
    // Record default validation: partial defaults with missing required fields
    // =========================================================================
    //
    // Java rejects record defaults that omit required fields (fields without
    // their own defaults). Our Rust implementation must also reject these.

    #[test]
    fn record_default_partial_missing_required_field_rejected() {
        let result = Idl::new().convert_str(
            r#"
            @namespace("test")
            protocol P {
                record Inner {
                    string name;
                    int value;  // required - no default
                }
                record Outer { Inner inner = {"name": "partial"}; }
            }
            "#,
        );
        let err = result.expect_err("partial record default should be rejected");
        let msg = format!("{err}");
        assert!(
            msg.contains("missing required field"),
            "should report missing field, got: {msg}"
        );
        assert!(
            msg.contains("value"),
            "should mention the missing field name, got: {msg}"
        );
    }

    #[test]
    fn record_default_complete_with_all_fields_accepted() {
        let output = Idl::new()
            .convert_str(
                r#"
            @namespace("test")
            protocol P {
                record Inner {
                    string name;
                    int value;
                }
                record Outer { Inner inner = {"name": "test", "value": 42}; }
            }
            "#,
            )
            .expect("complete record default should be accepted");
        assert_eq!(output.json["protocol"], "P");
    }

    #[test]
    fn record_default_partial_with_field_default_allowed() {
        // Fields with defaults in the schema can be omitted.
        let output = Idl::new()
            .convert_str(
                r#"
            @namespace("test")
            protocol P {
                record Inner {
                    string name;
                    int value = 0;  // has default
                }
                record Outer { Inner inner = {"name": "test"}; }
            }
            "#,
            )
            .expect("record default omitting field with default should be accepted");
        assert_eq!(output.json["protocol"], "P");
    }

    #[test]
    fn record_default_nested_validates_inner() {
        let output = Idl::new()
            .convert_str(
                r#"
            @namespace("test")
            protocol P {
                record Inner { int x; }
                record Middle { Inner inner; }
                record Outer { Middle m = {"inner": {"x": 1}}; }
            }
            "#,
            )
            .expect("nested complete record defaults should be accepted");
        assert_eq!(output.json["protocol"], "P");
    }

    #[test]
    fn record_default_nested_incomplete_rejected() {
        let result = Idl::new().convert_str(
            r#"
            @namespace("test")
            protocol P {
                record Inner { int x; }
                record Middle { Inner inner; }
                record Outer { Middle m = {"inner": {}}; }
            }
            "#,
        );
        let err = result.expect_err("incomplete nested record default should fail");
        let msg = format!("{err}");
        assert!(
            msg.contains("missing required field"),
            "should report missing field, got: {msg}"
        );
    }

    #[test]
    fn record_default_wrong_field_type_rejected() {
        let result = Idl::new().convert_str(
            r#"
            @namespace("test")
            protocol P {
                record Inner { int count; }
                record Outer { Inner inner = {"count": "not_an_int"}; }
            }
            "#,
        );
        let err = result.expect_err("record default with wrong field type should fail");
        let msg = format!("{err}");
        // The error should mention something about the invalid value.
        assert!(
            msg.contains("count") || msg.contains("int"),
            "should mention the field or expected type, got: {msg}"
        );
    }

    // =========================================================================
    // Import-only schema-mode files
    // =========================================================================
    //
    // Schema-mode files with only `namespace` and `import` statements (no local
    // type declarations, no `schema` keyword, no protocol) should be accepted
    // by `idl2schemata` (which extracts imported named schemas) but rejected by
    // `idl` (which requires a protocol or schema declaration to produce output).
    // This matches Java's behavior: `IdlToSchemataTool` accepts such files,
    // while `IdlTool` rejects them.

    #[test]
    fn idl_rejects_import_only_schema_mode() {
        let result = Idl::new().convert_str("namespace org.example;");
        let err = result.expect_err("idl should reject import-only schema-mode file");
        let msg = format!("{err}");
        assert!(
            msg.contains("neither a protocol nor a schema declaration"),
            "expected 'neither a protocol nor a schema declaration', got: {msg}"
        );
    }

    #[test]
    fn idl2schemata_accepts_import_only_schema_mode() {
        // A schema-mode file with only a namespace and no declarations should
        // succeed (producing zero schemas) rather than erroring.
        let output = Idl2Schemata::new()
            .extract_str("namespace org.example;")
            .expect("idl2schemata should accept import-only schema-mode file");
        assert!(
            output.schemas.is_empty(),
            "expected no schemas from namespace-only file"
        );
    }

    #[test]
    fn idl2schemata_extracts_schemas_from_import_only_file() {
        // Create a temporary directory with an .avsc file and an .avdl that
        // imports it. The .avdl has no local type declarations.
        let dir = tempfile::tempdir().expect("create temp dir");
        let avsc_path = dir.path().join("Foo.avsc");
        std::fs::write(
            &avsc_path,
            r#"{"type":"record","name":"Foo","namespace":"org.example","fields":[{"name":"x","type":"string"}]}"#,
        )
        .expect("write .avsc");

        let avdl_path = dir.path().join("import-only.avdl");
        std::fs::write(
            &avdl_path,
            "namespace org.example;\nimport schema \"Foo.avsc\";\n",
        )
        .expect("write .avdl");

        let output = Idl2Schemata::new()
            .extract(&avdl_path)
            .expect("idl2schemata should extract imported schemas");
        assert_eq!(output.schemas.len(), 1, "should extract one schema");
        assert_eq!(output.schemas[0].name, "Foo");
        assert_eq!(output.schemas[0].schema["type"], "record");
    }

    #[test]
    fn idl_rejects_import_only_file_even_with_imports() {
        // Even when there are import statements, `idl` should reject a
        // schema-mode file that has no local schema declarations, matching
        // Java's `IdlTool` behavior.
        let dir = tempfile::tempdir().expect("create temp dir");
        let avsc_path = dir.path().join("Bar.avsc");
        std::fs::write(
            &avsc_path,
            r#"{"type":"record","name":"Bar","namespace":"org.example","fields":[{"name":"y","type":"int"}]}"#,
        )
        .expect("write .avsc");

        let avdl_path = dir.path().join("import-only.avdl");
        std::fs::write(
            &avdl_path,
            "namespace org.example;\nimport schema \"Bar.avsc\";\n",
        )
        .expect("write .avdl");

        let result = Idl::new().convert(&avdl_path);
        let err = result.expect_err("idl should reject import-only file");
        let msg = format!("{err}");
        assert!(
            msg.contains("neither a protocol nor a schema declaration"),
            "expected 'neither a protocol nor a schema declaration', got: {msg}"
        );
    }

    // =========================================================================
    // Bare named type declarations (no `schema` keyword, no `protocol`)
    // =========================================================================
    //
    // Java's `IdlTool.run()` rejects files with only named type declarations
    // (records, enums, fixed) but no `schema` keyword or `protocol` — both
    // `m` (main schema) and `p` (protocol) are null. The `idl` subcommand
    // should match this behavior, while `idl2schemata` should accept them.

    #[test]
    fn idl_rejects_bare_named_types() {
        let result = Idl::new().convert_str(
            r#"
            namespace org.test;
            record Foo { string name; }
            enum Color { RED, GREEN, BLUE }
            "#,
        );
        let err = result.expect_err("idl should reject bare named types without schema keyword");
        let msg = format!("{err}");
        assert!(
            msg.contains("neither a protocol nor a schema declaration"),
            "expected 'neither a protocol nor a schema declaration', got: {msg}"
        );
    }

    // =========================================================================
    // Field default validation for Reference-typed fields (issue #0f6b49e3)
    // =========================================================================

    #[test]
    fn field_default_invalid_for_enum_reference() {
        // An enum field with an integer default should be rejected after
        // the reference is resolved.
        let result = Idl::new().convert_str(
            r#"
            protocol P {
                enum Color { RED, GREEN, BLUE }
                record R {
                    Color favorite = 42;
                }
            }
            "#,
        );
        let err = result.unwrap_err();
        insta::assert_snapshot!(crate::error::render_diagnostic(&err));
    }

    #[test]
    fn field_default_multiple_invalid_references() {
        // Two fields with bad defaults exercises the `related` diagnostics
        // loop that builds secondary error messages from additional errors.
        let result = Idl::new().convert_str(
            r#"
            protocol P {
                enum Color { RED, GREEN, BLUE }
                record R {
                    Color first = 1;
                    Color second = 2;
                }
            }
            "#,
        );
        let err = result.unwrap_err();
        insta::assert_snapshot!(crate::error::render_diagnostic(&err));
    }

    #[test]
    fn field_default_valid_for_enum_reference() {
        // A valid string default for an enum reference should be accepted.
        let output = Idl::new()
            .convert_str(
                r#"
                protocol P {
                    enum Color { RED, GREEN, BLUE }
                    record R {
                        Color favorite = "RED";
                    }
                }
                "#,
            )
            .expect("valid enum default should be accepted");
        assert_eq!(output.json["protocol"], "P");
    }

    #[test]
    fn field_default_invalid_for_record_reference() {
        // A record field with a string default should be rejected (records
        // expect object defaults).
        let result = Idl::new().convert_str(
            r#"
            protocol P {
                record Inner { string name; }
                record Outer {
                    Inner nested = "not an object";
                }
            }
            "#,
        );
        let err = result.unwrap_err();
        insta::assert_snapshot!(crate::error::render_diagnostic(&err));
    }

    #[test]
    fn idl2schemata_accepts_bare_named_types() {
        let output = Idl2Schemata::new()
            .extract_str(
                r#"
                namespace org.test;
                record Foo { string name; }
                enum Color { RED, GREEN, BLUE }
                "#,
            )
            .expect("idl2schemata should accept bare named types");
        assert_eq!(output.schemas.len(), 2, "should extract two schemas");
        assert_eq!(output.schemas[0].name, "Foo");
        assert_eq!(output.schemas[1].name, "Color");
    }

    // =========================================================================
    // Levenshtein edit distance
    // =========================================================================

    #[test]
    fn levenshtein_identical_strings() {
        assert_eq!(levenshtein("string", "string"), 0);
    }

    #[test]
    fn levenshtein_empty_strings() {
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", "xyz"), 3);
    }

    #[test]
    fn levenshtein_single_edit() {
        // Substitution.
        assert_eq!(levenshtein("string", "strang"), 1);
        // Insertion.
        assert_eq!(levenshtein("sting", "string"), 1);
        // Deletion.
        assert_eq!(levenshtein("string", "sting"), 1);
    }

    #[test]
    fn levenshtein_two_edits() {
        // "stiring" -> "string" requires only 1 edit (delete the extra 'i'):
        // s-t-i-r-i-n-g -> s-t-r-i-n-g
        assert_eq!(levenshtein("stiring", "string"), 1);
        // "bolean" -> "boolean" requires 1 edit (insert 'o'):
        assert_eq!(levenshtein("bolean", "boolean"), 1);
        // "dubble" -> "double" requires 2 edits:
        assert_eq!(levenshtein("dubble", "double"), 2);
    }

    #[test]
    fn levenshtein_case_difference() {
        assert_eq!(levenshtein("String", "string"), 1);
        assert_eq!(levenshtein("INT", "int"), 3);
    }

    // =========================================================================
    // "Did you mean?" suggestions for undefined type names
    // =========================================================================

    #[test]
    fn suggest_primitive_typo_stiring() {
        let reg = SchemaRegistry::new();
        let suggestion = suggest_similar_name("test.stiring", &reg);
        assert!(
            suggestion.is_some(),
            "should suggest something for 'stiring'"
        );
        let s = suggestion.expect("just checked is_some");
        assert!(
            s.contains("string"),
            "should suggest 'string', got: {s}"
        );
    }

    #[test]
    fn suggest_primitive_case_mismatch() {
        let reg = SchemaRegistry::new();
        let suggestion = suggest_similar_name("String", &reg);
        assert!(
            suggestion.is_some(),
            "should suggest something for 'String'"
        );
        let s = suggestion.expect("just checked is_some");
        assert!(
            s.contains("string"),
            "should suggest 'string', got: {s}"
        );
        assert!(
            s.contains("lowercase"),
            "should mention primitives are lowercase, got: {s}"
        );
    }

    #[test]
    fn suggest_primitive_int_capitalized() {
        let reg = SchemaRegistry::new();
        let suggestion = suggest_similar_name("Int", &reg);
        assert!(
            suggestion.is_some(),
            "should suggest something for 'Int'"
        );
        let s = suggestion.expect("just checked is_some");
        assert!(s.contains("int"), "should suggest 'int', got: {s}");
        assert!(
            s.contains("lowercase"),
            "should mention primitives are lowercase, got: {s}"
        );
    }

    #[test]
    fn suggest_no_match_for_unrelated_name() {
        let reg = SchemaRegistry::new();
        let suggestion = suggest_similar_name("CompletelyUnrelated", &reg);
        assert!(
            suggestion.is_none(),
            "should not suggest anything for a completely unrelated name"
        );
    }

    #[test]
    fn suggest_registered_type_typo() {
        let mut reg = SchemaRegistry::new();
        reg.register(AvroSchema::Record {
            name: "UserProfile".to_string(),
            namespace: Some("com.example".to_string()),
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        })
        .expect("registration succeeds");

        let suggestion = suggest_similar_name("com.example.UserProfle", &reg);
        assert!(
            suggestion.is_some(),
            "should suggest something for 'UserProfle'"
        );
        let s = suggestion.expect("just checked is_some");
        assert!(
            s.contains("com.example.UserProfile"),
            "should suggest the full name, got: {s}"
        );
    }

    #[test]
    fn suggest_registered_type_simple_name_typo() {
        let mut reg = SchemaRegistry::new();
        reg.register(AvroSchema::Record {
            name: "Account".to_string(),
            namespace: Some("org.bank".to_string()),
            doc: None,
            fields: vec![],
            is_error: false,
            aliases: vec![],
            properties: HashMap::new(),
        })
        .expect("registration succeeds");

        // Typo in the simple name part, correct namespace.
        let suggestion = suggest_similar_name("org.bank.Acount", &reg);
        assert!(
            suggestion.is_some(),
            "should suggest something for 'Acount'"
        );
        let s = suggestion.expect("just checked is_some");
        assert!(
            s.contains("org.bank.Account"),
            "should suggest the full registered name, got: {s}"
        );
    }

    // =========================================================================
    // Integration: error messages include suggestions
    // =========================================================================

    #[test]
    fn undefined_type_suggests_primitive() {
        let result = Idl::new().convert_str(
            r#"
            @namespace("test")
            protocol P {
                record R { stiring name; }
            }
            "#,
        );
        let err = result.expect_err("should fail with undefined type");
        let rendered = crate::error::render_diagnostic(&err);
        assert!(
            rendered.contains("did you mean"),
            "error should include 'did you mean', got:\n{rendered}"
        );
        assert!(
            rendered.contains("string"),
            "error should suggest 'string', got:\n{rendered}"
        );
    }

    #[test]
    fn undefined_type_suggests_capitalized_primitive() {
        let result = Idl::new().convert_str(
            r#"
            @namespace("test")
            protocol P {
                record R { String name; }
            }
            "#,
        );
        let err = result.expect_err("should fail with undefined type");
        let rendered = crate::error::render_diagnostic(&err);
        assert!(
            rendered.contains("did you mean"),
            "error should include 'did you mean', got:\n{rendered}"
        );
        assert!(
            rendered.contains("lowercase"),
            "error should mention primitives are lowercase, got:\n{rendered}"
        );
    }

    #[test]
    fn undefined_type_suggests_registered_type() {
        let result = Idl::new().convert_str(
            r#"
            @namespace("test")
            protocol P {
                record UserProfile { string name; }
                record R { UserProfle author; }
            }
            "#,
        );
        let err = result.expect_err("should fail with undefined type");
        let rendered = crate::error::render_diagnostic(&err);
        assert!(
            rendered.contains("did you mean"),
            "error should include 'did you mean', got:\n{rendered}"
        );
        assert!(
            rendered.contains("UserProfile"),
            "error should suggest 'UserProfile', got:\n{rendered}"
        );
    }

    #[test]
    fn undefined_type_no_suggestion_for_unrelated() {
        let result = Idl::new().convert_str(
            r#"
            @namespace("test")
            protocol P {
                record R { CompletelyUnrelated field; }
            }
            "#,
        );
        let err = result.expect_err("should fail with undefined type");
        let rendered = crate::error::render_diagnostic(&err);
        assert!(
            rendered.contains("Undefined name"),
            "error should report undefined name, got:\n{rendered}"
        );
        // Should NOT contain "did you mean" since nothing is close.
        let has_suggestion = rendered.contains("did you mean");
        assert!(
            has_suggestion == false,
            "error should NOT include 'did you mean' for unrelated name, got:\n{rendered}"
        );
    }

    // =========================================================================
    // Imported .avsc with undefined type reference (issue 37840ce8)
    // =========================================================================

    #[test]
    #[cfg_attr(windows, ignore)]
    fn imported_avsc_undefined_type_includes_file_path() {
        let dir = tempfile::tempdir().expect("create temp dir");

        let avsc_path = dir.path().join("bad.avsc");
        std::fs::write(
            &avsc_path,
            r#"{"type":"record","name":"Foo","fields":[{"name":"x","type":"UnknownType"}]}"#,
        )
        .expect("write .avsc");

        let avdl_path = dir.path().join("test.avdl");
        std::fs::write(
            &avdl_path,
            "protocol Test {\n  import schema \"bad.avsc\";\n}\n",
        )
        .expect("write .avdl");

        let result = Idl::new().convert(&avdl_path);
        let err = result.expect_err("should fail with undefined type");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("Undefined name"),
            "should report undefined name, got: {msg}"
        );
        assert!(
            msg.contains("bad.avsc"),
            "error should mention the imported file path, got: {msg}"
        );
    }

    #[test]
    #[cfg_attr(windows, ignore)]
    fn imported_avsc_undefined_type_snapshot() {
        let dir = tempfile::tempdir().expect("create temp dir");

        let avsc_path = dir.path().join("bad-ref.avsc");
        std::fs::write(
            &avsc_path,
            r#"{"type":"record","name":"Foo","fields":[{"name":"x","type":"UnknownType"}]}"#,
        )
        .expect("write .avsc");

        let avdl_path = dir.path().join("test.avdl");
        std::fs::write(
            &avdl_path,
            "protocol Test {\n  import schema \"bad-ref.avsc\";\n}\n",
        )
        .expect("write .avdl");

        let err = Idl::new()
            .convert(&avdl_path)
            .expect_err("should fail with undefined type");
        let canonical_dir = dir
            .path()
            .canonicalize()
            .expect("canonicalize temp dir");
        let handler = miette::GraphicalReportHandler::new_themed(
            miette::GraphicalTheme::none(),
        )
        .with_width(200);
        let mut rendered = String::new();
        handler
            .render_report(&mut rendered, err.as_ref())
            .expect("render to String is infallible");

        let canonical_str = canonical_dir.display().to_string();
        let raw_str = dir.path().display().to_string();
        let stable: String = rendered
            .replace(&canonical_str, "<tmpdir>")
            .replace(&raw_str, "<tmpdir>");
        insta::assert_snapshot!(stable);
    }

    // =========================================================================
    // `Idl2Schemata::drain_warnings` after failed `extract_str` call
    // =========================================================================
    //
    // When `extract_str` returns `Err`, warnings collected before the error
    // (e.g., orphaned doc comments from parsing) are stashed in the builder
    // and can only be retrieved via `drain_warnings()`. This test verifies
    // that path.

    #[test]
    fn idl2schemata_drain_warnings_after_error() {
        let mut builder = Idl2Schemata::new();

        // This IDL has an orphaned doc comment inside a record body (produces
        // a warning) and an undefined type reference in a second record
        // (produces an error). The orphaned doc comment sits after the last
        // field and before the closing brace, so it is not consumed by any
        // declaration.
        let result = builder.extract_str(
            r#"
            @namespace("test")
            protocol P {
                record A {
                    string name;
                    /** orphaned doc comment */
                }
                record B { MissingType field; }
            }
            "#,
        );
        assert!(result.is_err(), "should fail due to undefined type");

        let warnings = builder.drain_warnings();
        assert!(
            !warnings.is_empty(),
            "drain_warnings() should return warnings accumulated before the error"
        );

        // A second drain should return empty (the buffer was consumed).
        let second = builder.drain_warnings();
        assert!(
            second.is_empty(),
            "second drain_warnings() call should return empty Vec"
        );
    }

    #[test]
    #[cfg_attr(windows, ignore)]
    fn imported_avpr_undefined_type_includes_file_path() {
        let dir = tempfile::tempdir().expect("create temp dir");

        let avpr_path = dir.path().join("bad.avpr");
        std::fs::write(
            &avpr_path,
            r#"{"protocol":"BadProto","types":[{"type":"record","name":"Rec","fields":[{"name":"f","type":"MissingRef"}]}],"messages":{}}"#,
        )
        .expect("write .avpr");

        let avdl_path = dir.path().join("test.avdl");
        std::fs::write(
            &avdl_path,
            "protocol Test {\n  import protocol \"bad.avpr\";\n}\n",
        )
        .expect("write .avdl");

        let result = Idl::new().convert(&avdl_path);
        let err = result.expect_err("should fail with undefined type");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("Undefined name"),
            "should report undefined name, got: {msg}"
        );
        assert!(
            msg.contains("bad.avpr"),
            "error should mention the imported file path, got: {msg}"
        );
    }

    // =========================================================================
    // Multiple unresolved references (validate_all_references edge cases)
    // =========================================================================
    //
    // These tests exercise branches in `validate_all_references` that were
    // previously untested:
    //   1. The `span_iter` loop that builds `related` diagnostics from the
    //      2nd, 3rd, ... spanned unresolved references.
    //   2. The spanless-only path when all unresolved references lack source
    //      spans (from JSON imports).
    //   3. The mixed span/spanless path that appends spanless references as
    //      related diagnostics alongside spanned ones.

    #[test]
    fn multiple_undefined_types_reported_together() {
        // Two distinct undefined types in the same protocol exercise the
        // `related` diagnostics loop (lines that build ParseDiagnostic
        // entries for the 2nd, 3rd, ... unresolved spanned references).
        let result = Idl::new().convert_str(
            r#"
            @namespace("test")
            protocol P {
                record R {
                    AlphaType a;
                    BetaType b;
                }
            }
            "#,
        );
        let err = result.expect_err("should fail with two undefined types");
        let rendered = crate::error::render_diagnostic(&err);
        assert!(
            rendered.contains("AlphaType"),
            "error should mention first undefined type, got:\n{rendered}"
        );
        assert!(
            rendered.contains("BetaType"),
            "error should mention second undefined type as related, got:\n{rendered}"
        );
        insta::assert_snapshot!(rendered);
    }

    #[test]
    #[cfg_attr(windows, ignore)]
    fn spanless_only_unresolved_references() {
        // When all unresolved references come from JSON imports (no IDL
        // source spans), the code falls back to a `ParseDiagnostic` using
        // the import statement's span, or to a plain `miette::bail!` if
        // no import span is available.
        //
        // This test imports a .avsc that references an undefined type, but
        // the IDL itself has no local undefined references. This exercises
        // the `with_span.is_empty()` branch.
        let dir = tempfile::tempdir().expect("create temp dir");

        let avsc_path = dir.path().join("spanless.avsc");
        std::fs::write(
            &avsc_path,
            r#"{"type":"record","name":"Rec","fields":[{"name":"f","type":"NoSuchType"}]}"#,
        )
        .expect("write .avsc");

        let avdl_path = dir.path().join("test.avdl");
        std::fs::write(
            &avdl_path,
            "protocol Test {\n  import schema \"spanless.avsc\";\n}\n",
        )
        .expect("write .avdl");

        let err = Idl::new()
            .convert(&avdl_path)
            .expect_err("should fail with undefined type from import");
        let canonical_dir = dir
            .path()
            .canonicalize()
            .expect("canonicalize temp dir");
        let handler = miette::GraphicalReportHandler::new_themed(
            miette::GraphicalTheme::none(),
        )
        .with_width(200);
        let mut rendered = String::new();
        handler
            .render_report(&mut rendered, err.as_ref())
            .expect("render to String is infallible");

        let canonical_str = canonical_dir.display().to_string();
        let raw_str = dir.path().display().to_string();
        let stable: String = rendered
            .replace(&canonical_str, "<tmpdir>")
            .replace(&raw_str, "<tmpdir>");

        assert!(
            stable.contains("Undefined name"),
            "should report undefined name, got:\n{stable}"
        );
        assert!(
            stable.contains("spanless.avsc"),
            "should mention the imported file, got:\n{stable}"
        );
        insta::assert_snapshot!(stable);
    }

    #[test]
    #[cfg_attr(windows, ignore)]
    fn mixed_span_and_spanless_unresolved_references() {
        // When there are both spanned (from IDL source) and spanless (from
        // JSON imports) unresolved references, the spanless references
        // should appear as related diagnostics appended after the spanned
        // ones. This exercises the `for (name, _) in &without_span` loop.
        let dir = tempfile::tempdir().expect("create temp dir");

        let avsc_path = dir.path().join("mixed.avsc");
        std::fs::write(
            &avsc_path,
            r#"{"type":"record","name":"Imported","fields":[{"name":"r","type":"FromJsonOnly"}]}"#,
        )
        .expect("write .avsc");

        let avdl_path = dir.path().join("test.avdl");
        std::fs::write(
            &avdl_path,
            r#"protocol Test {
  import schema "mixed.avsc";
  record Local { FromIdlOnly x; }
}
"#,
        )
        .expect("write .avdl");

        let err = Idl::new()
            .convert(&avdl_path)
            .expect_err("should fail with both spanned and spanless undefined types");
        let canonical_dir = dir
            .path()
            .canonicalize()
            .expect("canonicalize temp dir");
        let handler = miette::GraphicalReportHandler::new_themed(
            miette::GraphicalTheme::none(),
        )
        .with_width(200);
        let mut rendered = String::new();
        handler
            .render_report(&mut rendered, err.as_ref())
            .expect("render to String is infallible");

        let canonical_str = canonical_dir.display().to_string();
        let raw_str = dir.path().display().to_string();
        let stable: String = rendered
            .replace(&canonical_str, "<tmpdir>")
            .replace(&raw_str, "<tmpdir>");

        assert!(
            stable.contains("FromIdlOnly"),
            "should report spanned undefined type, got:\n{stable}"
        );
        assert!(
            stable.contains("FromJsonOnly"),
            "should report spanless undefined type as related, got:\n{stable}"
        );
        insta::assert_snapshot!(stable);
    }

    // =========================================================================
    // `Idl2Schemata::extract()` with directory input
    // =========================================================================
    //
    // The `extract_directory` code path (called when `extract()` receives a
    // directory) was previously untested. These tests verify that:
    // - schemas from multiple `.avdl` files are concatenated in sorted filename order
    // - non-`.avdl` files in the directory are ignored
    // - an empty directory (no `.avdl` files) returns an empty `SchemataOutput`
    // - subdirectories are walked recursively

    #[test]
    fn extract_directory_multiple_files() {
        let dir = tempfile::tempdir().expect("create temp dir");

        // Create three `.avdl` files with distinct schemas. The filenames are
        // chosen so their sorted order (a_, b_, c_) differs from any insertion
        // order we might accidentally rely on.
        std::fs::write(
            dir.path().join("b_second.avdl"),
            "protocol B { record Bravo { int id; } }",
        )
        .expect("write b_second.avdl");
        std::fs::write(
            dir.path().join("a_first.avdl"),
            "protocol A { record Alpha { string name; } }",
        )
        .expect("write a_first.avdl");
        std::fs::write(
            dir.path().join("c_third.avdl"),
            "protocol C { enum Gamma { X, Y, Z } }",
        )
        .expect("write c_third.avdl");

        // Also write a non-`.avdl` file that should be ignored.
        std::fs::write(dir.path().join("readme.txt"), "not avdl").expect("write readme.txt");

        let output = Idl2Schemata::new()
            .extract(dir.path())
            .expect("extract from directory should succeed");

        // We expect three schemas, one from each `.avdl` file, in sorted
        // filename order: a_first.avdl -> Alpha, b_second.avdl -> Bravo,
        // c_third.avdl -> Gamma.
        assert_eq!(
            output.schemas.len(),
            3,
            "should extract one schema per .avdl file"
        );
        assert_eq!(output.schemas[0].name, "Alpha");
        assert_eq!(output.schemas[1].name, "Bravo");
        assert_eq!(output.schemas[2].name, "Gamma");
    }

    #[test]
    fn extract_directory_empty() {
        let dir = tempfile::tempdir().expect("create temp dir");

        // Write a non-`.avdl` file so the directory is not completely empty on
        // disk, but still has no `.avdl` files to process.
        std::fs::write(dir.path().join("notes.txt"), "no avdl here").expect("write notes.txt");

        let output = Idl2Schemata::new()
            .extract(dir.path())
            .expect("extract from empty directory should succeed");

        assert!(
            output.schemas.is_empty(),
            "directory with no .avdl files should produce empty schemas"
        );
        assert!(
            output.warnings.is_empty(),
            "directory with no .avdl files should produce no warnings"
        );
    }

    #[test]
    fn extract_directory_recursive() {
        let dir = tempfile::tempdir().expect("create temp dir");

        // Create a nested directory structure:
        //   dir/
        //     top.avdl        -> record Top
        //     sub/
        //       nested.avdl   -> record Nested
        let sub = dir.path().join("sub");
        std::fs::create_dir(&sub).expect("create sub directory");

        std::fs::write(
            dir.path().join("top.avdl"),
            "protocol T { record Top { string a; } }",
        )
        .expect("write top.avdl");
        std::fs::write(
            sub.join("nested.avdl"),
            "protocol N { record Nested { int b; } }",
        )
        .expect("write nested.avdl");

        let output = Idl2Schemata::new()
            .extract(dir.path())
            .expect("extract from directory with subdirs should succeed");

        // walkdir sorts by filename within each directory level, and walks
        // depth-first. The exact order depends on walkdir's traversal, but
        // both schemas should be present.
        assert_eq!(
            output.schemas.len(),
            2,
            "should find .avdl files in subdirectories"
        );

        let names: Vec<&str> = output.schemas.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"Nested"),
            "should include schema from subdirectory, got: {names:?}"
        );
        assert!(
            names.contains(&"Top"),
            "should include schema from top-level, got: {names:?}"
        );
    }

    // =========================================================================
    // Import error paths in compiler (issue f512e05f, items 1-4)
    // =========================================================================

    #[test]
    fn import_resolution_error_has_source_span() {
        let result = Idl::new().convert_str(
            r#"
            protocol P {
                import schema "nonexistent-file.avsc";
            }
            "#,
        );
        let err = result.expect_err("missing import file should be rejected");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("import not found"),
            "should report import not found, got: {msg}"
        );
        assert!(
            msg.contains("nonexistent-file.avsc"),
            "should mention the missing file, got: {msg}"
        );
    }

    #[test]
    fn idl_import_parse_failure() {
        let dir = tempfile::tempdir().expect("create temp dir");

        let bad_avdl = dir.path().join("bad-syntax.avdl");
        std::fs::write(&bad_avdl, "this is not valid avdl {{{").expect("write bad .avdl");

        let main_avdl = dir.path().join("main.avdl");
        std::fs::write(
            &main_avdl,
            "protocol Main {\n  import idl \"bad-syntax.avdl\";\n}\n",
        )
        .expect("write main .avdl");

        let result = Idl::new().convert(&main_avdl);
        let err = result.expect_err("invalid imported IDL should be rejected");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("bad-syntax.avdl"),
            "error should mention the imported file, got: {msg}"
        );
    }

    #[test]
    fn idl_import_read_failure() {
        let dir = tempfile::tempdir().expect("create temp dir");

        let subdir = dir.path().join("not-a-file.avdl");
        std::fs::create_dir(&subdir).expect("create subdirectory");

        let main_avdl = dir.path().join("main.avdl");
        std::fs::write(
            &main_avdl,
            "protocol Main {\n  import idl \"not-a-file.avdl\";\n}\n",
        )
        .expect("write main .avdl");

        let result = Idl::new().convert(&main_avdl);
        let err = result.expect_err("reading a directory as IDL should fail");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("not-a-file.avdl"),
            "error should mention the import path, got: {msg}"
        );
    }

    #[test]
    fn nested_import_resolution_failure() {
        let dir = tempfile::tempdir().expect("create temp dir");

        let inner_avdl = dir.path().join("inner.avdl");
        std::fs::write(
            &inner_avdl,
            "protocol Inner {\n  import schema \"deeply-missing.avsc\";\n}\n",
        )
        .expect("write inner .avdl");

        let main_avdl = dir.path().join("main.avdl");
        std::fs::write(
            &main_avdl,
            "protocol Main {\n  import idl \"inner.avdl\";\n}\n",
        )
        .expect("write main .avdl");

        let result = Idl::new().convert(&main_avdl);
        let err = result.expect_err("nested missing import should fail");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("deeply-missing.avsc"),
            "error should mention the missing nested file, got: {msg}"
        );
    }

    #[test]
    fn protocol_import_with_invalid_json_shows_import_context() {
        let dir = tempfile::tempdir().expect("create temp dir");

        let avpr_path = dir.path().join("malformed.avpr");
        std::fs::write(&avpr_path, "{ not valid json }").expect("write malformed .avpr");

        let avdl_path = dir.path().join("test.avdl");
        std::fs::write(
            &avdl_path,
            "protocol Test {\n  import protocol \"malformed.avpr\";\n}\n",
        )
        .expect("write .avdl");

        let result = Idl::new().convert(&avdl_path);
        let err = result.expect_err("invalid JSON in .avpr should be rejected");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("malformed.avpr"),
            "error should mention the .avpr file, got: {msg}"
        );
        assert!(
            msg.contains("invalid JSON") || msg.contains("import protocol"),
            "error should indicate the nature of the failure, got: {msg}"
        );
    }

    #[test]
    fn schema_import_with_invalid_structure_shows_import_context() {
        let dir = tempfile::tempdir().expect("create temp dir");

        let avsc_path = dir.path().join("bad-structure.avsc");
        std::fs::write(&avsc_path, "42").expect("write invalid .avsc");

        let avdl_path = dir.path().join("test.avdl");
        std::fs::write(
            &avdl_path,
            "protocol Test {\n  import schema \"bad-structure.avsc\";\n}\n",
        )
        .expect("write .avdl");

        let result = Idl::new().convert(&avdl_path);
        let err = result.expect_err("invalid schema structure should be rejected");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("bad-structure.avsc"),
            "error should mention the .avsc file, got: {msg}"
        );
    }
}
