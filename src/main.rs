// ==============================================================================
// CLI for the Avro IDL Parser
// ==============================================================================
//
// Two subcommands that mirror the Java `avro-tools` interface:
//   - `avdl idl [INPUT] [OUTPUT]`        -- compile .avdl to .avpr or .avsc JSON
//   - `avdl idl2schemata [INPUT] [OUTDIR]` -- extract individual .avsc files

use std::fs;
use std::io::{self, Read as _};
use std::path::{Path, PathBuf};

use avdl::import::{ImportContext, import_protocol, import_schema};
use avdl::model::json::{build_lookup, protocol_to_json, schema_to_json, to_string_pretty_java};
use avdl::model::protocol::Message;
use avdl::reader::{DeclItem, IdlFile, ImportEntry, ImportKind, Warning, parse_idl_named};
use avdl::resolve::SchemaRegistry;
use lexopt::prelude::*;
use miette::{Context, IntoDiagnostic};
use std::collections::{HashMap, HashSet};

// ==============================================================================
// CLI Help Text
// ==============================================================================

const MAIN_HELP: &str = "\
avdl - Avro IDL compiler

Usage: avdl <COMMAND>

Commands:
  idl           Compile an Avro IDL file to protocol (.avpr) or schema (.avsc) JSON
  idl2schemata  Extract individual .avsc schema files from an Avro IDL protocol

Options:
  -h, --help    Print help";

const IDL_HELP: &str = "\
Usage: avdl idl [OPTIONS] [INPUT] [OUTPUT]

Options:
      --import-dir <DIR>  Additional directories to search for imports (repeatable)
  -h, --help              Print help";

const IDL2SCHEMATA_HELP: &str = "\
Usage: avdl idl2schemata [OPTIONS] INPUT [OUTDIR]

Options:
      --import-dir <DIR>  Additional directories to search for imports (repeatable)
  -h, --help              Print help";

// ==============================================================================
// Argument Parsing
// ==============================================================================

/// Parsed CLI arguments for the `idl` subcommand.
struct IdlArgs {
    input: Option<String>,
    output: Option<String>,
    import_dirs: Vec<PathBuf>,
}

/// Parsed CLI arguments for the `idl2schemata` subcommand.
struct Idl2schemataArgs {
    input: String,
    outdir: Option<PathBuf>,
    import_dirs: Vec<PathBuf>,
}

/// Parse `--import-dir` and positional args for the `idl` subcommand.
fn parse_idl_args(parser: &mut lexopt::Parser) -> Result<IdlArgs, lexopt::Error> {
    let mut import_dirs = Vec::new();
    let mut positionals: Vec<String> = Vec::new();

    while let Some(arg) = parser.next()? {
        match arg {
            Long("import-dir") => {
                let val: String = parser.value()?.string()?;
                import_dirs.push(PathBuf::from(val));
            }
            Short('h') | Long("help") => {
                println!("{IDL_HELP}");
                std::process::exit(0);
            }
            Value(val) => {
                positionals.push(val.string()?);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    let input = positionals.first().cloned();
    let output = positionals.get(1).cloned();

    Ok(IdlArgs {
        input,
        output,
        import_dirs,
    })
}

/// Parse `--import-dir` and positional args for the `idl2schemata` subcommand.
fn parse_idl2schemata_args(parser: &mut lexopt::Parser) -> Result<Idl2schemataArgs, lexopt::Error> {
    let mut import_dirs = Vec::new();
    let mut positionals: Vec<String> = Vec::new();

    while let Some(arg) = parser.next()? {
        match arg {
            Long("import-dir") => {
                let val: String = parser.value()?.string()?;
                import_dirs.push(PathBuf::from(val));
            }
            Short('h') | Long("help") => {
                println!("{IDL2SCHEMATA_HELP}");
                std::process::exit(0);
            }
            Value(val) => {
                positionals.push(val.string()?);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    let input = positionals
        .first()
        .cloned()
        .ok_or_else(|| lexopt::Error::MissingValue {
            option: Some("INPUT".to_string()),
        })?;
    let outdir = positionals.get(1).map(PathBuf::from);

    Ok(Idl2schemataArgs {
        input,
        outdir,
        import_dirs,
    })
}

// ==============================================================================
// Entry Point
// ==============================================================================

fn main() -> miette::Result<()> {
    miette::set_hook(Box::new(|_| {
        Box::new(miette::MietteHandlerOpts::new().build())
    }))?;

    let mut parser = lexopt::Parser::from_env();

    // The first positional value is the subcommand name.
    let subcommand = match parser.next() {
        Ok(Some(Value(val))) => val.string().map_err(|e| miette::miette!("{e}"))?,
        Ok(Some(Short('h') | Long("help"))) => {
            println!("{MAIN_HELP}");
            return Ok(());
        }
        Ok(Some(other)) => {
            let err = other.unexpected();
            eprintln!("error: {err}\n\n{MAIN_HELP}");
            std::process::exit(2);
        }
        Ok(None) => {
            eprintln!("error: a subcommand is required\n\n{MAIN_HELP}");
            std::process::exit(2);
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(2);
        }
    };

    match subcommand.as_str() {
        "idl" => {
            let args = parse_idl_args(&mut parser).map_err(|e| miette::miette!("{e}"))?;
            run_idl(args.input, args.output, args.import_dirs)
        }
        "idl2schemata" => {
            let args = parse_idl2schemata_args(&mut parser).map_err(|e| miette::miette!("{e}"))?;
            run_idl2schemata(args.input, args.outdir, args.import_dirs)
        }
        other => {
            eprintln!("error: unknown subcommand `{other}`\n\n{MAIN_HELP}");
            std::process::exit(2);
        }
    }
}

// ==============================================================================
// `idl` Subcommand
// ==============================================================================

fn run_idl(
    input: Option<String>,
    output: Option<String>,
    import_dirs: Vec<PathBuf>,
) -> miette::Result<()> {
    let (source, input_dir, input_path) = read_input(&input)?;
    let source_name = input.as_deref().unwrap_or("<stdin>");
    let (idl_file, registry, warnings) =
        parse_and_resolve(&source, source_name, &input_dir, input_path, import_dirs)?;

    // Emit any warnings to stderr, matching Java's `IdlTool` which prints
    // "Warning: " + message for each warning.
    emit_warnings(&warnings);

    // Serialize the parsed IDL to JSON. Protocols become .avpr, standalone
    // schemas become .avsc.
    let json_value = match &idl_file {
        IdlFile::ProtocolFile(protocol) => protocol_to_json(protocol),
        IdlFile::SchemaFile(schema) => {
            // In schema mode, we need to build a lookup table from the registry
            // so that Reference nodes (forward references, imported types) can
            // be resolved and inlined. Protocol mode does this inside
            // `protocol_to_json`, but schema mode must do it explicitly.
            let registry_schemas: Vec<_> = registry.schemas().cloned().collect();
            let lookup = build_lookup(&registry_schemas, None);
            schema_to_json(schema, &mut HashSet::new(), None, &lookup)
        }
        IdlFile::NamedSchemasFile(schemas) => {
            // Bare named type declarations (no `schema` keyword) are serialized
            // as a JSON array of all named schemas, matching Java's
            // `IdlFile.outputString()` behavior.
            let registry_schemas: Vec<_> = registry.schemas().cloned().collect();
            let lookup = build_lookup(&registry_schemas, None);
            let json_schemas: Vec<serde_json::Value> = schemas
                .iter()
                .map(|s| schema_to_json(s, &mut HashSet::new(), None, &lookup))
                .collect();
            serde_json::Value::Array(json_schemas)
        }
    };

    let json_str =
        to_string_pretty_java(&json_value).map_err(|e| miette::miette!("serialize JSON: {e}"))?;

    // Validate that all type references resolved before writing output.
    // Unresolved references indicate missing imports, undefined types, or
    // cross-namespace references that need fully-qualified names. Java's
    // IdlReader treats these as fatal errors ("Undefined name/schema"),
    // so we do the same.
    let mut unresolved = registry.validate_references();

    // The `SchemaFile` and `NamedSchemasFile` variants store their top-level
    // schemas outside the registry, so `validate_references` alone misses
    // unresolved references in them. For example, `schema DoesNotExist;`
    // produces an unresolved `Reference` that is never registered. We check
    // these separately to match Java's "Undefined schema" error.
    match &idl_file {
        IdlFile::SchemaFile(schema) => {
            unresolved.extend(registry.validate_schema(schema));
        }
        IdlFile::NamedSchemasFile(schemas) => {
            for schema in schemas {
                unresolved.extend(registry.validate_schema(schema));
            }
        }
        IdlFile::ProtocolFile(_) => {
            // Protocol types are already in the registry; no extra check needed.
        }
    }

    unresolved.sort();
    unresolved.dedup();
    if !unresolved.is_empty() {
        miette::bail!("Undefined name: {}", unresolved.join(", "));
    }

    write_output(&output, &json_str)?;

    Ok(())
}

// ==============================================================================
// `idl2schemata` Subcommand
// ==============================================================================

fn run_idl2schemata(
    input: String,
    outdir: Option<PathBuf>,
    import_dirs: Vec<PathBuf>,
) -> miette::Result<()> {
    let source_name = input.clone();
    let (source, input_dir, input_path) = read_input(&Some(input))?;
    let (idl_file, registry, warnings) =
        parse_and_resolve(&source, &source_name, &input_dir, input_path, import_dirs)?;

    // Emit any warnings to stderr.
    emit_warnings(&warnings);

    let output_dir = outdir.unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&output_dir)
        .into_diagnostic()
        .wrap_err("create output directory")?;

    // Build a lookup table from all registered schemas so that references
    // within each schema can be resolved and inlined.
    let registry_schemas: Vec<_> = registry.schemas().cloned().collect();
    let all_lookup = build_lookup(&registry_schemas, None);

    // Write each named schema as an individual .avsc file. Each schema gets
    // its own `known_names` set, matching Java's `Schema.toString(true)` which
    // creates a fresh `HashSet` per call. This ensures each `.avsc` file is
    // self-contained with all referenced types inlined on first occurrence.
    for schema in registry.schemas() {
        let mut known_names = HashSet::new();
        let type_name = match schema.full_name() {
            Some(name) => name,
            // Skip non-named schemas (primitives, etc.).
            None => continue,
        };

        // Use the simple name (after the last dot) for the filename, matching
        // Java avro-tools behavior.
        let simple_name = match schema.name() {
            Some(n) => n,
            None => continue,
        };

        // Pass `None` as `enclosing_namespace` so that each standalone `.avsc`
        // file always includes an explicit `"namespace"` key. Java's
        // `Schema.toString(true)` passes `null` for the same reason — standalone
        // schemas have no enclosing context to inherit namespace from.
        let json_value = schema_to_json(schema, &mut known_names, None, &all_lookup);
        let json_str = to_string_pretty_java(&json_value)
            .map_err(|e| miette::miette!("serialize JSON for {type_name}: {e}"))?;

        let file_path = output_dir.join(format!("{simple_name}.avsc"));
        // Append trailing newline to match Java's `PrintStream.println()`.
        fs::write(&file_path, format!("{json_str}\n"))
            .into_diagnostic()
            .wrap_err_with(|| format!("write {}", file_path.display()))?;
    }

    // Validate that all type references resolved, matching the `idl`
    // subcommand's behavior. Without this, unresolved references silently
    // produce bare name strings in the output `.avsc` files.
    let mut unresolved = registry.validate_references();

    // Also validate the top-level schema from `SchemaFile` / `NamedSchemasFile`,
    // which is not registered in the registry and would otherwise escape
    // validation. See the equivalent check in `run_idl` for details.
    match &idl_file {
        IdlFile::SchemaFile(schema) => {
            unresolved.extend(registry.validate_schema(schema));
        }
        IdlFile::NamedSchemasFile(schemas) => {
            for schema in schemas {
                unresolved.extend(registry.validate_schema(schema));
            }
        }
        IdlFile::ProtocolFile(_) => {}
    }

    unresolved.sort();
    unresolved.dedup();
    if !unresolved.is_empty() {
        miette::bail!("Undefined name: {}", unresolved.join(", "));
    }

    Ok(())
}

// ==============================================================================
// Shared Helpers: Input Reading, Parsing, and Import Resolution
// ==============================================================================

/// Read the IDL source text from a file or stdin.
///
/// Returns the source text and the directory containing the input file (used
/// as the base for resolving relative imports). When reading from stdin, the
/// current working directory is used as the import base.
/// Read the IDL source text from a file or stdin.
///
/// Returns the source text, the directory containing the input file (used as
/// the base for resolving relative imports), and the canonical path of the
/// input file (used for import cycle detection). When reading from stdin, the
/// canonical path is `None` since there is no file to track.
fn read_input(input: &Option<String>) -> miette::Result<(String, PathBuf, Option<PathBuf>)> {
    // Treat `None` and `Some("-")` as stdin; everything else is a file path.
    let file_path = input.as_deref().filter(|s| *s != "-");

    match file_path {
        None => {
            let mut source = String::new();
            io::stdin()
                .read_to_string(&mut source)
                .into_diagnostic()
                .wrap_err("read IDL from stdin")?;
            let cwd = std::env::current_dir()
                .into_diagnostic()
                .wrap_err("determine current directory")?;
            Ok((source, cwd, None))
        }
        Some(file_path) => {
            let path = PathBuf::from(file_path);
            let source = fs::read_to_string(&path)
                .into_diagnostic()
                .wrap_err_with(|| format!("read {}", path.display()))?;
            let dir = path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."));
            // Canonicalize the directory so that import cycle detection works
            // correctly with canonical paths.
            let dir = dir.canonicalize().unwrap_or(dir);
            let canonical_path = path.canonicalize().ok();
            Ok((source, dir, canonical_path))
        }
    }
}

/// Parse IDL source and recursively resolve all imports.
///
/// The key insight for correct type ordering: `parse_idl` returns declaration
/// items (imports and local types) in source order. We process them
/// sequentially here -- resolving imports when encountered and registering
/// local types when encountered -- so the registry reflects declaration order.
///
/// Returns the parsed IDL file, schema registry, and any warnings collected
/// during parsing (including warnings from imported files).
fn parse_and_resolve(
    source: &str,
    source_name: &str,
    input_dir: &Path,
    input_path: Option<PathBuf>,
    import_dirs: Vec<PathBuf>,
) -> miette::Result<(IdlFile, SchemaRegistry, Vec<Warning>)> {
    let (idl_file, decl_items, mut warnings) =
        parse_idl_named(source, source_name).wrap_err("parse IDL source")?;

    let mut registry = SchemaRegistry::new();
    let mut import_ctx = ImportContext::new(import_dirs);
    let mut messages = HashMap::new();

    // Mark the initial input file as "imported" before processing declaration
    // items, so that self-imports (direct or via a chain) are detected as
    // cycles and silently skipped. Without this, a file importing itself would
    // cause a confusing "duplicate schema name" error. This matches Java's
    // `IdlReader.readLocations` which includes the initial file.
    if let Some(path) = input_path {
        import_ctx.mark_imported(&path);
    }

    // Process declaration items in source order: resolve imports when
    // encountered, register local types when encountered. This ensures the
    // registry reflects the original declaration order from the IDL file.
    process_decl_items(
        &decl_items,
        &mut registry,
        &mut import_ctx,
        input_dir,
        &mut messages,
        &mut warnings,
        source,
        source_name,
    )?;

    // For protocol files, rebuild the types list from the registry (which now
    // includes imported types in declaration order) and prepend imported
    // messages before the protocol's own messages so that the output order
    // matches Java behavior.
    let idl_file = match idl_file {
        IdlFile::ProtocolFile(mut protocol) => {
            protocol.types = registry.schemas().cloned().collect();
            let own_messages = std::mem::take(&mut protocol.messages);
            protocol.messages = messages;
            protocol.messages.extend(own_messages);
            IdlFile::ProtocolFile(protocol)
        }
        other => other,
    };

    Ok((idl_file, registry, warnings))
}

/// Process declaration items (imports and local types) in source order.
///
/// This function iterates the interleaved declaration items, resolving imports
/// when encountered and registering local types when encountered. This ensures
/// the registry reflects the correct declaration order from the IDL file,
/// matching Java's behavior where imported types appear at the position of
/// their import statement.
fn process_decl_items(
    decl_items: &[DeclItem],
    registry: &mut SchemaRegistry,
    import_ctx: &mut ImportContext,
    current_dir: &Path,
    messages: &mut HashMap<String, Message>,
    warnings: &mut Vec<Warning>,
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
                // Register the locally-defined type in the registry at this
                // position, preserving its source-order placement relative to
                // imports.
                if let Err(msg) = registry.register(schema.clone()) {
                    if let Some(span) = span {
                        return Err(avdl::error::ParseDiagnostic {
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
fn resolve_single_import(
    import: &ImportEntry,
    registry: &mut SchemaRegistry,
    import_ctx: &mut ImportContext,
    current_dir: &Path,
    messages: &mut HashMap<String, Message>,
    warnings: &mut Vec<Warning>,
    source: &str,
    source_name: &str,
) -> miette::Result<()> {
    let resolved_path = match import_ctx.resolve_import(&import.path, current_dir) {
        Ok(p) => p,
        Err(e) => {
            // When the import statement has a source span, wrap the error in a
            // `ParseDiagnostic` so that miette renders the offending import
            // statement with source highlighting.
            if let Some(span) = import.span {
                return Err(avdl::error::ParseDiagnostic {
                    src: miette::NamedSource::new(source_name, source.to_string()),
                    span,
                    message: format!("{e}"),
                }
                .into());
            }
            return Err(e).wrap_err_with(|| format!("resolve import `{}`", import.path));
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
            let imported_messages = import_protocol(&resolved_path, registry)
                .wrap_err_with(|| format!("import protocol {}", resolved_path.display()))?;
            messages.extend(imported_messages);
        }
        ImportKind::Schema => {
            import_schema(&resolved_path, registry)
                .wrap_err_with(|| format!("import schema {}", resolved_path.display()))?;
        }
        ImportKind::Idl => {
            let imported_source = fs::read_to_string(&resolved_path)
                .into_diagnostic()
                .wrap_err_with(|| format!("read imported IDL {}", resolved_path.display()))?;

            let imported_name = resolved_path.display().to_string();
            let (imported_idl, nested_decl_items, import_warnings) =
                parse_idl_named(&imported_source, &imported_name)
                    .wrap_err_with(|| format!("parse imported IDL {}", resolved_path.display()))?;

            // Propagate warnings from the imported file, prepending the import
            // filename to each warning message. This matches Java's
            // `warnings.addAll(idlFile.getWarnings(importFile))` in
            // `IdlReader.exitImportStatement()`.
            let import_file_name = resolved_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(import.path.as_str());
            for w in import_warnings {
                warnings.push(w.with_import_prefix(import_file_name));
            }

            // If the imported IDL is a protocol, merge its messages.
            if let IdlFile::ProtocolFile(imported_protocol) = &imported_idl {
                messages.extend(imported_protocol.messages.clone());
            }

            // Recursively process declaration items from the imported file,
            // preserving their source order for correct type ordering.
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
            .wrap_err_with(|| {
                format!("resolve nested imports from `{}`", resolved_path.display())
            })?;
        }
    }

    Ok(())
}

/// Emit warnings to stderr, matching Java's `IdlTool` format:
///   "Warning: " + message
fn emit_warnings(warnings: &[Warning]) {
    for w in warnings {
        eprintln!("Warning: {w}");
    }
}

/// Write output to a file or stdout.
fn write_output(output: &Option<String>, content: &str) -> miette::Result<()> {
    // Treat `None` and `Some("-")` as stdout; everything else is a file path.
    let file_path = output.as_deref().filter(|s| *s != "-");

    match file_path {
        None => {
            // Write to stdout without trailing newline, matching Java behavior.
            // Handle BrokenPipe gracefully: when output is piped to a command
            // that closes early (e.g., `avdl idl file.avdl | head -1`), exit
            // silently instead of panicking, matching Unix CLI conventions.
            use std::io::Write;
            if let Err(e) = write!(io::stdout(), "{content}") {
                if e.kind() == io::ErrorKind::BrokenPipe {
                    return Ok(());
                }
                return Err(e).into_diagnostic().wrap_err("write to stdout");
            }
            Ok(())
        }
        Some(file_path) => {
            let path = PathBuf::from(file_path);
            // Append a trailing newline to match the golden files. Java's
            // `IdlTool` uses `PrintStream.println()` which adds one.
            fs::write(&path, format!("{content}\n"))
                .into_diagnostic()
                .wrap_err_with(|| format!("write {}", path.display()))
        }
    }
}
