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

    let (idl_file, decl_items, _warnings) =
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

    let (idl_file, decl_items, _warnings) =
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
                        let (imported_idl, nested_decl_items, _import_warnings) =
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

    let (idl_file, decl_items, _warnings) =
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

    // Protocol namespace is intentionally not used for idl2schemata output:
    // each schema is standalone with no enclosing namespace context,
    // matching Java's `Schema.toString(true)`.
    let _namespace = match &idl_file {
        IdlFile::ProtocolFile(protocol) => protocol.namespace.clone(),
        _ => None,
    };

    // Build a lookup table and serialize each named schema individually.
    // Each schema gets its own fresh `known_names` set and no enclosing
    // namespace context, matching `run_idl2schemata` in main.rs and Java's
    // `Schema.toString(true)` which creates a fresh `HashSet` per call.
    // This ensures each schema is self-contained with all referenced types
    // inlined on first occurrence.
    let registry_schemas: Vec<_> = registry.schemas().cloned().collect();
    let all_lookup = build_lookup(&registry_schemas, None);
    let mut result = indexmap::IndexMap::new();

    for schema in &registry_schemas {
        if let Some(simple_name) = schema.name() {
            let mut known_names = IndexSet::new();
            let json_value = schema_to_json(schema, &mut known_names, None, &all_lookup);
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

    // Verify Pong schema structure. Each schema is serialized independently
    // with fresh known_names, so Ping is inlined as a full record definition
    // inside Pong (making each .avsc self-contained).
    let pong = &schemata["Pong"];
    assert_eq!(pong["type"], "record");
    assert_eq!(pong["name"], "Pong");
    let pong_fields = pong["fields"].as_array().expect("Pong should have fields");
    let ping_field = &pong_fields[1];
    assert_eq!(ping_field["name"], "ping");
    let ping_type = &ping_field["type"];
    assert_eq!(
        ping_type["type"], "record",
        "Ping should be inlined as a full record in Pong's standalone schema"
    );
    assert_eq!(ping_type["name"], "Ping");
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

    // MD5: fixed type with size 16. Each schema is serialized independently
    // with fresh known_names, so MD5 is a full inline definition even though
    // it also appears inside TestRecord.
    let md5 = &schemata["MD5"];
    assert_eq!(md5["type"], "fixed");
    assert_eq!(md5["name"], "MD5");
    assert_eq!(md5["size"], 16);

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
        Ok((IdlFile::ProtocolFile(_), decl_items, _warnings)) => {
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

    let (_, decl_items, _warnings) =
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

/// Records must not contain duplicate field names. The Java Schema constructor
/// rejects duplicates with "Duplicate field X in record Y".
#[test]
fn test_duplicate_field_name_rejected() {
    let input = r#"
        @namespace("test")
        protocol P {
            record R {
                string name;
                int name;
            }
        }
    "#;

    let result = parse_idl(input);
    assert!(
        result.is_err(),
        "duplicate field names should be rejected"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("duplicate field 'name'"),
        "error should mention duplicate field, got: {err_msg}"
    );
}

/// Enum declarations must not contain duplicate symbols. The Java Schema
/// constructor rejects duplicates with "Duplicate enum symbol: X".
#[test]
fn test_duplicate_enum_symbol_rejected() {
    let input = r#"
        @namespace("test")
        protocol P {
            enum Color { RED, GREEN, BLUE, RED }
        }
    "#;

    let result = parse_idl(input);
    assert!(
        result.is_err(),
        "duplicate enum symbols should be rejected"
    );
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("duplicate enum symbol: RED"),
        "error should mention duplicate symbol, got: {err_msg}"
    );
}

/// When a named type's identifier contains dots (e.g., `com.example.Foo`),
/// the dot-derived namespace takes priority over an explicit `@namespace`
/// annotation, matching Java's `IdlReader.namespace()` behavior.
#[test]
fn test_dotted_identifier_namespace_priority() {
    let input = r#"
        @namespace("foo")
        protocol P {
            @namespace("bar") record com.example.MyRecord {
                int x;
            }
        }
    "#;

    let (idl_file, decl_items, _warnings) = parse_idl(input).expect("should parse successfully");
    let mut registry = avdl::resolve::SchemaRegistry::new();
    for item in &decl_items {
        if let DeclItem::Type(schema) = item {
            let _ = registry.register(schema.clone());
        }
    }
    let json = idl_file_to_json(idl_file, registry);
    let types = json.get("types").expect("missing types");
    let record = &types[0];
    assert_eq!(
        record.get("namespace").and_then(|v| v.as_str()),
        Some("com.example"),
        "dots in identifier should take priority over @namespace annotation"
    );
    assert_eq!(
        record.get("name").and_then(|v| v.as_str()),
        Some("MyRecord"),
        "name should be extracted after the last dot"
    );
}

// ==============================================================================
// Namespace Correctness Tests
// ==============================================================================

/// `@namespace("")` should produce `"namespace": ""` in JSON output, explicitly
/// opting the type out of the enclosing protocol namespace. Previously, empty
/// namespace annotations were collapsed to `None` and the type silently
/// inherited the protocol namespace.
#[test]
fn test_empty_namespace_annotation_emits_namespace_key() {
    let input = r#"
        @namespace("org.example")
        protocol P {
            @namespace("")
            record NoNamespace { string name; }
        }
    "#;

    let (idl_file, decl_items, _warnings) = parse_idl(input).expect("should parse successfully");
    let mut registry = avdl::resolve::SchemaRegistry::new();
    for item in &decl_items {
        if let DeclItem::Type(schema) = item {
            let _ = registry.register(schema.clone());
        }
    }

    // The type should be registered under its bare name, not "org.example.NoNamespace".
    assert!(
        registry.contains("NoNamespace"),
        "type with @namespace(\"\") should be registered under bare name"
    );
    assert!(
        !registry.contains("org.example.NoNamespace"),
        "type with @namespace(\"\") must not inherit the protocol namespace"
    );

    let json = idl_file_to_json(idl_file, registry);
    let types = json.get("types").and_then(|t| t.as_array()).expect("missing types");
    assert_eq!(types.len(), 1);
    let record = &types[0];

    // The JSON should contain "namespace": "" to explicitly indicate no namespace.
    assert_eq!(
        record.get("namespace").and_then(|v| v.as_str()),
        Some(""),
        "@namespace(\"\") should emit \"namespace\": \"\" in JSON output"
    );
    assert_eq!(
        record.get("name").and_then(|v| v.as_str()),
        Some("NoNamespace"),
    );
}

/// When a record has a different namespace from the protocol and contains
/// inline named types in its fields, `build_lookup` should register those
/// nested types under the record's effective namespace, not the protocol's
/// default namespace.
///
/// Avro IDL does not support inline named type definitions in field
/// declarations, so this test constructs the schema tree directly to exercise
/// the `collect_named_types` code path.
#[test]
fn test_nested_types_inherit_record_namespace_in_lookup() {
    use avdl::model::schema::{AvroSchema, Field};
    use indexmap::IndexMap;

    let inner_enum = AvroSchema::Enum {
        name: "InnerEnum".to_string(),
        namespace: None, // no explicit namespace -- should inherit from Outer
        doc: None,
        symbols: vec!["A".to_string(), "B".to_string()],
        default: None,
        aliases: vec![],
        properties: IndexMap::new(),
    };

    let outer_record = AvroSchema::Record {
        name: "Outer".to_string(),
        namespace: Some("com.other".to_string()),
        doc: None,
        fields: vec![Field {
            name: "inner".to_string(),
            schema: inner_enum,
            doc: None,
            default: None,
            order: None,
            aliases: vec![],
            properties: IndexMap::new(),
        }],
        is_error: false,
        aliases: vec![],
        properties: IndexMap::new(),
    };

    // Protocol default namespace is "org.example", but Outer overrides to
    // "com.other". InnerEnum has no explicit namespace and should inherit
    // from Outer, producing lookup key "com.other.InnerEnum".
    let lookup = build_lookup(&[outer_record], Some("org.example"));

    assert!(
        lookup.contains_key("com.other.Outer"),
        "Outer should be registered under com.other"
    );
    assert!(
        lookup.contains_key("com.other.InnerEnum"),
        "InnerEnum should inherit the record's namespace (com.other), not the protocol's (org.example)"
    );
    assert!(
        !lookup.contains_key("org.example.InnerEnum"),
        "InnerEnum must not be registered under the protocol namespace"
    );
}

/// Cross-namespace unqualified references should be flagged as unresolved.
/// When a type is in namespace "org.other" and is referenced by short name
/// from namespace "org.example", the reference should NOT resolve because
/// the full name would be "org.example.OtherRecord" (not found) rather than
/// "org.other.OtherRecord" (the actual type).
#[test]
fn test_cross_namespace_unqualified_reference_is_unresolved() {
    let input = r#"
        @namespace("org.example")
        protocol P {
            @namespace("org.other")
            record OtherRecord { string name; }
            record MainRecord { OtherRecord other; }
        }
    "#;

    let (_, decl_items, _warnings) = parse_idl(input).expect("should parse successfully");
    let mut registry = avdl::resolve::SchemaRegistry::new();
    for item in &decl_items {
        if let DeclItem::Type(schema) = item {
            let _ = registry.register(schema.clone());
        }
    }

    // OtherRecord is in org.other, but the unqualified reference from
    // MainRecord resolves as org.example.OtherRecord, which does not exist.
    let unresolved = registry.validate_references();
    assert!(
        unresolved.contains(&"org.example.OtherRecord".to_string()),
        "unqualified cross-namespace reference should be flagged as unresolved, got: {unresolved:?}"
    );
}

// ==============================================================================
// Protocol Tests (with IDL imports)
// ==============================================================================

const CLASSPATH_DIR: &str = "avro/lang/java/idl/src/test/idl/putOnClassPath";

/// Test `import.avdl`: exercises every import kind (IDL, protocol, schema) in a
/// single protocol, including classpath-resolved imports.
///
/// The import chain is deep: `import.avdl` imports `reservedwords.avdl` (IDL),
/// `nestedimport.avdl` (IDL, which itself imports `reservedwords.avdl` and
/// `bar.avpr`), `OnTheClasspath.avdl` (IDL, resolved via classpath, which
/// chains to `folder/relativePath.avdl` and then `nestedtypes.avdl`),
/// `OnTheClasspath.avpr` (protocol), `OnTheClasspath.avsc` (schema),
/// `baz.avsc` (schema), `foo.avsc` (schema), and `bar.avpr` (protocol).
///
/// Both `input/` and `putOnClassPath/` must be passed as import directories.
#[test]
fn test_import() {
    let input_dir = PathBuf::from(INPUT_DIR);
    let classpath_dir = PathBuf::from(CLASSPATH_DIR);
    let actual = parse_and_serialize_with_idl_imports(
        &input_path("import.avdl"),
        &[&input_dir, &classpath_dir],
    );
    let expected = load_expected(&output_path("import.avpr"));
    assert_eq!(actual, expected);
}

/// Test `nestedimport.avdl`: exercises nested import chains.
///
/// This protocol imports `reservedwords.avdl` (IDL), `bar.avpr` (protocol),
/// `position.avsc` and `player.avsc` (schemas). The IDL import pulls in the
/// reserved-word messages from `reservedwords.avdl`, and the schema imports
/// bring in the baseball `Position` and `Player` types.
#[test]
fn test_nestedimport() {
    let input_dir = PathBuf::from(INPUT_DIR);
    let actual = parse_and_serialize_with_idl_imports(
        &input_path("nestedimport.avdl"),
        &[&input_dir],
    );
    let expected = load_expected(&output_path("nestedimport.avpr"));
    assert_eq!(actual, expected);
}

// ==============================================================================
// Tools Test Suite (tools/src/test/idl/)
// ==============================================================================
//
// These tests exercise golden files from the Java `TestIdlTool` and
// `TestIdlToSchemataTool` test suites, which live in a separate directory from
// the main IDL test suite.

const TOOLS_IDL_DIR: &str = "avro/lang/java/tools/src/test/idl";

/// Test `tools/src/test/idl/schema.avdl` in schema mode.
///
/// This exercises a pattern not covered by other schema-mode tests: `schema
/// TestRecord;` where the named type itself contains forward references to
/// types (`Kind`, `MD5`) defined later in the file. The expected output is
/// `schema.avsc`, a single record JSON object with forward-referenced types
/// inlined at first use.
#[test]
fn test_tools_schema() {
    let avdl_path = PathBuf::from(TOOLS_IDL_DIR).join("schema.avdl");
    let actual = parse_and_serialize(&avdl_path, &[]);
    let expected = load_expected(&PathBuf::from(TOOLS_IDL_DIR).join("schema.avsc"));
    assert_eq!(actual, expected);
}

/// Test `tools/src/test/idl/protocol.avdl` in protocol mode.
///
/// Similar to `simple.avdl` but exercises `@aliases(["hash"])` on a nullable
/// field declared as `MD5?` (which produces `["null", "MD5"]` union ordering)
/// rather than the explicit `union { MD5, null }` in `simple.avdl` (which
/// produces `["MD5", "null"]` ordering).
#[test]
fn test_tools_protocol() {
    let avdl_path = PathBuf::from(TOOLS_IDL_DIR).join("protocol.avdl");
    let actual = parse_and_serialize(&avdl_path, &[]);
    let expected = load_expected(&PathBuf::from(TOOLS_IDL_DIR).join("protocol.avpr"));
    assert_eq!(actual, expected);
}

// ==============================================================================
// Compiler Module Tests
// ==============================================================================
//
// Test inputs from the `avro/lang/java/compiler/` module's test directory, which
// exercises edge cases not covered by the IDL module's standard test suite.

const COMPILER_TEST_DIR: &str = "avro/lang/java/compiler/src/test/idl";

/// AVRO-3706: Parse an `.avdl` file from a directory whose path contains spaces.
///
/// This exercises file path resolution with spaces and chained IDL imports:
/// `root.avdl` imports `level1.avdl`, which imports `level2.avdl`, all within
/// the `work space/` directory.
#[test]
fn test_workspace_path() {
    let workspace_dir = PathBuf::from(COMPILER_TEST_DIR).join("work space");
    let avdl_path = workspace_dir.join("root.avdl");
    let expected_path = workspace_dir.join("root.avpr");

    let actual = parse_and_serialize_with_idl_imports(&avdl_path, &[]);
    let expected = load_expected(&expected_path);
    assert_eq!(actual, expected);
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

    let (idl_file, decl_items, _warnings) =
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

    let (idl_file, decl_items, _warnings) =
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

// ==============================================================================
// Logical Type Propagation Tests
// ==============================================================================
//
// These tests verify that logical types -- both built-in keywords and custom
// `@logicalType("...")` annotations -- are correctly propagated through the
// parse -> serialize pipeline and appear in the JSON output.

/// Helper: parse an inline `.avdl` string and serialize to JSON.
///
/// This mirrors `parse_and_serialize` but accepts a string instead of a file
/// path, making it convenient for inline test inputs.
fn parse_inline_to_json(avdl_input: &str) -> Value {
    let (idl_file, decl_items, _warnings) =
        parse_idl(avdl_input).unwrap_or_else(|e| panic!("failed to parse inline avdl: {e}"));

    let mut registry = avdl::resolve::SchemaRegistry::new();
    for item in &decl_items {
        if let DeclItem::Type(schema) = item {
            let _ = registry.register(schema.clone());
        }
    }

    idl_file_to_json(idl_file, registry)
}

/// Verify that all built-in logical type keywords (`date`, `time_ms`,
/// `timestamp_ms`, `local_timestamp_ms`, `uuid`) and `decimal(p, s)` produce
/// the correct `logicalType` key in JSON output.
#[test]
fn test_builtin_logical_types_propagate_to_json() {
    let input = r#"
        @namespace("test")
        protocol LogicalTypes {
            record Event {
                date created_date;
                time_ms created_time;
                timestamp_ms created_timestamp;
                local_timestamp_ms local_created;
                uuid event_id;
                decimal(10, 2) amount;
            }
        }
    "#;

    let json = parse_inline_to_json(input);
    let types = json["types"].as_array().expect("missing types array");
    assert_eq!(types.len(), 1, "expected exactly one record type");

    let fields = types[0]["fields"]
        .as_array()
        .expect("Event record should have fields");
    assert_eq!(fields.len(), 6, "Event should have 6 fields");

    // date -> {"type": "int", "logicalType": "date"}
    let created_date = &fields[0];
    assert_eq!(created_date["name"], "created_date");
    assert_eq!(created_date["type"]["type"], "int");
    assert_eq!(created_date["type"]["logicalType"], "date");

    // time_ms -> {"type": "int", "logicalType": "time-millis"}
    let created_time = &fields[1];
    assert_eq!(created_time["name"], "created_time");
    assert_eq!(created_time["type"]["type"], "int");
    assert_eq!(created_time["type"]["logicalType"], "time-millis");

    // timestamp_ms -> {"type": "long", "logicalType": "timestamp-millis"}
    let created_timestamp = &fields[2];
    assert_eq!(created_timestamp["name"], "created_timestamp");
    assert_eq!(created_timestamp["type"]["type"], "long");
    assert_eq!(created_timestamp["type"]["logicalType"], "timestamp-millis");

    // local_timestamp_ms -> {"type": "long", "logicalType": "local-timestamp-millis"}
    let local_created = &fields[3];
    assert_eq!(local_created["name"], "local_created");
    assert_eq!(local_created["type"]["type"], "long");
    assert_eq!(local_created["type"]["logicalType"], "local-timestamp-millis");

    // uuid -> {"type": "string", "logicalType": "uuid"}
    let event_id = &fields[4];
    assert_eq!(event_id["name"], "event_id");
    assert_eq!(event_id["type"]["type"], "string");
    assert_eq!(event_id["type"]["logicalType"], "uuid");

    // decimal(10, 2) -> {"type": "bytes", "logicalType": "decimal", "precision": 10, "scale": 2}
    let amount = &fields[5];
    assert_eq!(amount["name"], "amount");
    assert_eq!(amount["type"]["type"], "bytes");
    assert_eq!(amount["type"]["logicalType"], "decimal");
    assert_eq!(amount["type"]["precision"], 10);
    assert_eq!(amount["type"]["scale"], 2);
}

/// Verify that custom/user-defined logical types via `@logicalType("...")`
/// annotations are propagated as-is to JSON output. The annotation becomes a
/// property on the base type, producing e.g.
/// `{"type": "long", "logicalType": "timestamp-micros"}`.
#[test]
fn test_custom_logical_type_annotation_propagates_to_json() {
    let input = r#"
        @namespace("test")
        protocol CustomLogical {
            record Measurements {
                @logicalType("timestamp-micros") long precise_time;
                @logicalType("custom-type") bytes payload;
                @logicalType("temperature-celsius") double temp;
            }
        }
    "#;

    let json = parse_inline_to_json(input);
    let types = json["types"].as_array().expect("missing types array");
    let fields = types[0]["fields"]
        .as_array()
        .expect("Measurements record should have fields");
    assert_eq!(fields.len(), 3, "Measurements should have 3 fields");

    // @logicalType("timestamp-micros") long -> {"type": "long", "logicalType": "timestamp-micros"}
    let precise_time = &fields[0];
    assert_eq!(precise_time["name"], "precise_time");
    assert_eq!(precise_time["type"]["type"], "long");
    assert_eq!(precise_time["type"]["logicalType"], "timestamp-micros");

    // @logicalType("custom-type") bytes -> {"type": "bytes", "logicalType": "custom-type"}
    let payload = &fields[1];
    assert_eq!(payload["name"], "payload");
    assert_eq!(payload["type"]["type"], "bytes");
    assert_eq!(payload["type"]["logicalType"], "custom-type");

    // @logicalType("temperature-celsius") double -> {"type": "double", "logicalType": "temperature-celsius"}
    let temp = &fields[2];
    assert_eq!(temp["name"], "temp");
    assert_eq!(temp["type"]["type"], "double");
    assert_eq!(temp["type"]["logicalType"], "temperature-celsius");
}

/// Verify that `@logicalType` combined with other custom annotations preserves
/// all properties in JSON output. This exercises the interaction between
/// `@logicalType` and additional annotations like `@precision`, `@scale`, etc.
#[test]
fn test_custom_logical_type_with_additional_annotations() {
    let input = r#"
        @namespace("test")
        protocol AnnotatedLogical {
            record Payment {
                @logicalType("decimal") @precision(6) @scale(2) bytes allowance;
                @logicalType("fixed-size-string") @minLength(1) @maxLength(50) string bounded;
            }
        }
    "#;

    let json = parse_inline_to_json(input);
    let types = json["types"].as_array().expect("missing types array");
    let fields = types[0]["fields"]
        .as_array()
        .expect("Payment record should have fields");
    assert_eq!(fields.len(), 2, "Payment should have 2 fields");

    // @logicalType("decimal") @precision(6) @scale(2) bytes
    // -> {"type": "bytes", "logicalType": "decimal", "precision": 6, "scale": 2}
    let allowance = &fields[0];
    assert_eq!(allowance["name"], "allowance");
    assert_eq!(allowance["type"]["type"], "bytes");
    assert_eq!(allowance["type"]["logicalType"], "decimal");
    assert_eq!(allowance["type"]["precision"], 6);
    assert_eq!(allowance["type"]["scale"], 2);

    // @logicalType("fixed-size-string") @minLength(1) @maxLength(50) string
    // -> {"type": "string", "logicalType": "fixed-size-string", "minLength": 1, "maxLength": 50}
    let bounded = &fields[1];
    assert_eq!(bounded["name"], "bounded");
    assert_eq!(bounded["type"]["type"], "string");
    assert_eq!(bounded["type"]["logicalType"], "fixed-size-string");
    assert_eq!(bounded["type"]["minLength"], 1);
    assert_eq!(bounded["type"]["maxLength"], 50);
}

/// Verify that built-in logical type keywords can carry additional custom
/// annotations from the `fullType` position. For example, `@version("2") date`
/// should produce `{"type": "int", "logicalType": "date", "version": "2"}`.
///
/// Annotations in the `fullType` position (before the type keyword) become
/// properties on the type's schema object, because `fullType` uses `BARE_PROPS`
/// where no annotation names are intercepted as special.
#[test]
fn test_builtin_logical_type_with_custom_annotation() {
    let input = r#"
        @namespace("test")
        protocol AnnotatedBuiltin {
            record Annotated {
                @version("2") date versioned_date;
                @source("external") timestamp_ms annotated_ts;
            }
        }
    "#;

    let json = parse_inline_to_json(input);
    let types = json["types"].as_array().expect("missing types array");
    let fields = types[0]["fields"]
        .as_array()
        .expect("Annotated record should have fields");

    // @version("2") date -> {"type": "int", "logicalType": "date", "version": "2"}
    let versioned_date = &fields[0];
    assert_eq!(versioned_date["name"], "versioned_date");
    assert_eq!(versioned_date["type"]["type"], "int");
    assert_eq!(versioned_date["type"]["logicalType"], "date");
    assert_eq!(versioned_date["type"]["version"], "2");

    // @source("external") timestamp_ms -> {"type": "long", "logicalType": "timestamp-millis", "source": "external"}
    let annotated_ts = &fields[1];
    assert_eq!(annotated_ts["name"], "annotated_ts");
    assert_eq!(annotated_ts["type"]["type"], "long");
    assert_eq!(annotated_ts["type"]["logicalType"], "timestamp-millis");
    assert_eq!(annotated_ts["type"]["source"], "external");
}

// ==============================================================================
// Additional `idl2schemata` Tests (Gap #2)
// ==============================================================================

/// Test `idl2schemata` for `interop.avdl`.
///
/// The InteropProtocol defines five named types: Foo (record), Kind (enum),
/// MD5 (fixed), Node (record with self-referential `array<Node>` field), and
/// Interop (record referencing all others). Verifies correct handling of
/// self-referential records and that all named types are extracted.
#[test]
fn test_idl2schemata_interop() {
    let schemata = parse_idl2schemata(&input_path("interop.avdl"), &[]);

    let names: Vec<&String> = schemata.keys().collect();
    assert_eq!(
        names,
        vec!["Foo", "Kind", "MD5", "Node", "Interop"],
        "expected five named schemas from interop.avdl in declaration order"
    );

    // Foo: simple record with a single string field.
    let foo = &schemata["Foo"];
    assert_eq!(foo["type"], "record");
    assert_eq!(foo["name"], "Foo");
    let foo_fields = foo["fields"].as_array().expect("Foo should have fields");
    assert_eq!(foo_fields.len(), 1);
    assert_eq!(foo_fields[0]["name"], "label");

    // Kind: enum with three symbols (A, B, C).
    let kind = &schemata["Kind"];
    assert_eq!(kind["type"], "enum");
    let kind_symbols = kind["symbols"].as_array().expect("Kind should have symbols");
    assert_eq!(kind_symbols.len(), 3);

    // MD5: fixed type with size 16.
    let md5 = &schemata["MD5"];
    assert_eq!(md5["type"], "fixed");
    assert_eq!(md5["size"], 16);

    // Node: self-referential record (children field is array<Node>). In
    // idl2schemata mode, Node is serialized standalone with its own fresh
    // known_names, so the self-reference should appear as a string "Node"
    // (not re-inlined).
    let node = &schemata["Node"];
    assert_eq!(node["type"], "record");
    assert_eq!(node["name"], "Node");
    let node_fields = node["fields"].as_array().expect("Node should have fields");
    assert_eq!(node_fields.len(), 2);
    let children_field = &node_fields[1];
    assert_eq!(children_field["name"], "children");
    // The children type is an array whose items reference Node by name.
    let children_type = &children_field["type"];
    assert_eq!(children_type["type"], "array");
    assert_eq!(
        children_type["items"], "Node",
        "self-referential Node should appear as a string reference, not re-inlined"
    );

    // Interop: large record referencing all other types. Each type should
    // be inlined on first occurrence since each schema gets fresh known_names.
    let interop = &schemata["Interop"];
    assert_eq!(interop["type"], "record");
    let interop_fields = interop["fields"]
        .as_array()
        .expect("Interop should have fields");
    assert_eq!(
        interop_fields.len(),
        13,
        "Interop should have 13 fields"
    );
}

/// Parse an `.avdl` file that has IDL imports through the `idl2schemata`
/// pipeline. This is the IDL-import-aware variant of `parse_idl2schemata`.
///
/// Returns a map of `SimpleName -> serde_json::Value` for each named schema,
/// including schemas imported via `import idl`.
fn parse_idl2schemata_with_idl_imports(
    avdl_path: &Path,
    import_dirs: &[&Path],
) -> indexmap::IndexMap<String, Value> {
    let input = fs::read_to_string(avdl_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", avdl_path.display()));

    let (idl_file, decl_items, _warnings) =
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

    // Messages are not needed for idl2schemata, but process_decl_items_test
    // requires a messages accumulator.
    let mut messages = indexmap::IndexMap::new();

    process_decl_items_test(
        &decl_items,
        &mut registry,
        &mut import_ctx,
        current_dir,
        &mut messages,
    );

    // Build a lookup table and serialize each named schema individually.
    let _namespace = match &idl_file {
        IdlFile::ProtocolFile(protocol) => protocol.namespace.clone(),
        _ => None,
    };

    let registry_schemas: Vec<_> = registry.schemas().cloned().collect();
    let all_lookup = build_lookup(&registry_schemas, None);
    let mut result = indexmap::IndexMap::new();

    for schema in &registry_schemas {
        if let Some(simple_name) = schema.name() {
            let mut known_names = IndexSet::new();
            let json_value = schema_to_json(schema, &mut known_names, None, &all_lookup);
            result.insert(simple_name.to_string(), json_value);
        }
    }

    result
}

/// Test `idl2schemata` for `import.avdl`.
///
/// The Import protocol has IDL imports that pull in types from
/// `reservedwords.avdl`, `nestedimport.avdl`, `OnTheClasspath.avdl`,
/// plus schema and protocol imports. Verifies that imported types
/// are included in the idl2schemata output alongside locally-defined types.
#[test]
fn test_idl2schemata_import() {
    let input_dir = PathBuf::from(INPUT_DIR);
    let classpath_dir = PathBuf::from(CLASSPATH_DIR);
    let schemata = parse_idl2schemata_with_idl_imports(
        &input_path("import.avdl"),
        &[&input_dir, &classpath_dir],
    );

    // The import protocol has many named types from various imports.
    // At minimum, verify that the locally-defined Bar type is present,
    // along with several imported types.
    assert!(
        schemata.contains_key("Bar"),
        "locally-defined Bar should be present in idl2schemata output"
    );

    // Types imported via `import schema "baz.avsc"` and `import schema "foo.avsc"`.
    assert!(
        schemata.contains_key("Baz"),
        "Baz (imported via schema import) should be present"
    );
    assert!(
        schemata.contains_key("Foo"),
        "Foo (imported via schema import) should be present"
    );

    // Types from `import idl "OnTheClasspath.avdl"` chain.
    assert!(
        schemata.contains_key("NestedType"),
        "NestedType (from classpath IDL import chain) should be present"
    );

    // Verify the total count matches what the protocol produces. The
    // import.avpr has 10 named types in its types array.
    assert_eq!(
        schemata.len(),
        10,
        "import.avdl idl2schemata should produce 10 named schemas, got: {:?}",
        schemata.keys().collect::<Vec<_>>()
    );
}

// ==============================================================================
// Import Cycle Detection Tests (Gap #4)
// ==============================================================================

/// A file that imports itself should be handled gracefully by the cycle
/// prevention logic in `ImportContext`. The self-import should be silently
/// skipped (since the file is already marked as imported), producing a
/// valid protocol with just the locally-defined types.
#[test]
fn test_self_import_cycle_handled_gracefully() {
    let avdl_path = PathBuf::from("tests/testdata/self_import.avdl");
    let testdata_dir = PathBuf::from("tests/testdata");

    // Use the full IDL import pipeline. The cycle prevention logic should
    // skip re-importing self_import.avdl when it encounters the self-import.
    let actual = parse_and_serialize_with_idl_imports(&avdl_path, &[&testdata_dir]);

    // The protocol should parse successfully with the locally-defined Rec type.
    assert_eq!(actual["protocol"], "SelfImport");
    let types = actual["types"].as_array().expect("should have types array");
    assert_eq!(types.len(), 1, "should have one type (Rec)");
    assert_eq!(types[0]["name"], "Rec");
}

/// Two files that import each other (A imports B, B imports A) should be
/// handled gracefully. The cycle prevention logic should skip the circular
/// import on the second visit, producing a valid protocol with types from
/// both files.
#[test]
fn test_mutual_import_cycle_handled_gracefully() {
    let avdl_path = PathBuf::from("tests/testdata/cycle_a.avdl");
    let testdata_dir = PathBuf::from("tests/testdata");

    // Parse cycle_a.avdl, which imports cycle_b.avdl, which tries to import
    // cycle_a.avdl again. The second import of cycle_a.avdl should be skipped.
    let actual = parse_and_serialize_with_idl_imports(&avdl_path, &[&testdata_dir]);

    // The protocol should parse successfully. cycle_a defines RecA, and
    // cycle_b defines RecB. Both should appear in the merged types.
    assert_eq!(actual["protocol"], "CycleA");
    let types = actual["types"].as_array().expect("should have types array");

    let type_names: Vec<&str> = types
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();

    assert!(
        type_names.contains(&"RecB"),
        "RecB from cycle_b.avdl should be included via the import, got: {type_names:?}"
    );
    assert!(
        type_names.contains(&"RecA"),
        "RecA from cycle_a.avdl should be included, got: {type_names:?}"
    );
}

// ==============================================================================
// Doc Comment Warning Tests (Gap #5)
// ==============================================================================

/// Verify that parsing `comments.avdl` produces exactly 24 out-of-place
/// documentation comment warnings, matching Java's
/// `testDocCommentsAndWarnings` assertion count.
///
/// The `comments.avdl` file has many intentionally misplaced doc comments
/// (e.g., between keyword and identifier, between enum symbols, etc.) that
/// should each produce a warning.
#[test]
fn test_comments_warnings_count() {
    let avdl_path = input_path("comments.avdl");
    let input = fs::read_to_string(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", avdl_path.display()));

    let (_idl_file, _decl_items, warnings) =
        parse_idl(&input).unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    assert_eq!(
        warnings.len(),
        24,
        "comments.avdl should produce exactly 24 out-of-place doc comment warnings \
         (matching Java's testDocCommentsAndWarnings), got {}:\n{}",
        warnings.len(),
        warnings
            .iter()
            .enumerate()
            .map(|(i, w)| format!("  {}: {}", i + 1, w))
            .collect::<Vec<_>>()
            .join("\n")
    );

    // Each warning should mention "out-of-place documentation comment".
    for (i, warning) in warnings.iter().enumerate() {
        assert!(
            warning.message.contains("Ignoring out-of-place documentation comment"),
            "warning {} should mention out-of-place doc comment, got: {}",
            i + 1,
            warning
        );
    }
}

// ==============================================================================
// Java Test Behavior: `idl2schemata` File Count (Gap #9)
// ==============================================================================

/// Verify that `idl2schemata` on `tools/protocol.avdl` produces exactly 4
/// named schemas: Kind (enum), MD5 (fixed), TestRecord (record), and
/// TestError (error record).
///
/// This mirrors Java's `TestIdlToSchemataTool.splitIdlIntoSchemata` assertion
/// that the tool produces exactly 4 `.avsc` files.
#[test]
fn test_idl2schemata_tools_protocol() {
    let avdl_path = PathBuf::from(TOOLS_IDL_DIR).join("protocol.avdl");
    let schemata = parse_idl2schemata(&avdl_path, &[]);

    let names: Vec<&String> = schemata.keys().collect();
    assert_eq!(
        names,
        vec!["Kind", "MD5", "TestRecord", "TestError"],
        "tools/protocol.avdl should produce exactly 4 named schemas"
    );

    // Kind: enum with 3 symbols.
    let kind = &schemata["Kind"];
    assert_eq!(kind["type"], "enum");
    assert_eq!(kind["symbols"].as_array().unwrap().len(), 3);

    // MD5: fixed with size 16.
    let md5 = &schemata["MD5"];
    assert_eq!(md5["type"], "fixed");
    assert_eq!(md5["size"], 16);

    // TestRecord: record type.
    let test_record = &schemata["TestRecord"];
    assert_eq!(test_record["type"], "record");
    assert_eq!(test_record["name"], "TestRecord");

    // TestError: error record.
    let test_error = &schemata["TestError"];
    assert_eq!(test_error["type"], "error");
    assert_eq!(test_error["name"], "TestError");
}
