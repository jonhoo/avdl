// ==============================================================================
// Integration Tests: Parse .avdl Files and Compare Against Expected JSON Output
// ==============================================================================
//
// Each test reads an `.avdl` input file from the Avro test suite, parses it
// through our IDL reader, serializes the result to JSON, and compares it
// semantically against the expected `.avpr` or `.avsc` output file.
//
// We compare parsed `serde_json::Value` trees rather than strings, so
// differences in whitespace or key ordering within standard JSON objects do not
// cause false failures. (Note: Avro's JSON output preserves key order via
// `IndexMap`, so order-sensitive comparison is actually desirable for full
// fidelity, but `serde_json::Value` uses `BTreeMap` internally, which
// normalizes key order. This is acceptable for correctness testing.)

use std::fs;
use std::path::{Path, PathBuf};

use avdl::import::{import_protocol, import_schema, ImportContext};
use avdl::model::json::{build_lookup, protocol_to_json, schema_to_json};
use avdl::reader::{parse_idl, IdlFile, ImportKind};
use indexmap::IndexSet;
use pretty_assertions::assert_eq;
use serde_json::Value;

const INPUT_DIR: &str = "avro/lang/java/idl/src/test/idl/input";
const OUTPUT_DIR: &str = "avro/lang/java/idl/src/test/idl/output";

// ==============================================================================
// Test Infrastructure
// ==============================================================================

/// Parse an `.avdl` file and serialize the result to a `serde_json::Value`.
///
/// For protocol files, this handles import resolution for `schema` and
/// `protocol` import kinds (loading `.avsc` and `.avpr` files). IDL imports
/// require recursive parsing which is handled separately in
/// `parse_and_serialize_with_idl_imports`.
///
/// For schema files, the main schema declaration is serialized directly.
fn parse_and_serialize(avdl_path: &Path, import_dirs: &[&Path]) -> Value {
    let input = fs::read_to_string(avdl_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", avdl_path.display()));

    let (idl_file, mut registry, imports) =
        parse_idl(&input).unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    // Resolve non-IDL imports (schema and protocol). IDL imports require
    // recursive parsing, which is handled by the caller if needed.
    let current_dir = avdl_path
        .parent()
        .expect("avdl_path should have a parent directory");
    let search_dirs: Vec<PathBuf> = import_dirs.iter().map(|p| p.to_path_buf()).collect();
    let import_ctx = ImportContext::new(search_dirs);

    for import in &imports {
        let resolved = import_ctx
            .resolve_import(&import.path, current_dir)
            .unwrap_or_else(|e| {
                panic!(
                    "failed to resolve import '{}' from {}: {e}",
                    import.path,
                    avdl_path.display()
                )
            });

        match import.kind {
            ImportKind::Schema => {
                import_schema(&resolved, &mut registry)
                    .unwrap_or_else(|e| panic!("failed to import schema {}: {e}", resolved.display()));
            }
            ImportKind::Protocol => {
                let messages = import_protocol(&resolved, &mut registry)
                    .unwrap_or_else(|e| {
                        panic!("failed to import protocol {}: {e}", resolved.display())
                    });
                // Merge imported messages into the protocol (handled below
                // when we reconstruct the protocol).
                if let IdlFile::ProtocolFile(_) = &idl_file {
                    // Protocol import messages are merged below.
                    // We store them temporarily -- but since our current test
                    // cases with protocol imports are skipped, this is a
                    // placeholder.
                    let _ = messages;
                }
            }
            ImportKind::Idl => {
                // IDL imports require recursive parsing. For test cases that
                // need this, use `parse_and_serialize_with_idl_imports` instead.
                panic!(
                    "IDL imports not supported in basic parse_and_serialize; \
                     use parse_and_serialize_with_idl_imports for '{}'",
                    import.path
                );
            }
        }
    }

    idl_file_to_json(idl_file, registry)
}

/// Serialize an `IdlFile` to a `serde_json::Value`, using the given registry
/// for reference resolution.
fn idl_file_to_json(idl_file: IdlFile, registry: avdl::resolve::SchemaRegistry) -> Value {
    match idl_file {
        IdlFile::ProtocolFile(mut protocol) => {
            // Merge any types discovered during import resolution into the
            // protocol's type list. The registry now contains both the
            // protocol's own types and any imported types.
            protocol.types = registry.into_schemas();
            protocol_to_json(&protocol)
        }
        IdlFile::SchemaFile(schema) => {
            // Build a lookup from registry schemas so that references can be
            // resolved and inlined, matching the protocol-mode behavior.
            let registry_schemas: Vec<_> = registry.schemas().cloned().collect();
            let lookup = build_lookup(&registry_schemas, None);
            schema_to_json(&schema, &mut IndexSet::new(), None, &lookup)
        }
        IdlFile::NamedSchemasFile(schemas) => {
            // Bare named type declarations are serialized as a JSON array.
            let registry_schemas: Vec<_> = registry.schemas().cloned().collect();
            let lookup = build_lookup(&registry_schemas, None);
            let json_schemas: Vec<Value> = schemas
                .iter()
                .map(|s| schema_to_json(s, &mut IndexSet::new(), None, &lookup))
                .collect();
            Value::Array(json_schemas)
        }
    }
}

/// Parse an `.avdl` file that has IDL imports, recursively resolving them.
///
/// This handles the full import pipeline: for each `import idl` statement,
/// it recursively parses the imported `.avdl` file and merges its types and
/// messages into the current protocol.
fn parse_and_serialize_with_idl_imports(avdl_path: &Path, import_dirs: &[&Path]) -> Value {
    let input = fs::read_to_string(avdl_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", avdl_path.display()));

    let (idl_file, mut registry, imports) =
        parse_idl(&input).unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    let current_dir = avdl_path
        .parent()
        .expect("avdl_path should have a parent directory");
    let search_dirs: Vec<PathBuf> = import_dirs.iter().map(|p| p.to_path_buf()).collect();
    let mut import_ctx = ImportContext::new(search_dirs);

    // Mark the current file as imported to prevent cycles.
    let canonical = avdl_path
        .canonicalize()
        .unwrap_or_else(|e| panic!("failed to canonicalize {}: {e}", avdl_path.display()));
    import_ctx.mark_imported(&canonical);

    // Accumulate messages from IDL imports.
    let mut imported_messages = indexmap::IndexMap::new();

    for import in &imports {
        let resolved = import_ctx
            .resolve_import(&import.path, current_dir)
            .unwrap_or_else(|e| {
                panic!(
                    "failed to resolve import '{}' from {}: {e}",
                    import.path,
                    avdl_path.display()
                )
            });

        // Skip already-imported files (cycle prevention).
        if import_ctx.mark_imported(&resolved) {
            continue;
        }

        match import.kind {
            ImportKind::Schema => {
                import_schema(&resolved, &mut registry).unwrap_or_else(|e| {
                    panic!("failed to import schema {}: {e}", resolved.display())
                });
            }
            ImportKind::Protocol => {
                let messages = import_protocol(&resolved, &mut registry).unwrap_or_else(|e| {
                    panic!("failed to import protocol {}: {e}", resolved.display())
                });
                imported_messages.extend(messages);
            }
            ImportKind::Idl => {
                // Recursively parse the imported IDL file.
                let imported_input = fs::read_to_string(&resolved).unwrap_or_else(|e| {
                    panic!("failed to read imported IDL {}: {e}", resolved.display())
                });
                let (imported_idl, imported_registry, _nested_imports) =
                    parse_idl(&imported_input).unwrap_or_else(|e| {
                        panic!("failed to parse imported IDL {}: {e}", resolved.display())
                    });

                // Merge the imported registry into ours.
                registry.merge(imported_registry);

                // If the imported file is a protocol, merge its messages too.
                if let IdlFile::ProtocolFile(imported_protocol) = imported_idl {
                    imported_messages.extend(imported_protocol.messages);
                }
            }
        }
    }

    // For protocol files, prepend imported messages before dispatching.
    let idl_file = match idl_file {
        IdlFile::ProtocolFile(mut protocol) => {
            let own_messages = protocol.messages;
            protocol.messages = imported_messages;
            protocol.messages.extend(own_messages);
            IdlFile::ProtocolFile(protocol)
        }
        other => other,
    };

    idl_file_to_json(idl_file, registry)
}

/// Load an expected output file (`.avpr` or `.avsc`) as a `serde_json::Value`.
fn load_expected(path: &Path) -> Value {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read expected output {}: {e}", path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("failed to parse expected JSON {}: {e}", path.display()))
}

/// Helper to construct the input path for a test case.
fn input_path(filename: &str) -> PathBuf {
    PathBuf::from(INPUT_DIR).join(filename)
}

/// Helper to construct the output path for a test case.
fn output_path(filename: &str) -> PathBuf {
    PathBuf::from(OUTPUT_DIR).join(filename)
}

// ==============================================================================
// Protocol Tests (no imports needed)
// ==============================================================================

#[test]
fn test_echo() {
    let actual = parse_and_serialize(&input_path("echo.avdl"), &[]);
    let expected = load_expected(&output_path("echo.avpr"));
    assert_eq!(actual, expected);
}

#[test]
fn test_forward_ref() {
    let actual = parse_and_serialize(&input_path("forward_ref.avdl"), &[]);
    let expected = load_expected(&output_path("forward_ref.avpr"));
    assert_eq!(actual, expected);
}

#[test]
fn test_simple() {
    let actual = parse_and_serialize(&input_path("simple.avdl"), &[]);
    let expected = load_expected(&output_path("simple.avpr"));
    assert_eq!(actual, expected);
}

#[test]
fn test_comments() {
    let actual = parse_and_serialize(&input_path("comments.avdl"), &[]);
    let expected = load_expected(&output_path("comments.avpr"));
    assert_eq!(actual, expected);
}

#[test]
fn test_union() {
    let actual = parse_and_serialize(&input_path("union.avdl"), &[]);
    let expected = load_expected(&output_path("union.avpr"));
    assert_eq!(actual, expected);
}

#[test]
fn test_uuid() {
    let actual = parse_and_serialize(&input_path("uuid.avdl"), &[]);
    let expected = load_expected(&output_path("uuid.avpr"));
    assert_eq!(actual, expected);
}

#[test]
fn test_namespaces() {
    let actual = parse_and_serialize(&input_path("namespaces.avdl"), &[]);
    let expected = load_expected(&output_path("namespaces.avpr"));
    assert_eq!(actual, expected);
}

#[test]
fn test_leading_underscore() {
    let actual = parse_and_serialize(&input_path("leading_underscore.avdl"), &[]);
    let expected = load_expected(&output_path("leading_underscore.avpr"));
    assert_eq!(actual, expected);
}

#[test]
fn test_reservedwords() {
    let actual = parse_and_serialize(&input_path("reservedwords.avdl"), &[]);
    let expected = load_expected(&output_path("reservedwords.avpr"));
    assert_eq!(actual, expected);
}

#[test]
fn test_unicode() {
    let actual = parse_and_serialize(&input_path("unicode.avdl"), &[]);
    let expected = load_expected(&output_path("unicode.avpr"));
    assert_eq!(actual, expected);
}

#[test]
fn test_mr_events() {
    let actual = parse_and_serialize(&input_path("mr_events.avdl"), &[]);
    let expected = load_expected(&output_path("mr_events.avpr"));
    assert_eq!(actual, expected);
}

#[test]
fn test_interop() {
    let actual = parse_and_serialize(&input_path("interop.avdl"), &[]);
    let expected = load_expected(&output_path("interop.avpr"));
    assert_eq!(actual, expected);
}

#[test]
fn test_cycle() {
    let actual = parse_and_serialize(&input_path("cycle.avdl"), &[]);
    let expected = load_expected(&output_path("cycle.avpr"));
    assert_eq!(actual, expected);
}

// ==============================================================================
// Protocol Tests (with schema/protocol imports)
// ==============================================================================

#[test]
fn test_baseball() {
    // baseball.avdl imports position.avsc and player.avsc, both in the input
    // directory. These are schema imports, not IDL imports, so
    // `parse_and_serialize` handles them.
    let input_dir = PathBuf::from(INPUT_DIR);
    let actual = parse_and_serialize(&input_path("baseball.avdl"), &[&input_dir]);
    let expected = load_expected(&output_path("baseball.avpr"));
    assert_eq!(actual, expected);
}

// ==============================================================================
// Schema Tests (standalone schema syntax)
// ==============================================================================

/// The `schema_syntax_schema.avdl` test uses schema mode and has imports
/// (including an `import idl` for `status_schema.avdl`). We use the full
/// import-resolution pipeline.
#[test]
fn test_schema_syntax() {
    let input_dir = PathBuf::from(INPUT_DIR);
    let actual = parse_and_serialize_with_idl_imports(
        &input_path("schema_syntax_schema.avdl"),
        &[&input_dir],
    );
    let expected = load_expected(&output_path("schema_syntax.avsc"));
    assert_eq!(actual, expected);
}

/// The `status_schema.avdl` file defines a standalone schema (an enum with a
/// default). Its expected output is `status.avsc`, which is a JSON array
/// containing a single schema.
///
/// This file uses bare named type declarations without a `schema` keyword,
/// so the parser returns `IdlFile::NamedSchemasFile` and the output is a
/// JSON array of all named schemas.
#[test]
fn test_status_schema() {
    let actual = parse_and_serialize(&input_path("status_schema.avdl"), &[]);
    let expected = load_expected(&output_path("status.avsc"));
    assert_eq!(actual, expected);
}
