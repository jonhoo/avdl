// ==============================================================================
// Integration Tests: Parse .avdl Files and Compare Against Expected JSON Output
// ==============================================================================
//
// Each test reads an `.avdl` input file from the Avro test suite, parses it
// through the library's builder API, serializes the result to JSON, and compares
// it semantically against the expected `.avpr` or `.avsc` output file.
//
// JSON comparison approach: We compare `serde_json::Value` trees via
// `assert_eq!`. Our JSON serialization sorts object keys alphabetically,
// so `Value` comparison is order-insensitive and tests semantic equality.

mod common;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use avdl::{Idl, Idl2Schemata};
use common::{normalize_crlf, render_diagnostic, render_diagnostics};
use pretty_assertions::assert_eq;
use serde_json::Value;

const INPUT_DIR: &str = "avro/lang/java/idl/src/test/idl/input";
const OUTPUT_DIR: &str = "avro/lang/java/idl/src/test/idl/output";

// ==============================================================================
// Test Infrastructure
// ==============================================================================

/// Parse an `.avdl` file through the `Idl` builder and return the JSON output.
/// Applies CRLF normalization so tests pass on Windows where Git checks out
/// files with `\r\n` line endings.
fn parse_and_serialize(avdl_path: &Path, import_dirs: &[&Path]) -> Value {
    let mut builder = Idl::new();
    for dir in import_dirs {
        builder.import_dir(dir);
    }
    let output = builder
        .convert(avdl_path)
        .unwrap_or_else(|e| panic!("failed to compile {}: {e}", avdl_path.display()));
    normalize_crlf(output.json)
}

/// Parse an `.avdl` file through the `Idl2Schemata` builder and return a map
/// of `SimpleName -> serde_json::Value` for each named schema.
fn parse_idl2schemata(avdl_path: &Path, import_dirs: &[&Path]) -> HashMap<String, Value> {
    let mut builder = Idl2Schemata::new();
    for dir in import_dirs {
        builder.import_dir(dir);
    }
    let output = builder.extract(avdl_path).unwrap_or_else(|e| {
        panic!(
            "failed to extract schemas from {}: {e}",
            avdl_path.display()
        )
    });
    output
        .schemas
        .into_iter()
        .map(|s| (s.name, normalize_crlf(s.schema)))
        .collect()
}

/// Load an expected output file (`.avpr` or `.avsc`) as a `serde_json::Value`.
/// Applies CRLF normalization so tests pass on Windows where Git checks out
/// files with `\r\n` line endings.
fn load_expected(path: &Path) -> Value {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read expected output {}: {e}", path.display()));
    let value: Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("failed to parse expected JSON {}: {e}", path.display()));
    normalize_crlf(value)
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
    let input_dir = PathBuf::from(INPUT_DIR);
    let actual = parse_and_serialize(&input_path("baseball.avdl"), &[&input_dir]);
    let expected = load_expected(&output_path("baseball.avpr"));
    assert_eq!(actual, expected);
}

// ==============================================================================
// Schema Tests (standalone schema syntax)
// ==============================================================================

#[test]
fn test_schema_syntax() {
    let input_dir = PathBuf::from(INPUT_DIR);
    let actual = parse_and_serialize(&input_path("schema_syntax_schema.avdl"), &[&input_dir]);
    let expected = load_expected(&output_path("schema_syntax.avsc"));
    assert_eq!(actual, expected);
}

/// `status_schema.avdl` contains only bare named types (no `schema` keyword,
/// no `protocol`), so the `idl` subcommand should reject it — matching Java's
/// `IdlTool.run()` behavior. The `idl2schemata` path still accepts it (tested
/// in `test_idl2schemata_golden_comparison`).
#[test]
#[cfg_attr(windows, ignore)]
fn test_status_schema_rejected_by_idl() {
    let avdl_path = input_path("status_schema.avdl");
    let result = Idl::new().convert(&avdl_path);
    let err = result.expect_err("idl should reject bare named types file");
    insta::assert_snapshot!(render_diagnostic(&err));
}

// ==============================================================================
// `idl2schemata` Tests
// ==============================================================================

#[test]
fn test_idl2schemata_echo() {
    let schemata = parse_idl2schemata(&input_path("echo.avdl"), &[]);

    let mut keys: Vec<_> = schemata.keys().collect();
    keys.sort();
    assert_eq!(
        keys,
        vec!["Ping", "Pong"],
        "expected Ping and Pong schemas from echo.avdl"
    );

    let ping = &schemata["Ping"];
    assert_eq!(ping["type"], "record");
    assert_eq!(ping["name"], "Ping");
    let ping_fields = ping["fields"].as_array().expect("Ping should have fields");
    assert_eq!(ping_fields.len(), 2);
    assert_eq!(ping_fields[0]["name"], "timestamp");
    assert_eq!(ping_fields[1]["name"], "text");

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

#[test]
fn test_idl2schemata_simple() {
    let schemata = parse_idl2schemata(&input_path("simple.avdl"), &[]);

    let mut names: Vec<&String> = schemata.keys().collect();
    names.sort();
    assert_eq!(
        names,
        vec!["Kind", "MD5", "Status", "TestError", "TestRecord"],
        "expected five named schemas from simple.avdl"
    );

    let kind = &schemata["Kind"];
    assert_eq!(kind["type"], "enum");
    assert_eq!(
        kind["symbols"]
            .as_array()
            .expect("symbols should be an array")
            .len(),
        3
    );
    assert_eq!(kind["aliases"], serde_json::json!(["org.foo.KindOf"]));

    let status = &schemata["Status"];
    assert_eq!(status["type"], "enum");
    assert_eq!(status["default"], "C");

    let test_record = &schemata["TestRecord"];
    assert_eq!(test_record["type"], "record");
    assert_eq!(test_record["doc"], "A TestRecord.");
    assert!(test_record.get("my-property").is_some());
    let fields = test_record["fields"]
        .as_array()
        .expect("TestRecord should have fields");
    assert!(fields.len() > 5, "TestRecord should have many fields");

    let md5 = &schemata["MD5"];
    assert_eq!(md5["type"], "fixed");
    assert_eq!(md5["name"], "MD5");
    assert_eq!(md5["size"], 16);

    let test_error = &schemata["TestError"];
    assert_eq!(test_error["type"], "error");
    assert_eq!(test_error["name"], "TestError");
}

// ==============================================================================
// Negative / Error-Case Tests
// ==============================================================================

/// Duplicate type definitions should produce an error.
#[test]
fn test_duplicate_type_definition() {
    let result = Idl::new().convert_str(
        r#"
        @namespace("org.test")
        protocol DupTest {
            record Dup { string name; }
            record Dup { int id; }
        }
    "#,
    );
    assert!(
        result.is_err(),
        "duplicate type names should produce an error"
    );
}

/// Importing a nonexistent file should produce an error.
#[test]
fn test_import_nonexistent_file() {
    let result = Idl::new().convert_str(
        r#"
        @namespace("org.test")
        protocol ImportTest {
            import schema "does_not_exist.avsc";
            record Rec { string name; }
        }
    "#,
    );
    assert!(
        result.is_err(),
        "importing a nonexistent file should produce an error"
    );
}

/// Nested unions should be rejected during parsing.
#[test]
fn test_nested_union_rejected() {
    let result = Idl::new().convert_str(
        r#"
        @namespace("org.test")
        protocol NestedUnionTest {
            record Bad {
                union { null, union { string, int } } nested;
            }
        }
    "#,
    );
    let err = result.unwrap_err();
    insta::assert_snapshot!(render_diagnostic(&err));
}

/// Type names that collide with Avro built-in types must be rejected.
#[test]
fn test_reserved_type_name_rejected() {
    let result = Idl::new().convert_str(r#"record `int` { string value; }"#);
    let err = result.unwrap_err();
    insta::assert_snapshot!(render_diagnostic(&err));
}

/// Records must not contain duplicate field names.
#[test]
fn test_duplicate_field_name_rejected() {
    let result = Idl::new().convert_str(
        r#"
        @namespace("test")
        protocol P {
            record R {
                string name;
                int name;
            }
        }
    "#,
    );
    let err = result.unwrap_err();
    insta::assert_snapshot!(render_diagnostic(&err));
}

/// Enum declarations must not contain duplicate symbols.
#[test]
fn test_duplicate_enum_symbol_rejected() {
    let result = Idl::new().convert_str(
        r#"
        @namespace("test")
        protocol P {
            enum Color { RED, GREEN, BLUE, RED }
        }
    "#,
    );
    let err = result.unwrap_err();
    insta::assert_snapshot!(render_diagnostic(&err));
}

/// When a named type's identifier contains dots, the dot-derived namespace
/// takes priority over an explicit `@namespace` annotation.
#[test]
fn test_dotted_identifier_namespace_priority() {
    let output = Idl::new()
        .convert_str(
            r#"
        @namespace("foo")
        protocol P {
            @namespace("bar") record com.example.MyRecord {
                int x;
            }
        }
    "#,
        )
        .expect("should parse successfully");

    let types = output.json.get("types").expect("missing types");
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

#[test]
fn test_empty_namespace_annotation_emits_namespace_key() {
    let output = Idl::new()
        .convert_str(
            r#"
        @namespace("org.example")
        protocol P {
            @namespace("")
            record NoNamespace { string name; }
        }
    "#,
        )
        .expect("should parse successfully");

    let types = output
        .json
        .get("types")
        .and_then(|t| t.as_array())
        .expect("missing types");
    assert_eq!(types.len(), 1);
    let record = &types[0];
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

/// Cross-namespace unqualified references should be flagged as unresolved.
#[test]
fn test_cross_namespace_unqualified_reference_is_unresolved() {
    let result = Idl::new().convert_str(
        r#"
        @namespace("org.example")
        protocol P {
            @namespace("org.other")
            record OtherRecord { string name; }
            record MainRecord { OtherRecord other; }
        }
    "#,
    );
    let err = result.unwrap_err();
    insta::assert_snapshot!(render_diagnostic(&err));
}

// ==============================================================================
// Protocol Tests (with IDL imports)
// ==============================================================================

const CLASSPATH_DIR: &str = "avro/lang/java/idl/src/test/idl/putOnClassPath";

#[test]
fn test_import() {
    let input_dir = PathBuf::from(INPUT_DIR);
    let classpath_dir = PathBuf::from(CLASSPATH_DIR);
    let actual = parse_and_serialize(&input_path("import.avdl"), &[&input_dir, &classpath_dir]);
    let expected = load_expected(&output_path("import.avpr"));
    assert_eq!(actual, expected);
}

#[test]
fn test_nestedimport() {
    let input_dir = PathBuf::from(INPUT_DIR);
    let actual = parse_and_serialize(&input_path("nestedimport.avdl"), &[&input_dir]);
    let expected = load_expected(&output_path("nestedimport.avpr"));
    assert_eq!(actual, expected);
}

// ==============================================================================
// Tools Test Suite (tools/src/test/idl/)
// ==============================================================================

const TOOLS_IDL_DIR: &str = "avro/lang/java/tools/src/test/idl";

#[test]
fn test_tools_schema() {
    let avdl_path = PathBuf::from(TOOLS_IDL_DIR).join("schema.avdl");
    let actual = parse_and_serialize(&avdl_path, &[]);
    let expected = load_expected(&PathBuf::from(TOOLS_IDL_DIR).join("schema.avsc"));
    assert_eq!(actual, expected);
}

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

const COMPILER_TEST_DIR: &str = "avro/lang/java/compiler/src/test/idl";

#[test]
fn test_workspace_path() {
    let workspace_dir = PathBuf::from(COMPILER_TEST_DIR).join("work space");
    let avdl_path = workspace_dir.join("root.avdl");
    let expected_path = workspace_dir.join("root.avpr");

    let actual = parse_and_serialize(&avdl_path, &[]);
    let expected = load_expected(&expected_path);
    assert_eq!(actual, expected);
}

// ==============================================================================
// Extra Directory Tests
// ==============================================================================

const EXTRA_DIR: &str = "avro/lang/java/idl/src/test/idl/extra";

#[test]
fn test_extra_protocol_syntax() {
    let avdl_path = PathBuf::from(EXTRA_DIR).join("protocolSyntax.avdl");
    let output = Idl::new()
        .convert(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    assert_eq!(output.json["protocol"], "Parrot");
    assert_eq!(output.json["namespace"], "communication");

    let types = output.json["types"].as_array().expect("missing types");
    assert_eq!(
        types.len(),
        1,
        "protocolSyntax.avdl should define exactly one named type"
    );
    assert_eq!(types[0]["name"], "Message");
    // The type's namespace is omitted from JSON when it matches the enclosing
    // protocol namespace — this is the expected serialization behavior.
    assert_eq!(types[0]["type"], "record");

    let messages = output.json["messages"]
        .as_object()
        .expect("missing messages");
    assert_eq!(messages.len(), 1);
    assert!(
        messages.contains_key("echo"),
        "protocol should have an 'echo' message"
    );
}

#[test]
fn test_extra_schema_syntax() {
    let avdl_path = PathBuf::from(EXTRA_DIR).join("schemaSyntax.avdl");
    let output = Idl::new()
        .convert(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    // The main schema should be an array type with Message items.
    assert_eq!(output.json["type"], "array");
    // Items should reference the Message record (either inline or by name).
    let items = &output.json["items"];
    assert!(
        items.is_object() || items.is_string(),
        "array items should be a record object or a string reference"
    );
}

// ==============================================================================
// Logical Type Propagation Tests
// ==============================================================================

/// Helper: parse an inline `.avdl` string via the builder and return JSON.
fn parse_inline_to_json(avdl_input: &str) -> Value {
    let output = Idl::new()
        .convert_str(avdl_input)
        .unwrap_or_else(|e| panic!("failed to parse inline avdl: {e}"));
    output.json
}

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

    assert_eq!(fields[0]["name"], "created_date");
    assert_eq!(fields[0]["type"]["type"], "int");
    assert_eq!(fields[0]["type"]["logicalType"], "date");

    assert_eq!(fields[1]["name"], "created_time");
    assert_eq!(fields[1]["type"]["type"], "int");
    assert_eq!(fields[1]["type"]["logicalType"], "time-millis");

    assert_eq!(fields[2]["name"], "created_timestamp");
    assert_eq!(fields[2]["type"]["type"], "long");
    assert_eq!(fields[2]["type"]["logicalType"], "timestamp-millis");

    assert_eq!(fields[3]["name"], "local_created");
    assert_eq!(fields[3]["type"]["type"], "long");
    assert_eq!(fields[3]["type"]["logicalType"], "local-timestamp-millis");

    assert_eq!(fields[4]["name"], "event_id");
    assert_eq!(fields[4]["type"]["type"], "string");
    assert_eq!(fields[4]["type"]["logicalType"], "uuid");

    assert_eq!(fields[5]["name"], "amount");
    assert_eq!(fields[5]["type"]["type"], "bytes");
    assert_eq!(fields[5]["type"]["logicalType"], "decimal");
    assert_eq!(fields[5]["type"]["precision"], 10);
    assert_eq!(fields[5]["type"]["scale"], 2);
}

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
    assert_eq!(fields.len(), 3);

    assert_eq!(fields[0]["type"]["logicalType"], "timestamp-micros");
    assert_eq!(fields[1]["type"]["logicalType"], "custom-type");
    assert_eq!(fields[2]["type"]["logicalType"], "temperature-celsius");
}

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
    assert_eq!(fields.len(), 2);

    let allowance = &fields[0];
    assert_eq!(allowance["type"]["logicalType"], "decimal");
    assert_eq!(allowance["type"]["precision"], 6);
    assert_eq!(allowance["type"]["scale"], 2);

    let bounded = &fields[1];
    assert_eq!(bounded["type"]["logicalType"], "fixed-size-string");
    assert_eq!(bounded["type"]["minLength"], 1);
    assert_eq!(bounded["type"]["maxLength"], 50);
}

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

    assert_eq!(fields[0]["type"]["logicalType"], "date");
    assert_eq!(fields[0]["type"]["version"], "2");

    assert_eq!(fields[1]["type"]["logicalType"], "timestamp-millis");
    assert_eq!(fields[1]["type"]["source"], "external");
}

// ==============================================================================
// Additional `idl2schemata` Tests
// ==============================================================================

#[test]
fn test_idl2schemata_interop() {
    let schemata = parse_idl2schemata(&input_path("interop.avdl"), &[]);

    let mut names: Vec<&String> = schemata.keys().collect();
    names.sort();
    assert_eq!(names, vec!["Foo", "Interop", "Kind", "MD5", "Node"],);

    let node = &schemata["Node"];
    assert_eq!(node["type"], "record");
    let node_fields = node["fields"].as_array().expect("Node should have fields");
    let children_type = &node_fields[1]["type"];
    assert_eq!(children_type["type"], "array");
    assert_eq!(children_type["items"], "Node");

    let interop = &schemata["Interop"];
    let interop_fields = interop["fields"]
        .as_array()
        .expect("Interop should have fields");
    assert_eq!(interop_fields.len(), 13);
}

#[test]
fn test_idl2schemata_import() {
    let input_dir = PathBuf::from(INPUT_DIR);
    let classpath_dir = PathBuf::from(CLASSPATH_DIR);
    let schemata = parse_idl2schemata(&input_path("import.avdl"), &[&input_dir, &classpath_dir]);

    assert!(schemata.contains_key("Bar"));
    assert!(schemata.contains_key("Baz"));
    assert!(schemata.contains_key("Foo"));
    assert!(schemata.contains_key("NestedType"));
    assert_eq!(
        schemata.len(),
        10,
        "import.avdl idl2schemata should produce 10 named schemas, got: {:?}",
        schemata.keys().collect::<Vec<_>>()
    );
}

// ==============================================================================
// Import Cycle Detection Tests
// ==============================================================================

#[test]
fn test_self_import_cycle_handled_gracefully() {
    let avdl_path = PathBuf::from("tests/testdata/self_import.avdl");
    let testdata_dir = PathBuf::from("tests/testdata");

    let actual = parse_and_serialize(&avdl_path, &[&testdata_dir]);

    assert_eq!(actual["protocol"], "SelfImport");
    let types = actual["types"].as_array().expect("should have types array");
    assert_eq!(types.len(), 1);
    assert_eq!(types[0]["name"], "Rec");
}

#[test]
fn test_mutual_import_cycle_handled_gracefully() {
    let avdl_path = PathBuf::from("tests/testdata/cycle_a.avdl");
    let testdata_dir = PathBuf::from("tests/testdata");

    let actual = parse_and_serialize(&avdl_path, &[&testdata_dir]);

    assert_eq!(actual["protocol"], "CycleA");
    let types = actual["types"].as_array().expect("should have types array");

    let type_names: Vec<&str> = types.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(type_names.contains(&"RecB"));
    assert!(type_names.contains(&"RecA"));
}

// ==============================================================================
// Doc Comment Warning Tests
// ==============================================================================

#[test]
#[cfg_attr(windows, ignore)]
fn test_comments_warnings() {
    let avdl_path = input_path("comments.avdl");

    let output = Idl::new()
        .convert(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    insta::assert_snapshot!(render_diagnostics(&output.warnings));
}

// ==============================================================================
// Java Test Behavior: `idl2schemata` File Count
// ==============================================================================

#[test]
fn test_idl2schemata_tools_protocol() {
    let avdl_path = PathBuf::from(TOOLS_IDL_DIR).join("protocol.avdl");
    let schemata = parse_idl2schemata(&avdl_path, &[]);

    let mut names: Vec<&String> = schemata.keys().collect();
    names.sort();
    assert_eq!(names, vec!["Kind", "MD5", "TestError", "TestRecord"],);
}

// ==============================================================================
// `idl2schemata` Golden-File Comparison Tests
// ==============================================================================
//
// Golden `.avsc` files live in `tests/testdata/idl2schemata-golden/{name}/`
// and were generated from the Java `avro-tools idl2schemata` command. Each
// schema is compared via full `serde_json::Value` equality, catching field
// serialization, annotation propagation, and default value bugs that a
// metadata-only check would miss.

const IDL2SCHEMATA_GOLDEN_DIR: &str = "tests/testdata/idl2schemata-golden";

/// Load all golden `.avsc` files for a given test case into a name-to-JSON map.
fn load_golden_schemata(test_name: &str) -> HashMap<String, Value> {
    let dir = PathBuf::from(IDL2SCHEMATA_GOLDEN_DIR).join(test_name);
    if !dir.exists() {
        return HashMap::new();
    }
    let mut result = HashMap::new();
    for entry in fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("failed to read golden dir {}: {e}", dir.display()))
    {
        let entry = entry.expect("failed to read directory entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("avsc") {
            let name = path
                .file_stem()
                .expect("avsc file should have a stem")
                .to_string_lossy()
                .to_string();
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
            let value: Value = serde_json::from_str(&content)
                .unwrap_or_else(|e| panic!("failed to parse JSON {}: {e}", path.display()));
            result.insert(name, normalize_crlf(value));
        }
    }
    result
}

#[test]
fn test_idl2schemata_golden_comparison() {
    // Files with no import dirs needed.
    let simple_files = [
        "echo",
        "simple",
        "comments",
        "cycle",
        "forward_ref",
        "interop",
        "leading_underscore",
        "mr_events",
        "namespaces",
        "reservedwords",
        "unicode",
        "union",
        "uuid",
    ];

    // Files that need specific import directories.
    let import_files: &[(&str, &[&str])] = &[
        ("baseball", &[INPUT_DIR]),
        ("import", &[INPUT_DIR, CLASSPATH_DIR]),
        ("nestedimport", &[INPUT_DIR, CLASSPATH_DIR]),
        // Schema-mode files: idl2schemata extracts named types from these too.
        ("schema_syntax_schema", &[INPUT_DIR]),
        ("status_schema", &[]),
    ];

    fn compare_schemata_golden(
        name: &str,
        schemata: &HashMap<String, Value>,
        golden: &HashMap<String, Value>,
    ) {
        // Verify schema counts match.
        let mut actual_names: Vec<&String> = schemata.keys().collect();
        actual_names.sort();
        let mut golden_names: Vec<&String> = golden.keys().collect();
        golden_names.sort();
        assert_eq!(
            actual_names, golden_names,
            "{name}.avdl: schema name mismatch.\n  actual:  {actual_names:?}\n  golden:  {golden_names:?}"
        );

        // Compare each schema's full JSON content.
        for (schema_name, actual_json) in schemata {
            let golden_json = golden.get(schema_name).unwrap_or_else(|| {
                panic!("{name}.avdl: schema '{schema_name}' not found in golden .avsc files")
            });
            assert_eq!(
                actual_json, golden_json,
                "{name}.avdl: schema '{schema_name}' content mismatch"
            );
        }
    }

    for name in &simple_files {
        let avdl = input_path(&format!("{name}.avdl"));
        let schemata = parse_idl2schemata(&avdl, &[]);
        let golden = load_golden_schemata(name);
        compare_schemata_golden(name, &schemata, &golden);
    }

    for &(name, import_dirs) in import_files {
        let avdl = input_path(&format!("{name}.avdl"));
        let dirs: Vec<PathBuf> = import_dirs.iter().map(PathBuf::from).collect();
        let dir_refs: Vec<&Path> = dirs.iter().map(|p| p.as_path()).collect();
        let schemata = parse_idl2schemata(&avdl, &dir_refs);
        let golden = load_golden_schemata(name);
        compare_schemata_golden(name, &schemata, &golden);
    }
}

// ==============================================================================
// `idl2schemata` Error Path Tests
// ==============================================================================

#[test]
fn test_idl2schemata_unresolved_type_detected() {
    let result = Idl2Schemata::new().extract_str(
        r#"
        @namespace("test")
        protocol P {
            record R {
                MissingType field;
            }
        }
    "#,
    );
    let err = result.unwrap_err();
    insta::assert_snapshot!(render_diagnostic(&err));
}

// ==============================================================================
// Test-Root `cycle.avdl`
// ==============================================================================

const TEST_ROOT_DIR: &str = "avro/lang/java/idl/src/test/idl";

#[test]
fn test_cycle_test_root() {
    let avdl_path = PathBuf::from(TEST_ROOT_DIR).join("cycle.avdl");
    let output = Idl::new()
        .convert(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    assert!(output.warnings.is_empty());

    let types = output.json["types"]
        .as_array()
        .expect("should have types array");

    // All 5 types are inlined inside Record1 (first occurrence), so the
    // top-level types array has 1 entry.
    assert_eq!(types.len(), 1);

    let record1 = &types[0];
    assert_eq!(record1["type"], "record");
    assert_eq!(record1["name"], "Record1");
    let r1_fields = record1["fields"]
        .as_array()
        .expect("Record1 should have fields");
    assert_eq!(r1_fields.len(), 2);

    let record3 = &r1_fields[1]["type"];
    assert_eq!(record3["name"], "Record3");
    let r3_fields = record3["fields"]
        .as_array()
        .expect("Record3 should have fields");

    let test_enum = &r3_fields[0]["type"];
    assert_eq!(test_enum["type"], "enum");
    assert_eq!(test_enum["name"], "TestEnum");

    let record2 = &r3_fields[1]["type"];
    assert_eq!(record2["name"], "Record2");
    let r2_fields = record2["fields"]
        .as_array()
        .expect("Record2 should have fields");

    let test_fixed = &r2_fields[0]["type"];
    assert_eq!(test_fixed["type"], "fixed");
    assert_eq!(test_fixed["size"], 16);

    let f_rec1_type = &r2_fields[2]["type"];
    let union = f_rec1_type
        .as_array()
        .expect("fRec1 type should be a union array");
    assert_eq!(union[0], "null");
    assert_eq!(union[1], "Record1");
}

// ==============================================================================
// `logicalTypes.avdl` Test
// ==============================================================================

#[test]
fn test_logical_types_file() {
    let avdl_path = PathBuf::from(TEST_ROOT_DIR).join("logicalTypes.avdl");
    let output = Idl::new()
        .convert(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    assert!(output.warnings.is_empty());

    let types = output.json["types"]
        .as_array()
        .expect("should have types array");
    assert_eq!(types.len(), 1);

    let fields = types[0]["fields"]
        .as_array()
        .expect("LogicalTypeFields should have fields");
    assert_eq!(fields.len(), 9);

    assert_eq!(fields[0]["type"]["logicalType"], "date");
    assert_eq!(fields[1]["type"]["logicalType"], "time-millis");
    assert_eq!(fields[2]["type"]["logicalType"], "timestamp-millis");
    assert_eq!(fields[3]["type"]["logicalType"], "local-timestamp-millis");
    assert_eq!(fields[4]["type"]["logicalType"], "decimal");
    assert_eq!(fields[4]["type"]["precision"], 6);
    assert_eq!(fields[5]["type"]["logicalType"], "uuid");
    assert_eq!(fields[6]["type"]["logicalType"], "timestamp-micros");
    assert_eq!(fields[7]["type"]["logicalType"], "decimal");
    assert_eq!(fields[7]["type"]["precision"], 6);

    assert_eq!(
        fields[8]["type"]["precision"],
        serde_json::json!(3_000_000_000_u64),
    );
}

// ==============================================================================
// Warning Assertion Tests
// ==============================================================================

#[test]
#[cfg_attr(windows, ignore)]
fn test_tools_protocol_warning() {
    let avdl_path = PathBuf::from(TOOLS_IDL_DIR).join("protocol.avdl");
    let output = Idl::new()
        .convert(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    insta::assert_snapshot!(render_diagnostics(&output.warnings));
}

#[test]
#[cfg_attr(windows, ignore)]
fn test_tools_schema_warning() {
    let avdl_path = PathBuf::from(TOOLS_IDL_DIR).join("schema.avdl");
    let output = Idl::new()
        .convert(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    insta::assert_snapshot!(render_diagnostics(&output.warnings));
}

// ==============================================================================
// Message Error Declaration Tests
// ==============================================================================

/// A message with `throws Err1, Err2` should produce an `"errors"` array
/// listing both error types in the serialized JSON output.
#[test]
fn test_multiple_throws_error_types() {
    let input = r#"
        @namespace("test")
        protocol MultiThrows {
            error Err1 { string message; }
            error Err2 { string reason; }
            void dangerous() throws Err1, Err2;
        }
    "#;

    let json = parse_inline_to_json(input);
    let messages = json["messages"]
        .as_object()
        .expect("protocol should have messages");
    let dangerous = messages
        .get("dangerous")
        .expect("should have 'dangerous' message");
    let errors = dangerous["errors"]
        .as_array()
        .expect("throws message should have errors array");
    assert_eq!(
        errors,
        &[serde_json::json!("Err1"), serde_json::json!("Err2")],
        "errors array should list both thrown error types"
    );
}

/// `oneway` and `throws` are grammar-level alternatives, so combining them
/// should produce a parse error. This test guards against future grammar
/// changes that might relax this constraint.
#[test]
fn test_oneway_with_throws_is_rejected() {
    let result = Idl::new().convert_str(
        r#"
        @namespace("test")
        protocol P {
            error E { string msg; }
            void fire(string s) oneway throws E;
        }
    "#,
    );
    assert!(
        result.is_err(),
        "oneway with throws should be rejected as a parse error"
    );
}

// ==============================================================================
// AnnotationOnTypeReference Error Test
// ==============================================================================

#[test]
#[cfg_attr(windows, ignore)]
fn test_annotation_on_type_reference_file() {
    let avdl_path = PathBuf::from(TEST_ROOT_DIR).join("AnnotationOnTypeReference.avdl");
    let result = Idl::new().convert(&avdl_path);
    let err = result.unwrap_err();
    insta::assert_snapshot!(render_diagnostic(&err));
}

// ==============================================================================
// Schema Mode Tests: Standalone `schema <type>;` Declarations
// ==============================================================================
//
// These tests exercise the schema mode path in `walk_idl_file`, which produces
// a standalone `.avsc` JSON value rather than a protocol `.avpr` object. The
// existing test coverage only covered `schema array<T>` with named types;
// these tests fill the gaps for primitives, logical types, maps, unions,
// nullable shorthand, and namespace interactions.

#[test]
fn test_schema_mode_primitive_int() {
    let json = parse_inline_to_json("schema int;");
    assert_eq!(json, serde_json::json!("int"));
}

#[test]
fn test_schema_mode_primitive_string() {
    let json = parse_inline_to_json("schema string;");
    assert_eq!(json, serde_json::json!("string"));
}

#[test]
fn test_schema_mode_primitive_boolean() {
    let json = parse_inline_to_json("schema boolean;");
    assert_eq!(json, serde_json::json!("boolean"));
}

#[test]
fn test_schema_mode_primitive_long() {
    let json = parse_inline_to_json("schema long;");
    assert_eq!(json, serde_json::json!("long"));
}

#[test]
fn test_schema_mode_primitive_bytes() {
    let json = parse_inline_to_json("schema bytes;");
    assert_eq!(json, serde_json::json!("bytes"));
}

#[test]
fn test_schema_mode_primitive_float() {
    let json = parse_inline_to_json("schema float;");
    assert_eq!(json, serde_json::json!("float"));
}

#[test]
fn test_schema_mode_primitive_double() {
    let json = parse_inline_to_json("schema double;");
    assert_eq!(json, serde_json::json!("double"));
}

#[test]
fn test_schema_mode_primitive_null() {
    let json = parse_inline_to_json("schema null;");
    assert_eq!(json, serde_json::json!("null"));
}

#[test]
fn test_schema_mode_logical_type_date() {
    let json = parse_inline_to_json("schema date;");
    assert_eq!(json["type"], "int");
    assert_eq!(json["logicalType"], "date");
}

#[test]
fn test_schema_mode_logical_type_uuid() {
    let json = parse_inline_to_json("schema uuid;");
    assert_eq!(json["type"], "string");
    assert_eq!(json["logicalType"], "uuid");
}

#[test]
fn test_schema_mode_logical_type_time_ms() {
    let json = parse_inline_to_json("schema time_ms;");
    assert_eq!(json["type"], "int");
    assert_eq!(json["logicalType"], "time-millis");
}

#[test]
fn test_schema_mode_logical_type_timestamp_ms() {
    let json = parse_inline_to_json("schema timestamp_ms;");
    assert_eq!(json["type"], "long");
    assert_eq!(json["logicalType"], "timestamp-millis");
}

#[test]
fn test_schema_mode_logical_type_decimal() {
    let json = parse_inline_to_json("schema decimal(10, 2);");
    assert_eq!(json["type"], "bytes");
    assert_eq!(json["logicalType"], "decimal");
    assert_eq!(json["precision"], 10);
    assert_eq!(json["scale"], 2);
}

#[test]
fn test_schema_mode_map() {
    let json = parse_inline_to_json("schema map<string>;");
    assert_eq!(json["type"], "map");
    assert_eq!(json["values"], "string");
}

#[test]
fn test_schema_mode_map_with_complex_values() {
    let json = parse_inline_to_json("schema map<array<int>>;");
    assert_eq!(json["type"], "map");
    assert_eq!(json["values"]["type"], "array");
    assert_eq!(json["values"]["items"], "int");
}

#[test]
fn test_schema_mode_union() {
    let json = parse_inline_to_json("schema union { null, string };");
    let union = json
        .as_array()
        .expect("union should serialize as a JSON array");
    assert_eq!(union.len(), 2);
    assert_eq!(union[0], "null");
    assert_eq!(union[1], "string");
}

#[test]
fn test_schema_mode_union_multiple_types() {
    let json = parse_inline_to_json("schema union { null, string, int, long };");
    let union = json
        .as_array()
        .expect("union should serialize as a JSON array");
    assert_eq!(union.len(), 4);
    assert_eq!(union[0], "null");
    assert_eq!(union[1], "string");
    assert_eq!(union[2], "int");
    assert_eq!(union[3], "long");
}

#[test]
fn test_schema_mode_nullable_shorthand() {
    let json = parse_inline_to_json("schema string?;");
    let union = json
        .as_array()
        .expect("nullable should serialize as a union array");
    assert_eq!(union.len(), 2);
    assert_eq!(union[0], "null");
    assert_eq!(union[1], "string");
}

#[test]
fn test_schema_mode_nullable_int() {
    let json = parse_inline_to_json("schema int?;");
    let union = json
        .as_array()
        .expect("nullable should serialize as a union array");
    assert_eq!(union.len(), 2);
    assert_eq!(union[0], "null");
    assert_eq!(union[1], "int");
}

#[test]
fn test_schema_mode_array() {
    let json = parse_inline_to_json("schema array<string>;");
    assert_eq!(json["type"], "array");
    assert_eq!(json["items"], "string");
}

#[test]
fn test_schema_mode_named_type_with_namespace() {
    // The grammar requires `schema <type>;` before named type declarations.
    // The schema declaration forward-references the record defined below it.
    let json = parse_inline_to_json(
        r#"
        namespace org.test;
        schema Foo;
        @namespace("org.test")
        record Foo { string name; }
        "#,
    );
    assert_eq!(json["type"], "record");
    assert_eq!(json["name"], "Foo");
    assert_eq!(json["namespace"], "org.test");
    let fields = json["fields"]
        .as_array()
        .expect("record should have fields");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0]["name"], "name");
    assert_eq!(fields[0]["type"], "string");
}

#[test]
fn test_schema_mode_namespace_directive() {
    let json = parse_inline_to_json(
        r#"
        namespace org.example;
        schema Bar;
        record Bar { int value; }
        "#,
    );
    assert_eq!(json["type"], "record");
    assert_eq!(json["name"], "Bar");
    assert_eq!(json["namespace"], "org.example");
}

#[test]
fn test_schema_mode_multiple_named_types_with_schema_ref() {
    // When multiple named types are declared and `schema` references one,
    // all named types should be available for cross-referencing. The grammar
    // requires `schema <type>;` before named type declarations.
    let json = parse_inline_to_json(
        r#"
        namespace org.test;
        schema Item;
        enum Color { RED, GREEN, BLUE }
        record Item {
            string name;
            Color color;
        }
        "#,
    );
    assert_eq!(json["type"], "record");
    assert_eq!(json["name"], "Item");
    let fields = json["fields"]
        .as_array()
        .expect("record should have fields");
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0]["name"], "name");
    // The Color enum should be inlined on first occurrence within the schema.
    let color_type = &fields[1]["type"];
    assert_eq!(color_type["type"], "enum");
    assert_eq!(color_type["name"], "Color");
}

#[test]
fn test_schema_mode_custom_logical_type_annotation() {
    let json = parse_inline_to_json("schema @logicalType(\"date\") int;");
    assert_eq!(json["type"], "int");
    assert_eq!(json["logicalType"], "date");
}

// ==============================================================================
// IdlUtils Test Files (idl_utils_test_protocol.avdl, idl_utils_test_schema.avdl)
// ==============================================================================
//
// These files live in the Java `resources/` directory (for classpath loading)
// rather than in `test/idl/input/`, so they were not picked up by the golden-file
// test sweep. They combine many features in a single file: `@version()` with
// numeric values, `@aliases()` on records, `@generator()` annotations, hyphenated
// annotation keys, `@order()`, cross-namespace references, `oneway`, `throws`,
// annotations inside unions, `decimal()`, and custom logical types.

const IDL_UTILS_DIR: &str =
    "avro/lang/java/idl/src/test/resources/org/apache/avro/util";

#[test]
fn test_idl_utils_test_protocol() {
    let avdl_path = PathBuf::from(IDL_UTILS_DIR).join("idl_utils_test_protocol.avdl");
    let output = Idl::new()
        .convert(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    let json = &output.json;

    // Protocol-level metadata.
    assert_eq!(json["protocol"], "HappyFlow");
    assert_eq!(json["namespace"], "naming");
    assert_eq!(json["version"], "1.0.5");

    // Top-level types: NewMessage and Failure. Counter, Flag, and Nonce are
    // inlined within NewMessage (first occurrence), so they do not appear as
    // separate top-level entries.
    let types = json["types"].as_array().expect("missing types array");
    let type_names: Vec<&str> = types.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(type_names.contains(&"NewMessage"), "should define NewMessage");
    assert!(type_names.contains(&"Failure"), "should define Failure");
    assert_eq!(types.len(), 2, "expected NewMessage and Failure as top-level types");

    // NewMessage should have the `@aliases` annotation.
    let new_message = types
        .iter()
        .find(|t| t["name"] == "NewMessage")
        .expect("NewMessage not found");
    assert_eq!(new_message["doc"], "A sample record type.");
    assert_eq!(new_message["version"], 2);
    let aliases = new_message["aliases"]
        .as_array()
        .expect("NewMessage should have aliases");
    assert!(aliases.iter().any(|a| a == "OldMessage"));

    // NewMessage fields should include the `@generator` annotation on `id`.
    let fields = new_message["fields"]
        .as_array()
        .expect("NewMessage should have fields");
    let id_field = fields.iter().find(|f| f["name"] == "id").expect("id field");
    assert_eq!(id_field["generator"], "uuid-type1");

    // The `flags` field should have `@order("DESCENDING")`.
    let flags_field = fields
        .iter()
        .find(|f| f["name"] == "flags")
        .expect("flags field");
    assert_eq!(flags_field["order"], "descending");

    // Nonce is inlined within NewMessage (first occurrence). Verify it
    // appears as a fixed type with size 8 on the `nonce` field.
    let nonce_field = fields
        .iter()
        .find(|f| f["name"] == "nonce")
        .expect("nonce field");
    assert_eq!(nonce_field["type"]["type"], "fixed");
    assert_eq!(nonce_field["type"]["name"], "Nonce");
    assert_eq!(nonce_field["type"]["size"], 8);

    // Messages: `send` (oneway) and `echo` (throws Failure).
    let messages = json["messages"]
        .as_object()
        .expect("should have messages");
    assert_eq!(messages.len(), 2);

    let send = messages.get("send").expect("send message");
    assert_eq!(send["one-way"], true);

    let echo = messages.get("echo").expect("echo message");
    assert_eq!(echo["doc"], "Simple echoing service");
    let errors = echo["errors"]
        .as_array()
        .expect("echo should have errors");
    assert!(errors.iter().any(|e| e == "Failure"));
}

#[test]
fn test_idl_utils_test_protocol_idl2schemata() {
    let avdl_path = PathBuf::from(IDL_UTILS_DIR).join("idl_utils_test_protocol.avdl");
    let schemata = parse_idl2schemata(&avdl_path, &[]);

    let mut names: Vec<&String> = schemata.keys().collect();
    names.sort();
    assert_eq!(
        names,
        vec!["Counter", "Failure", "Flag", "NewMessage", "Nonce"],
        "idl_utils_test_protocol should produce 5 named schemas"
    );

    // Verify the Counter record has the expected fields.
    let counter = &schemata["Counter"];
    assert_eq!(counter["type"], "record");
    let counter_fields = counter["fields"]
        .as_array()
        .expect("Counter should have fields");
    assert_eq!(counter_fields.len(), 3);
}

#[test]
fn test_idl_utils_test_schema() {
    let avdl_path = PathBuf::from(IDL_UTILS_DIR).join("idl_utils_test_schema.avdl");
    let output = Idl::new()
        .convert(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    let json = &output.json;

    // Schema mode: the top-level output should be a record (NewMessage).
    assert_eq!(json["type"], "record");
    assert_eq!(json["name"], "NewMessage");
    assert_eq!(json["namespace"], "naming");
    assert_eq!(json["doc"], "A sample record type.");
    assert_eq!(json["version"], 2);

    let aliases = json["aliases"]
        .as_array()
        .expect("NewMessage should have aliases");
    assert!(aliases.iter().any(|a| a == "OldMessage"));

    let fields = json["fields"]
        .as_array()
        .expect("NewMessage should have fields");
    // NewMessage has 11 fields: id, message, flags, mainCounter, otherCounters,
    // nonce, my_date, my_time, my_timestamp, my_number, my_dummy.
    assert_eq!(fields.len(), 11);

    // Spot-check the decimal field.
    let my_number = fields
        .iter()
        .find(|f| f["name"] == "my_number")
        .expect("my_number field");
    assert_eq!(my_number["type"]["logicalType"], "decimal");
    assert_eq!(my_number["type"]["precision"], 12);
    assert_eq!(my_number["type"]["scale"], 3);

    // Spot-check the custom logical type field.
    let my_dummy = fields
        .iter()
        .find(|f| f["name"] == "my_dummy")
        .expect("my_dummy field");
    assert_eq!(my_dummy["type"]["logicalType"], "time-micros");
}

#[test]
fn test_idl_utils_test_schema_idl2schemata() {
    let avdl_path = PathBuf::from(IDL_UTILS_DIR).join("idl_utils_test_schema.avdl");
    let schemata = parse_idl2schemata(&avdl_path, &[]);

    let mut names: Vec<&String> = schemata.keys().collect();
    names.sort();
    assert_eq!(
        names,
        vec!["Counter", "Flag", "NewMessage", "Nonce"],
        "idl_utils_test_schema should produce 4 named schemas"
    );
}

// ==============================================================================
// Doc Comment Content Assertions
// ==============================================================================
//
// These tests parse `comments.avdl` and assert specific doc comment string
// values, replicating Java's `testDocCommentsAndWarnings` assertions. The
// golden-file comparison test (`test_comments`) implicitly verifies the same
// values, but these explicit assertions survive golden file regeneration that
// might accidentally drop or corrupt a doc string.

#[test]
fn test_comments_doc_content() {
    let json = parse_and_serialize(&input_path("comments.avdl"), &[]);
    let types = json["types"].as_array().expect("missing types array");

    // ---- Named type doc strings ----

    // DocumentedEnum: doc == "Documented Enum"
    let documented_enum = types
        .iter()
        .find(|t| t["name"] == "DocumentedEnum")
        .expect("DocumentedEnum not found");
    assert_eq!(
        documented_enum["doc"], "Documented Enum",
        "DocumentedEnum should have doc 'Documented Enum'"
    );

    // UndocumentedEnum: doc should be absent
    let undocumented_enum = types
        .iter()
        .find(|t| t["name"] == "UndocumentedEnum")
        .expect("UndocumentedEnum not found");
    assert!(
        undocumented_enum.get("doc").is_none()
            || undocumented_enum["doc"].is_null(),
        "UndocumentedEnum should not have a doc string"
    );

    // DocumentedFixed: doc == "Documented Fixed Type"
    let documented_fixed = types
        .iter()
        .find(|t| t["name"] == "DocumentedFixed")
        .expect("DocumentedFixed not found");
    assert_eq!(
        documented_fixed["doc"], "Documented Fixed Type",
        "DocumentedFixed should have doc 'Documented Fixed Type'"
    );

    // UndocumentedFixed: doc should be absent
    let undocumented_fixed = types
        .iter()
        .find(|t| t["name"] == "UndocumentedFixed")
        .expect("UndocumentedFixed not found");
    assert!(
        undocumented_fixed.get("doc").is_none()
            || undocumented_fixed["doc"].is_null(),
        "UndocumentedFixed should not have a doc string"
    );

    // DocumentedError: doc == "Documented Error"
    let documented_error = types
        .iter()
        .find(|t| t["name"] == "DocumentedError")
        .expect("DocumentedError not found");
    assert_eq!(
        documented_error["doc"], "Documented Error",
        "DocumentedError should have doc 'Documented Error'"
    );

    // DocumentedError field docs.
    let error_fields = documented_error["fields"]
        .as_array()
        .expect("DocumentedError should have fields");

    let reason_field = error_fields
        .iter()
        .find(|f| f["name"] == "reason")
        .expect("reason field not found");
    assert_eq!(
        reason_field["doc"], "Documented Reason Field",
        "reason field should have doc 'Documented Reason Field'"
    );

    let explanation_field = error_fields
        .iter()
        .find(|f| f["name"] == "explanation")
        .expect("explanation field not found");
    assert_eq!(
        explanation_field["doc"], "Default Doc Explanation Field",
        "explanation field should have doc 'Default Doc Explanation Field'"
    );

    // UndocumentedRecord: doc should be absent
    let undocumented_record = types
        .iter()
        .find(|t| t["name"] == "UndocumentedRecord")
        .expect("UndocumentedRecord not found");
    assert!(
        undocumented_record.get("doc").is_none()
            || undocumented_record["doc"].is_null(),
        "UndocumentedRecord should not have a doc string"
    );

    // ---- Message doc strings ----

    let messages = json["messages"]
        .as_object()
        .expect("should have messages");

    // documentedMethod: doc == "Documented Method"
    let documented_method = messages
        .get("documentedMethod")
        .expect("documentedMethod not found");
    assert_eq!(
        documented_method["doc"], "Documented Method",
        "documentedMethod should have doc 'Documented Method'"
    );

    // documentedMethod parameter docs.
    let params = documented_method["request"]
        .as_array()
        .expect("documentedMethod should have request params");

    let message_param = params
        .iter()
        .find(|p| p["name"] == "message")
        .expect("message param not found");
    assert_eq!(
        message_param["doc"], "Documented Parameter",
        "message param should have doc 'Documented Parameter'"
    );

    let def_msg_param = params
        .iter()
        .find(|p| p["name"] == "defMsg")
        .expect("defMsg param not found");
    assert_eq!(
        def_msg_param["doc"], "Default Documented Parameter",
        "defMsg param should have doc 'Default Documented Parameter'"
    );

    // undocumentedMethod: doc should be absent
    let undocumented_method = messages
        .get("undocumentedMethod")
        .expect("undocumentedMethod not found");
    assert!(
        undocumented_method.get("doc").is_none()
            || undocumented_method["doc"].is_null(),
        "undocumentedMethod should not have a doc string"
    );
}

// ==============================================================================
// gRPC, Maven, and Integration-Test Subproject Tests
// ==============================================================================
//
// These `.avdl` files live in Java subprojects outside the main `idl/` and
// `tools/` test directories. Having them in `cargo test` guards against
// regressions in feature combinations that are otherwise only exercised by
// `compare-adhoc.sh`.

#[test]
fn test_grpc_test_service() {
    let avdl_path =
        PathBuf::from("avro/lang/java/grpc/src/test/avro/TestService.avdl");
    let output = Idl::new()
        .convert(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    let json = &output.json;

    assert_eq!(json["protocol"], "TestService");
    assert_eq!(json["namespace"], "org.apache.avro.grpc.test");
    assert_eq!(json["doc"], "An example protocol in Avro IDL");

    // Named types: Kind (enum), MD5 (fixed, size 4), TestRecord, TestError.
    let types = json["types"].as_array().expect("missing types");
    let type_names: Vec<&str> = types.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(type_names.contains(&"Kind"));
    assert!(type_names.contains(&"MD5"));
    assert!(type_names.contains(&"TestRecord"));
    assert!(type_names.contains(&"TestError"));

    // MD5 fixed with size 4 (not the usual 16).
    let md5 = types.iter().find(|t| t["name"] == "MD5").expect("MD5");
    assert_eq!(md5["type"], "fixed");
    assert_eq!(md5["size"], 4);

    // Messages: echo, add (3 args), error (throws TestError), ping (oneway),
    // concatenate (union return).
    let messages = json["messages"]
        .as_object()
        .expect("should have messages");
    assert_eq!(messages.len(), 5);

    // `add` has 3 int arguments.
    let add = messages.get("add").expect("add message");
    let add_params = add["request"]
        .as_array()
        .expect("add should have request params");
    assert_eq!(add_params.len(), 3);

    // `ping` is oneway.
    let ping = messages.get("ping").expect("ping message");
    assert_eq!(ping["one-way"], true);

    // `error` throws TestError.
    let error_msg = messages.get("error").expect("error message");
    let errors = error_msg["errors"]
        .as_array()
        .expect("error message should have errors");
    assert!(errors.iter().any(|e| e == "TestError"));

    // `concatenate` returns union {null, string}.
    let concatenate = messages.get("concatenate").expect("concatenate message");
    let response = concatenate["response"]
        .as_array()
        .expect("concatenate response should be a union array");
    assert_eq!(response.len(), 2);
    assert_eq!(response[0], "null");
    assert_eq!(response[1], "string");
}

#[test]
fn test_maven_user() {
    let avdl_path =
        PathBuf::from("avro/lang/java/maven-plugin/src/test/avro/User.avdl");
    let output = Idl::new()
        .convert(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    let json = &output.json;

    assert_eq!(json["protocol"], "IdlTest");
    assert_eq!(json["namespace"], "test");

    let types = json["types"].as_array().expect("missing types");
    let type_names: Vec<&str> = types.iter().filter_map(|t| t["name"].as_str()).collect();
    assert!(type_names.contains(&"IdlPrivacy"));
    assert!(type_names.contains(&"IdlUser"));

    // IdlPrivacy is an enum with mixed-case symbols.
    let privacy = types
        .iter()
        .find(|t| t["name"] == "IdlPrivacy")
        .expect("IdlPrivacy");
    assert_eq!(privacy["type"], "enum");
    let symbols = privacy["symbols"]
        .as_array()
        .expect("IdlPrivacy should have symbols");
    assert_eq!(symbols, &[serde_json::json!("Public"), serde_json::json!("Private")]);

    // IdlUser has a `timestamp_ms` logical type field.
    let user = types
        .iter()
        .find(|t| t["name"] == "IdlUser")
        .expect("IdlUser");
    let fields = user["fields"]
        .as_array()
        .expect("IdlUser should have fields");

    let modified_on = fields
        .iter()
        .find(|f| f["name"] == "modifiedOn")
        .expect("modifiedOn field");
    assert_eq!(modified_on["type"]["logicalType"], "timestamp-millis");

    // `privacy` field is a nullable union with IdlPrivacy.
    let privacy_field = fields
        .iter()
        .find(|f| f["name"] == "privacy")
        .expect("privacy field");
    let privacy_type = privacy_field["type"]
        .as_array()
        .expect("privacy field type should be a union");
    assert_eq!(privacy_type[0], "null");
}

#[test]
fn test_custom_conversion_idl() {
    let avdl_path = PathBuf::from(
        "avro/lang/java/integration-test/codegen-test/src/test/resources/avro/custom_conversion_idl.avdl",
    );
    let output = Idl::new()
        .convert(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    let json = &output.json;

    assert_eq!(json["protocol"], "LogicalTypesWithCustomConversionIdlProtocol");
    assert_eq!(json["namespace"], "org.apache.avro.codegentest.testdata");

    let types = json["types"].as_array().expect("missing types");
    assert_eq!(types.len(), 1);

    let record = &types[0];
    assert_eq!(record["name"], "LogicalTypesWithCustomConversionIdl");
    assert_eq!(
        record["doc"],
        "Test unions with logical types in generated Java classes"
    );

    let fields = record["fields"]
        .as_array()
        .expect("record should have fields");
    assert_eq!(fields.len(), 4);

    // `nullableCustomField`: union { null, decimal(9,2) } with default null.
    let nullable_custom = fields
        .iter()
        .find(|f| f["name"] == "nullableCustomField")
        .expect("nullableCustomField");
    let nc_type = nullable_custom["type"]
        .as_array()
        .expect("nullableCustomField should be a union");
    assert_eq!(nc_type[0], "null");
    assert_eq!(nc_type[1]["logicalType"], "decimal");
    assert_eq!(nc_type[1]["precision"], 9);
    assert_eq!(nc_type[1]["scale"], 2);

    // `nonNullCustomField`: decimal(9,2) directly.
    let non_null_custom = fields
        .iter()
        .find(|f| f["name"] == "nonNullCustomField")
        .expect("nonNullCustomField");
    assert_eq!(non_null_custom["type"]["logicalType"], "decimal");
    assert_eq!(non_null_custom["type"]["precision"], 9);

    // `nullableFixedSizeString`: union with @logicalType and @minLength/@maxLength.
    let nullable_fss = fields
        .iter()
        .find(|f| f["name"] == "nullableFixedSizeString")
        .expect("nullableFixedSizeString");
    let nfss_type = nullable_fss["type"]
        .as_array()
        .expect("nullableFixedSizeString should be a union");
    assert_eq!(nfss_type[0], "null");
    assert_eq!(nfss_type[1]["logicalType"], "fixed-size-string");
    assert_eq!(nfss_type[1]["minLength"], 1);
    assert_eq!(nfss_type[1]["maxLength"], 50);

    // `nonNullFixedSizeString`: direct @logicalType bytes field.
    let non_null_fss = fields
        .iter()
        .find(|f| f["name"] == "nonNullFixedSizeString")
        .expect("nonNullFixedSizeString");
    assert_eq!(non_null_fss["type"]["logicalType"], "fixed-size-string");
    assert_eq!(non_null_fss["type"]["minLength"], 1);
    assert_eq!(non_null_fss["type"]["maxLength"], 50);
}
