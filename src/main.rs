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

use clap::{Parser, Subcommand};
use indexmap::{IndexMap, IndexSet};
use miette::Context;

use avdl::error::IdlError;
use avdl::import::{import_protocol, import_schema, ImportContext};
use avdl::model::json::{build_lookup, protocol_to_json, schema_to_json};
use avdl::model::protocol::Message;
use avdl::reader::{parse_idl, DeclItem, IdlFile, ImportEntry, ImportKind};
use avdl::resolve::SchemaRegistry;

// ==============================================================================
// CLI Argument Definitions
// ==============================================================================

#[derive(Parser)]
#[command(name = "avdl", about = "Avro IDL compiler")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile an Avro IDL file to a JSON protocol (.avpr) or schema (.avsc).
    Idl {
        /// Input .avdl file (reads from stdin if omitted or `-`).
        input: Option<String>,
        /// Output file (writes to stdout if omitted or `-`).
        output: Option<String>,
        /// Additional directories to search for imports. May be repeated.
        #[arg(long = "import-dir")]
        import_dir: Vec<PathBuf>,
    },
    /// Extract individual .avsc schema files from an Avro IDL protocol.
    Idl2schemata {
        /// Input .avdl file (reads from stdin if omitted or `-`).
        input: Option<String>,
        /// Output directory for .avsc files (defaults to current directory).
        outdir: Option<PathBuf>,
        /// Additional directories to search for imports. May be repeated.
        #[arg(long = "import-dir")]
        import_dir: Vec<PathBuf>,
    },
}

// ==============================================================================
// Entry Point
// ==============================================================================

fn main() -> miette::Result<()> {
    miette::set_hook(Box::new(|_| {
        Box::new(miette::MietteHandlerOpts::new().build())
    }))?;

    let cli = Cli::parse();

    match cli.command {
        Command::Idl {
            input,
            output,
            import_dir,
        } => run_idl(input, output, import_dir),
        Command::Idl2schemata {
            input,
            outdir,
            import_dir,
        } => run_idl2schemata(input, outdir, import_dir),
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
    let (source, input_dir) = read_input(&input)?;
    let (idl_file, registry) = parse_and_resolve(&source, &input_dir, import_dirs)?;

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
            schema_to_json(schema, &mut IndexSet::new(), None, &lookup)
        }
        IdlFile::NamedSchemasFile(schemas) => {
            // Bare named type declarations (no `schema` keyword) are serialized
            // as a JSON array of all named schemas, matching Java's
            // `IdlFile.outputString()` behavior.
            let registry_schemas: Vec<_> = registry.schemas().cloned().collect();
            let lookup = build_lookup(&registry_schemas, None);
            let json_schemas: Vec<serde_json::Value> = schemas
                .iter()
                .map(|s| schema_to_json(s, &mut IndexSet::new(), None, &lookup))
                .collect();
            serde_json::Value::Array(json_schemas)
        }
    };

    let json_str = serde_json::to_string_pretty(&json_value)
        .map_err(|e| IdlError::Other(format!("serialize JSON: {e}")))
        .map_err(miette::Report::new)?;

    write_output(&output, &json_str)?;

    // Validate that all type references resolved. This is a non-fatal warning
    // check -- the output is still written, but unresolved references indicate
    // missing imports or forward reference errors.
    let unresolved = registry.validate_references();
    if !unresolved.is_empty() {
        eprintln!(
            "warning: unresolved type references: {}",
            unresolved.join(", ")
        );
    }

    Ok(())
}

// ==============================================================================
// `idl2schemata` Subcommand
// ==============================================================================

fn run_idl2schemata(
    input: Option<String>,
    outdir: Option<PathBuf>,
    import_dirs: Vec<PathBuf>,
) -> miette::Result<()> {
    let (source, input_dir) = read_input(&input)?;
    let (idl_file, registry) = parse_and_resolve(&source, &input_dir, import_dirs)?;

    // Extract the protocol namespace so that `schema_to_json` can shorten
    // fully-qualified type references to simple names when they share the
    // same namespace (matching Java's `Schema.toString(true)` behavior).
    let namespace = match &idl_file {
        IdlFile::ProtocolFile(protocol) => protocol.namespace.clone(),
        _ => None,
    };

    let output_dir = outdir.unwrap_or_else(|| PathBuf::from("."));
    fs::create_dir_all(&output_dir)
        .map_err(|e| IdlError::Io { source: e })
        .map_err(miette::Report::new)
        .wrap_err("create output directory")?;

    // Build a lookup table from all registered schemas so that references
    // within each schema can be resolved and inlined.
    let registry_schemas: Vec<_> = registry.schemas().cloned().collect();
    let all_lookup = build_lookup(&registry_schemas, None);

    // Share `known_names` across schema iterations so that types inlined
    // inside a record on first encounter are emitted as bare string
    // references in subsequent `.avsc` files, matching Java behavior.
    let mut known_names = IndexSet::new();

    // Write each named schema as an individual .avsc file.
    for schema in registry.schemas() {
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

        let json_value = schema_to_json(schema, &mut known_names, namespace.as_deref(), &all_lookup);
        let json_str = serde_json::to_string_pretty(&json_value)
            .map_err(|e| IdlError::Other(format!("serialize JSON for {type_name}: {e}")))
            .map_err(miette::Report::new)?;

        let file_path = output_dir.join(format!("{simple_name}.avsc"));
        fs::write(&file_path, &json_str)
            .map_err(|e| IdlError::Io { source: e })
            .map_err(miette::Report::new)
            .wrap_err_with(|| format!("write {}", file_path.display()))?;
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
fn read_input(input: &Option<String>) -> miette::Result<(String, PathBuf)> {
    let is_stdin = match input {
        None => true,
        Some(s) if s == "-" => true,
        _ => false,
    };

    if is_stdin {
        let mut source = String::new();
        io::stdin()
            .read_to_string(&mut source)
            .map_err(|e| IdlError::Io { source: e })
            .map_err(miette::Report::new)
            .wrap_err("read IDL from stdin")?;
        let cwd = std::env::current_dir()
            .map_err(|e| IdlError::Io { source: e })
            .map_err(miette::Report::new)
            .wrap_err("determine current directory")?;
        Ok((source, cwd))
    } else {
        let path = PathBuf::from(input.as_ref().expect("checked for None above"));
        let source = fs::read_to_string(&path)
            .map_err(|e| IdlError::Io { source: e })
            .map_err(miette::Report::new)
            .wrap_err_with(|| format!("read {}", path.display()))?;
        let dir = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        // Canonicalize the directory so that import cycle detection works
        // correctly with canonical paths.
        let dir = dir.canonicalize().unwrap_or(dir);
        Ok((source, dir))
    }
}

/// Parse IDL source and recursively resolve all imports.
///
/// The key insight for correct type ordering: `parse_idl` returns declaration
/// items (imports and local types) in source order. We process them
/// sequentially here -- resolving imports when encountered and registering
/// local types when encountered -- so the registry reflects declaration order.
fn parse_and_resolve(
    source: &str,
    input_dir: &Path,
    import_dirs: Vec<PathBuf>,
) -> miette::Result<(IdlFile, SchemaRegistry)> {
    let (idl_file, decl_items) =
        parse_idl(source).map_err(miette::Report::new)?;

    let mut registry = SchemaRegistry::new();
    let mut import_ctx = ImportContext::new(import_dirs);
    let mut messages = IndexMap::new();

    // Process declaration items in source order: resolve imports when
    // encountered, register local types when encountered. This ensures the
    // registry reflects the original declaration order from the IDL file.
    process_decl_items(
        &decl_items,
        &mut registry,
        &mut import_ctx,
        input_dir,
        &mut messages,
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

    Ok((idl_file, registry))
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
    messages: &mut IndexMap<String, Message>,
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
                )?;
            }
            DeclItem::Type(schema) => {
                // Register the locally-defined type in the registry at this
                // position, preserving its source-order placement relative to
                // imports.
                registry
                    .register(schema.clone())
                    .map_err(IdlError::Other)
                    .map_err(miette::Report::new)?;
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
    messages: &mut IndexMap<String, Message>,
) -> miette::Result<()> {
    let resolved_path = import_ctx
        .resolve_import(&import.path, current_dir)
        .map_err(miette::Report::new)?;

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
                .map_err(miette::Report::new)
                .wrap_err_with(|| {
                    format!("import protocol {}", resolved_path.display())
                })?;
            messages.extend(imported_messages);
        }
        ImportKind::Schema => {
            import_schema(&resolved_path, registry)
                .map_err(miette::Report::new)
                .wrap_err_with(|| {
                    format!("import schema {}", resolved_path.display())
                })?;
        }
        ImportKind::Idl => {
            let imported_source = fs::read_to_string(&resolved_path)
                .map_err(|e| IdlError::Io { source: e })
                .map_err(miette::Report::new)
                .wrap_err_with(|| {
                    format!("read imported IDL {}", resolved_path.display())
                })?;

            let (imported_idl, nested_decl_items) =
                parse_idl(&imported_source)
                    .map_err(miette::Report::new)
                    .wrap_err_with(|| {
                        format!("parse imported IDL {}", resolved_path.display())
                    })?;

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
            )?;
        }
    }

    Ok(())
}

/// Write output to a file or stdout.
fn write_output(output: &Option<String>, content: &str) -> miette::Result<()> {
    let is_stdout = match output {
        None => true,
        Some(s) if s == "-" => true,
        _ => false,
    };

    if is_stdout {
        // Write to stdout without trailing newline, matching Java behavior.
        print!("{content}");
        Ok(())
    } else {
        let path = PathBuf::from(output.as_ref().expect("checked for None above"));
        fs::write(&path, content)
            .map_err(|e| IdlError::Io { source: e })
            .map_err(miette::Report::new)
            .wrap_err_with(|| format!("write {}", path.display()))
    }
}
