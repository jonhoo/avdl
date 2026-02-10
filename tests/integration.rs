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

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use std::fmt::Write;

use avdl::{Idl, Idl2Schemata};
use miette::{GraphicalReportHandler, GraphicalTheme};
use pretty_assertions::assert_eq;
use serde_json::Value;

const INPUT_DIR: &str = "avro/lang/java/idl/src/test/idl/input";
const OUTPUT_DIR: &str = "avro/lang/java/idl/src/test/idl/output";

/// Recursively replace `\r\n` with `\n` in all JSON string values. This is a
/// no-op on Linux/macOS; on Windows, Git checks out `.avdl` files with `\r\n`
/// line endings, which causes doc-comment strings to differ from the golden
/// `.avpr` files that use `\n`.
fn normalize_crlf(value: Value) -> Value {
    match value {
        Value::String(s) => Value::String(s.replace("\r\n", "\n")),
        Value::Array(arr) => Value::Array(arr.into_iter().map(normalize_crlf).collect()),
        Value::Object(obj) => {
            Value::Object(obj.into_iter().map(|(k, v)| (k, normalize_crlf(v))).collect())
        }
        other => other,
    }
}

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

/// Render a single error to a deterministic string suitable for snapshot
/// testing. Uses the same theme and width as [`render_warnings`].
fn render_error(err: &miette::Report) -> String {
    let handler =
        GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor()).with_width(80);
    let mut buf = String::new();
    handler
        .render_report(&mut buf, err.as_ref())
        .expect("render to String is infallible");
    buf
}

/// Render a list of warnings to a deterministic string suitable for snapshot
/// testing. Uses miette's graphical handler with unicode-nocolor theme and
/// fixed 80-column width for reproducible output.
fn render_warnings(warnings: &[miette::Report]) -> String {
    let handler =
        GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor()).with_width(80);
    let mut buf = String::new();
    for (i, w) in warnings.iter().enumerate() {
        if i > 0 {
            writeln!(buf).expect("write to String is infallible");
        }
        handler
            .render_report(&mut buf, w.as_ref())
            .expect("render to String is infallible");
    }
    buf
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

#[test]
fn test_status_schema() {
    let actual = parse_and_serialize(&input_path("status_schema.avdl"), &[]);
    let expected = load_expected(&output_path("status.avsc"));
    assert_eq!(actual, expected);
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
    insta::assert_snapshot!(render_error(&err));
}

/// Type names that collide with Avro built-in types must be rejected.
#[test]
fn test_reserved_type_name_rejected() {
    let result = Idl::new().convert_str(r#"record `int` { string value; }"#);
    let err = result.unwrap_err();
    insta::assert_snapshot!(render_error(&err));
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
    insta::assert_snapshot!(render_error(&err));
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
    insta::assert_snapshot!(render_error(&err));
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
    let msg = format!("{err}");
    assert!(
        msg.contains("org.example.OtherRecord"),
        "unqualified cross-namespace reference should be flagged as unresolved, got: {msg}"
    );
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
fn test_comments_warnings() {
    let avdl_path = input_path("comments.avdl");

    let output = Idl::new()
        .convert(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    insta::assert_snapshot!(render_warnings(&output.warnings));
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
    let msg = format!("{err}");
    assert!(
        msg.contains("Undefined name"),
        "idl2schemata should detect unresolved type references, got: {msg}"
    );
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
fn test_tools_protocol_warning() {
    let avdl_path = PathBuf::from(TOOLS_IDL_DIR).join("protocol.avdl");
    let output = Idl::new()
        .convert(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    insta::assert_snapshot!(render_warnings(&output.warnings));
}

#[test]
fn test_tools_schema_warning() {
    let avdl_path = PathBuf::from(TOOLS_IDL_DIR).join("schema.avdl");
    let output = Idl::new()
        .convert(&avdl_path)
        .unwrap_or_else(|e| panic!("failed to parse {}: {e}", avdl_path.display()));

    insta::assert_snapshot!(render_warnings(&output.warnings));
}

// ==============================================================================
// AnnotationOnTypeReference Error Test
// ==============================================================================

#[test]
fn test_annotation_on_type_reference_file() {
    let avdl_path = PathBuf::from(TEST_ROOT_DIR).join("AnnotationOnTypeReference.avdl");
    let result = Idl::new().convert(&avdl_path);
    let err = result.unwrap_err();
    insta::assert_snapshot!(render_error(&err));
}
