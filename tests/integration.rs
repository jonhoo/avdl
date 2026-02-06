// ==============================================================================
// Integration Tests: Parse .avdl Files and Compare Against Expected JSON Output
// ==============================================================================
//
// Each test reads an `.avdl` input file from the Avro test suite, parses it
// through our IDL reader, serializes the result to JSON, and compares it
// semantically against the expected `.avpr` or `.avsc` output file.
//
// JSON comparison approach: We compare `serde_json::Value` trees via
// `assert_eq!`. With the `preserve_order` feature enabled on `serde_json`,
// `Value::Object` uses `IndexMap` internally, so key ordering is preserved
// during both deserialization and comparison. This means our comparisons are
// sensitive to key order, which is the desired behavior since the Avro spec
// defines a canonical key order for schema JSON. If `preserve_order` were
// disabled, `Value` would use `BTreeMap` and sort keys alphabetically, making
// comparisons order-insensitive -- still correct for semantic equality, but
// unable to detect key-ordering regressions.

use std::fs;
use std::path::{Path, PathBuf};

use avdl::import::{import_protocol, import_schema, ImportContext};
use avdl::model::json::{build_lookup, protocol_to_json, schema_to_json};
use avdl::reader::{parse_idl, DeclItem, IdlFile, ImportKind};
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

    let (idl_file, decl_items) =
        parse_idl(&input).unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    // Process declaration items in source order to build a correctly ordered
    // registry. This resolves non-IDL imports and registers local types.
    let mut registry = avdl::resolve::SchemaRegistry::new();
    let current_dir = avdl_path
        .parent()
        .expect("avdl_path should have a parent directory");
    let search_dirs: Vec<PathBuf> = import_dirs.iter().map(|p| p.to_path_buf()).collect();
    let import_ctx = ImportContext::new(search_dirs);

    for item in &decl_items {
        match item {
            DeclItem::Import(import) => {
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
                        let _messages = import_protocol(&resolved, &mut registry)
                            .unwrap_or_else(|e| {
                                panic!("failed to import protocol {}: {e}", resolved.display())
                            });
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
            DeclItem::Type(schema) => {
                let _ = registry.register(schema.clone());
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
/// messages into the current protocol. Declaration items are processed in
/// source order to preserve correct type ordering.
fn parse_and_serialize_with_idl_imports(avdl_path: &Path, import_dirs: &[&Path]) -> Value {
    let input = fs::read_to_string(avdl_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", avdl_path.display()));

    let (idl_file, decl_items) =
        parse_idl(&input).unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    let current_dir = avdl_path
        .parent()
        .expect("avdl_path should have a parent directory");
    let search_dirs: Vec<PathBuf> = import_dirs.iter().map(|p| p.to_path_buf()).collect();
    let mut import_ctx = ImportContext::new(search_dirs);
    let mut registry = avdl::resolve::SchemaRegistry::new();

    // Mark the current file as imported to prevent cycles.
    let canonical = avdl_path
        .canonicalize()
        .unwrap_or_else(|e| panic!("failed to canonicalize {}: {e}", avdl_path.display()));
    import_ctx.mark_imported(&canonical);

    // Accumulate messages from IDL imports.
    let mut imported_messages = indexmap::IndexMap::new();

    // Process declaration items in source order, recursively resolving imports.
    process_decl_items_test(
        &decl_items,
        &mut registry,
        &mut import_ctx,
        current_dir,
        &mut imported_messages,
    );

    // For protocol files, prepend imported messages before dispatching.
    let idl_file = match idl_file {
        IdlFile::ProtocolFile(mut protocol) => {
            protocol.types = registry.schemas().cloned().collect();
            let own_messages = protocol.messages;
            protocol.messages = imported_messages;
            protocol.messages.extend(own_messages);
            IdlFile::ProtocolFile(protocol)
        }
        other => other,
    };

    idl_file_to_json(idl_file, registry)
}

/// Process declaration items in source order for integration tests, mirroring
/// the logic in `main.rs::process_decl_items`.
fn process_decl_items_test(
    decl_items: &[DeclItem],
    registry: &mut avdl::resolve::SchemaRegistry,
    import_ctx: &mut ImportContext,
    current_dir: &Path,
    messages: &mut indexmap::IndexMap<String, avdl::model::protocol::Message>,
) {
    for item in decl_items {
        match item {
            DeclItem::Import(import) => {
                let resolved = import_ctx
                    .resolve_import(&import.path, current_dir)
                    .unwrap_or_else(|e| {
                        panic!(
                            "failed to resolve import '{}': {e}",
                            import.path,
                        )
                    });

                // Skip already-imported files (cycle prevention).
                if import_ctx.mark_imported(&resolved) {
                    continue;
                }

                let import_dir = resolved
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("."));

                match import.kind {
                    ImportKind::Schema => {
                        import_schema(&resolved, registry).unwrap_or_else(|e| {
                            panic!("failed to import schema {}: {e}", resolved.display())
                        });
                    }
                    ImportKind::Protocol => {
                        let imported_messages = import_protocol(&resolved, registry).unwrap_or_else(|e| {
                            panic!("failed to import protocol {}: {e}", resolved.display())
                        });
                        messages.extend(imported_messages);
                    }
                    ImportKind::Idl => {
                        // Recursively parse the imported IDL file.
                        let imported_input = fs::read_to_string(&resolved).unwrap_or_else(|e| {
                            panic!("failed to read imported IDL {}: {e}", resolved.display())
                        });
                        let (imported_idl, nested_decl_items) =
                            parse_idl(&imported_input).unwrap_or_else(|e| {
                                panic!("failed to parse imported IDL {}: {e}", resolved.display())
                            });

                        // If the imported file is a protocol, merge its messages.
                        if let IdlFile::ProtocolFile(imported_protocol) = &imported_idl {
                            messages.extend(imported_protocol.messages.clone());
                        }

                        // Recursively process nested declaration items.
                        process_decl_items_test(
                            &nested_decl_items,
                            registry,
                            import_ctx,
                            &import_dir,
                            messages,
                        );
                    }
                }
            }
            DeclItem::Type(schema) => {
                let _ = registry.register(schema.clone());
            }
        }
    }
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

// ==============================================================================
// `idl2schemata` Tests
// ==============================================================================
//
// These tests exercise the `idl2schemata` pipeline, which extracts individual
// named schemas from a protocol and serializes each as a standalone `.avsc`
// JSON object. This mirrors the `avro-tools idl2schemata` subcommand.

/// Parse an `.avdl` file through the `idl2schemata` pipeline: parse the
/// protocol, collect named schemas from the registry, and serialize each one
/// as a standalone JSON value.
///
/// Returns a map of `SimpleName -> serde_json::Value` for each named schema.
fn parse_idl2schemata(
    avdl_path: &Path,
    import_dirs: &[&Path],
) -> indexmap::IndexMap<String, Value> {
    let input = fs::read_to_string(avdl_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", avdl_path.display()));

    let (idl_file, decl_items) =
        parse_idl(&input).unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    let mut registry = avdl::resolve::SchemaRegistry::new();
    let current_dir = avdl_path
        .parent()
        .expect("avdl_path should have a parent directory");
    let search_dirs: Vec<PathBuf> = import_dirs.iter().map(|p| p.to_path_buf()).collect();
    let import_ctx = ImportContext::new(search_dirs);

    for item in &decl_items {
        match item {
            DeclItem::Import(import) => {
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
                        let _messages = import_protocol(&resolved, &mut registry)
                            .unwrap_or_else(|e| {
                                panic!("failed to import protocol {}: {e}", resolved.display())
                            });
                    }
                    ImportKind::Idl => {
                        panic!(
                            "IDL imports not supported in parse_idl2schemata; '{}'",
                            import.path
                        );
                    }
                }
            }
            DeclItem::Type(schema) => {
                let _ = registry.register(schema.clone());
            }
        }
    }

    // Extract the protocol namespace for reference shortening.
    let namespace = match &idl_file {
        IdlFile::ProtocolFile(protocol) => protocol.namespace.clone(),
        _ => None,
    };

    // Build a lookup table and serialize each named schema individually,
    // sharing `known_names` across iterations to match the behavior of
    // `run_idl2schemata` in main.rs.
    let registry_schemas: Vec<_> = registry.schemas().cloned().collect();
    let all_lookup = build_lookup(&registry_schemas, None);
    let mut known_names = IndexSet::new();
    let mut result = indexmap::IndexMap::new();

    for schema in &registry_schemas {
        if let Some(simple_name) = schema.name() {
            let json_value =
                schema_to_json(schema, &mut known_names, namespace.as_deref(), &all_lookup);
            result.insert(simple_name.to_string(), json_value);
        }
    }

    result
}

/// Test `idl2schemata` for `echo.avdl`.
///
/// The Echo protocol has two record types (`Ping` and `Pong`). The
/// `idl2schemata` path should produce one `.avsc` file for each.
#[test]
fn test_idl2schemata_echo() {
    let schemata = parse_idl2schemata(&input_path("echo.avdl"), &[]);

    // echo.avdl defines two records: Ping and Pong.
    assert_eq!(
        schemata.keys().collect::<Vec<_>>(),
        vec!["Ping", "Pong"],
        "expected Ping and Pong schemas from echo.avdl"
    );

    // Verify Ping schema structure.
    let ping = &schemata["Ping"];
    assert_eq!(ping["type"], "record");
    assert_eq!(ping["name"], "Ping");
    let ping_fields = ping["fields"].as_array().expect("Ping should have fields");
    assert_eq!(ping_fields.len(), 2);
    assert_eq!(ping_fields[0]["name"], "timestamp");
    assert_eq!(ping_fields[1]["name"], "text");

    // Verify Pong schema structure. Since Ping was already serialized, the
    // `ping` field's type should be the bare name "Ping", not the full record.
    let pong = &schemata["Pong"];
    assert_eq!(pong["type"], "record");
    assert_eq!(pong["name"], "Pong");
    let pong_fields = pong["fields"].as_array().expect("Pong should have fields");
    let ping_field = &pong_fields[1];
    assert_eq!(ping_field["name"], "ping");
    assert_eq!(
        ping_field["type"], "Ping",
        "second occurrence of Ping should be a bare name reference"
    );
}

/// Test `idl2schemata` for `simple.avdl`.
///
/// The Simple protocol has several named types: Kind (enum), Status (enum),
/// TestRecord (record), MD5 (fixed), and TestError (error record). Verifies
/// correct file names and that each schema serializes to the expected JSON.
#[test]
fn test_idl2schemata_simple() {
    let schemata = parse_idl2schemata(&input_path("simple.avdl"), &[]);

    let names: Vec<&String> = schemata.keys().collect();
    assert_eq!(
        names,
        vec!["Kind", "Status", "TestRecord", "MD5", "TestError"],
        "expected five named schemas from simple.avdl in declaration order"
    );

    // Kind: enum with three symbols and an alias.
    let kind = &schemata["Kind"];
    assert_eq!(kind["type"], "enum");
    assert_eq!(kind["symbols"].as_array().unwrap().len(), 3);
    assert_eq!(kind["aliases"], serde_json::json!(["org.foo.KindOf"]));

    // Status: enum with default.
    let status = &schemata["Status"];
    assert_eq!(status["type"], "enum");
    assert_eq!(status["default"], "C");

    // TestRecord: record with fields, doc, and custom property.
    let test_record = &schemata["TestRecord"];
    assert_eq!(test_record["type"], "record");
    assert_eq!(test_record["doc"], "A TestRecord.");
    assert!(test_record.get("my-property").is_some());
    let fields = test_record["fields"].as_array().expect("TestRecord should have fields");
    assert!(fields.len() > 5, "TestRecord should have many fields");

    // MD5: fixed type with size 16. Since it was already inlined inside
    // TestRecord's `hash` field, the standalone serialization should be a
    // bare string reference.
    let md5 = &schemata["MD5"];
    assert_eq!(
        *md5,
        serde_json::Value::String("MD5".to_string()),
        "MD5 was already inlined in TestRecord, so its standalone entry should be a bare name"
    );

    // TestError: error record.
    let test_error = &schemata["TestError"];
    assert_eq!(test_error["type"], "error");
    assert_eq!(test_error["name"], "TestError");
}

// ==============================================================================
// Negative / Error-Case Tests
// ==============================================================================
//
// These tests verify that the parser correctly rejects invalid input.

/// Duplicate type definitions should produce an error when registering with
/// the schema registry. The parser itself accepts the syntax, but the registry
/// enforces uniqueness.
#[test]
fn test_duplicate_type_definition() {
    let input = r#"
        @namespace("org.test")
        protocol DupTest {
            record Dup { string name; }
            record Dup { int id; }
        }
    "#;

    let result = parse_idl(input);
    match result {
        Ok((IdlFile::ProtocolFile(_), decl_items)) => {
            // The parser may accept duplicate names, but registering them
            // in the SchemaRegistry should fail.
            let mut registry = avdl::resolve::SchemaRegistry::new();
            let mut saw_error = false;
            for item in &decl_items {
                if let DeclItem::Type(schema) = item {
                    if registry.register(schema.clone()).is_err() {
                        saw_error = true;
                    }
                }
            }
            assert!(
                saw_error,
                "registering duplicate type names should produce an error"
            );
        }
        Err(_) => {
            // If the parser itself rejects it, that's also acceptable.
        }
        Ok(_) => {
            panic!("expected a protocol file or parse error for duplicate type input");
        }
    }
}

/// Importing a nonexistent file should produce an error during import resolution.
#[test]
fn test_import_nonexistent_file() {
    let input = r#"
        @namespace("org.test")
        protocol ImportTest {
            import schema "does_not_exist.avsc";
            record Rec { string name; }
        }
    "#;

    let (_, decl_items) =
        parse_idl(input).expect("parsing the IDL text itself should succeed");

    let mut registry = avdl::resolve::SchemaRegistry::new();
    let import_ctx = ImportContext::new(vec![]);

    // Try to resolve the import -- it should fail because the file doesn't exist.
    let mut saw_resolve_error = false;
    for item in &decl_items {
        if let DeclItem::Import(import) = item {
            let result = import_ctx.resolve_import(&import.path, Path::new("."));
            if result.is_err() {
                saw_resolve_error = true;
            } else {
                // The path resolved, but the file shouldn't exist on disk.
                let resolved = result.unwrap();
                if import_schema(&resolved, &mut registry).is_err() {
                    saw_resolve_error = true;
                }
            }
        }
    }
    assert!(
        saw_resolve_error,
        "importing a nonexistent file should produce an error"
    );
}

/// Nested unions should be rejected during parsing. The Avro specification
/// states: "Unions may not immediately contain other unions."
#[test]
fn test_nested_union_rejected() {
    let input = r#"
        @namespace("org.test")
        protocol NestedUnionTest {
            record Bad {
                union { null, union { string, int } } nested;
            }
        }
    "#;

    let result = parse_idl(input);
    assert!(
        result.is_err(),
        "nested unions should be rejected by the parser"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Unions may not immediately contain other unions"),
        "error message should mention nested unions, got: {err_msg}"
    );
}

/// Type names that collide with Avro built-in types (e.g., `int`, `string`,
/// `null`) must be rejected. The Java implementation enforces this via
/// `INVALID_TYPE_NAMES` in `IdlReader.java`.
#[test]
fn test_reserved_type_name_rejected() {
    let input = r#"record `int` { string value; }"#;
    let result = parse_idl(input);
    assert!(
        result.is_err(),
        "expected error for record named `int` (reserved type name)"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("Illegal name"),
        "error should mention 'Illegal name', got: {err_msg}"
    );
}

// ==============================================================================
// Extra Directory Tests
// ==============================================================================
//
// The `extra/` directory contains test inputs that the Java TestIdlReader tests
// against but that are not in the standard `input/` directory.

const EXTRA_DIR: &str = "avro/lang/java/idl/src/test/idl/extra";

/// Test `extra/protocolSyntax.avdl`: a minimal protocol with one record type.
///
/// Verifies that the parser returns a `ProtocolFile` with:
/// - Protocol name "Parrot" in namespace "communication"
/// - One named type: the record `Message`
/// - One message: `echo`
#[test]
fn test_extra_protocol_syntax() {
    let avdl_path = PathBuf::from(EXTRA_DIR).join("protocolSyntax.avdl");
    let input = fs::read_to_string(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", avdl_path.display()));

    let (idl_file, decl_items) =
        parse_idl(&input).unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    // Verify it's a protocol file.
    let protocol = match idl_file {
        IdlFile::ProtocolFile(p) => p,
        other => panic!("expected ProtocolFile, got {:?}", std::mem::discriminant(&other)),
    };

    assert_eq!(protocol.name, "Parrot");
    assert_eq!(protocol.namespace.as_deref(), Some("communication"));

    // Verify one named type: the Message record.
    let type_items: Vec<_> = decl_items
        .iter()
        .filter_map(|item| match item {
            DeclItem::Type(schema) => Some(schema),
            _ => None,
        })
        .collect();
    assert_eq!(type_items.len(), 1, "protocolSyntax.avdl should define exactly one named type");
    assert_eq!(
        type_items[0].full_name(),
        Some("communication.Message".to_string()),
        "the named type should be communication.Message"
    );

    // Verify one message: echo.
    assert_eq!(protocol.messages.len(), 1);
    assert!(
        protocol.messages.contains_key("echo"),
        "protocol should have an 'echo' message"
    );
}

/// Test `extra/schemaSyntax.avdl`: a schema-mode file with `schema array<Message>;`.
///
/// Verifies that the parser returns a `SchemaFile` with:
/// - Main schema is an array type whose items are the Message record
/// - One named type in the declaration items: the `Message` record
#[test]
fn test_extra_schema_syntax() {
    let avdl_path = PathBuf::from(EXTRA_DIR).join("schemaSyntax.avdl");
    let input = fs::read_to_string(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", avdl_path.display()));

    let (idl_file, decl_items) =
        parse_idl(&input).unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    // Verify it's a schema file (not a protocol).
    let schema = match idl_file {
        IdlFile::SchemaFile(s) => s,
        other => panic!("expected SchemaFile, got {:?}", std::mem::discriminant(&other)),
    };

    // The main schema should be an array type.
    match &schema {
        avdl::model::schema::AvroSchema::Array { items, .. } => {
            // The items should reference the Message record. It might be
            // a Reference or an inline Record depending on parse order.
            match items.as_ref() {
                avdl::model::schema::AvroSchema::Reference { name, namespace, .. } => {
                    assert_eq!(name, "Message");
                    // Namespace could be Some("communication") or resolved later.
                    if let Some(ns) = namespace {
                        assert_eq!(ns, "communication");
                    }
                }
                avdl::model::schema::AvroSchema::Record { name, .. } => {
                    assert_eq!(name, "Message");
                }
                other => panic!(
                    "expected array items to be a Reference or Record for Message, got {:?}",
                    std::mem::discriminant(other)
                ),
            }
        }
        other => panic!(
            "expected Array schema, got {:?}",
            std::mem::discriminant(other)
        ),
    }

    // Verify the declaration items contain the Message record.
    let type_items: Vec<_> = decl_items
        .iter()
        .filter_map(|item| match item {
            DeclItem::Type(schema) => Some(schema),
            _ => None,
        })
        .collect();
    assert_eq!(
        type_items.len(),
        1,
        "schemaSyntax.avdl should define exactly one named type"
    );
    assert_eq!(
        type_items[0].full_name(),
        Some("communication.Message".to_string()),
        "the named type should be communication.Message"
    );
}
