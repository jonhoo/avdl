// ==============================================================================
// Error Reporting Snapshot Tests
// ==============================================================================
//
// These tests verify the *content* and *quality* of error messages produced by
// malformed or invalid `.avdl` inputs. Each test feeds an inline `.avdl` string
// to the parser, captures the rendered error output, and snapshots it with
// `insta` so that changes to error messages are reviewed explicitly.
//
// The helper functions render errors through `miette`'s `GraphicalReportHandler`
// (with Unicode and color disabled for reproducible snapshots) so that we test
// what the user actually sees, including source spans and labels.

use avdl::reader::{parse_idl, DeclItem, Warning};
use avdl::resolve::SchemaRegistry;
use miette::{Diagnostic, GraphicalReportHandler, GraphicalTheme};

// ==============================================================================
// Test Helpers
// ==============================================================================

/// Parse an inline `.avdl` string and return the rendered error message.
///
/// Uses `miette`'s `GraphicalReportHandler` to render rich diagnostics (with
/// source spans and labels) when available, falling back to plain `Display`
/// output for errors without source location info.
///
/// Returns `None` if parsing succeeds.
fn parse_error(input: &str) -> Option<String> {
    match parse_idl(input) {
        Ok(_) => None,
        Err(e) => {
            let mut buf = String::new();
            // Use the ASCII theme for reproducible snapshots (no Unicode box
            // drawing, no ANSI color codes).
            let handler = GraphicalReportHandler::new_themed(GraphicalTheme::none())
                .with_width(80);

            // Try to render as a full miette diagnostic. Only use the
            // graphical handler when the underlying error carries source code
            // and labels -- otherwise the graphical output adds noisy prefixes
            // (`x`, `|`) without any meaningful source context.
            let diag: &dyn Diagnostic = e.as_ref();
            if diag.source_code().is_some() {
                if handler.render_report(&mut buf, diag).is_ok() {
                    return Some(buf);
                }
            }

            // Fall back to plain Display for errors without diagnostic info.
            Some(format!("{e}"))
        }
    }
}

/// Parse an inline `.avdl` string and return warnings from a successful parse.
///
/// Panics if parsing fails, since warning tests require a successful parse.
fn parse_warnings(input: &str) -> Vec<Warning> {
    let (_idl_file, _decl_items, warnings) =
        parse_idl(input).expect("warning test input should parse successfully");
    warnings
}

/// Parse an inline `.avdl` string, then attempt to register all types in a
/// `SchemaRegistry` and return the first registration error message.
///
/// This catches semantic errors like duplicate type names that are only detected
/// during schema registration, not during parsing.
fn registry_error(input: &str) -> Option<String> {
    let (_idl_file, decl_items, _warnings) =
        parse_idl(input).expect("registry error test input should parse successfully");

    let mut registry = SchemaRegistry::new();
    for item in &decl_items {
        if let DeclItem::Type(schema) = item {
            if let Err(e) = registry.register(schema.clone()) {
                return Some(e);
            }
        }
    }
    None
}

// ==============================================================================
// Syntax Errors (ANTLR Parse Failures)
// ==============================================================================

/// Missing semicolon after a field declaration. The parser should report a
/// useful error pointing at the location where the semicolon was expected.
#[test]
fn test_error_missing_semicolon() {
    let input = "protocol P { record R { int x } }";
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Missing closing brace for the protocol — the record is closed but the
/// protocol body is not.
#[test]
fn test_error_unclosed_brace() {
    let input = "protocol P { record R { int x; }";
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Malformed union with a double comma (empty type slot).
#[test]
fn test_error_malformed_union() {
    let input = "protocol P { record R { union { int, , string } x; } }";
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Invalid token in type position — a number literal where a type name is
/// expected.
#[test]
fn test_error_invalid_token_in_type_position() {
    let input = "protocol P { record R { 123 x; } }";
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Malformed fixed declaration with a non-numeric size argument.
#[test]
fn test_error_malformed_fixed_size() {
    let input = "protocol P { fixed F(not_a_number); }";
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

// ==============================================================================
// Semantic / Validation Errors
// ==============================================================================

/// Two records with the same fully-qualified name should produce a "duplicate
/// schema name" error during registration.
#[test]
fn test_error_duplicate_type_name() {
    let input = r#"
        @namespace("org.test")
        protocol P {
            record Dup { string name; }
            record Dup { int id; }
        }
    "#;
    let error = registry_error(input).expect("should produce a registration error");
    insta::assert_snapshot!(error);
}

/// A record with two fields sharing the same name should be rejected.
#[test]
fn test_error_duplicate_field_name() {
    let input = r#"
        @namespace("test")
        protocol P {
            record R {
                string name;
                int name;
            }
        }
    "#;
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Nested unions are forbidden by the Avro specification: "Unions may not
/// immediately contain other unions."
#[test]
fn test_error_nested_union() {
    let input = r#"
        @namespace("test")
        protocol P {
            record Bad {
                union { null, union { string, int } } nested;
            }
        }
    "#;
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// An enum with duplicate symbols should be rejected.
#[test]
fn test_error_duplicate_enum_symbol() {
    let input = r#"
        @namespace("test")
        protocol P {
            enum Color { RED, GREEN, BLUE, RED }
        }
    "#;
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// A type name that collides with an Avro built-in type (e.g., `int`) should
/// be rejected with an "Illegal name" error.
#[test]
fn test_error_reserved_type_name() {
    let input = r#"record `int` { string value; }"#;
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Referencing an undefined type should produce an unresolved reference when
/// the schema registry validates references.
#[test]
fn test_error_undefined_type() {
    let input = r#"
        @namespace("test")
        protocol P {
            record R {
                Nonexistent x;
            }
        }
    "#;
    // This test exercises the registry's validate_references method rather than
    // parse_idl directly, since forward references are allowed during parsing
    // and only flagged during validation.
    let (_idl_file, decl_items, _warnings) =
        parse_idl(input).expect("parsing should succeed with unresolved references");

    let mut registry = SchemaRegistry::new();
    for item in &decl_items {
        if let DeclItem::Type(schema) = item {
            let _ = registry.register(schema.clone());
        }
    }

    let unresolved = registry.validate_references();
    assert!(
        !unresolved.is_empty(),
        "should have unresolved references for undefined type"
    );
    insta::assert_snapshot!(format!("unresolved references: {:?}", unresolved));
}

// ==============================================================================
// Warning Tests
// ==============================================================================

/// A doc comment (`/** ... */`) before a field that already has a doc comment,
/// or in a position where it's not attached to a declaration, should produce
/// an out-of-place doc comment warning (not an error).
#[test]
fn test_warning_out_of_place_doc_comment() {
    // Place a doc comment before a closing brace where no declaration follows.
    let input = r#"
        @namespace("test")
        protocol P {
            record R {
                string name;
                /** This doc comment is orphaned — nothing follows it. */
            }
        }
    "#;
    let warnings = parse_warnings(input);
    assert!(
        !warnings.is_empty(),
        "expected at least one warning for out-of-place doc comment"
    );
    let warning_text: Vec<String> = warnings.iter().map(|w| format!("{w}")).collect();
    insta::assert_snapshot!(warning_text.join("\n"));
}

/// Multiple out-of-place doc comments in a single file should each generate a
/// separate warning with correct line and column information.
#[test]
fn test_warning_multiple_out_of_place_doc_comments() {
    let input = r#"
        @namespace("test")
        protocol P {
            /** orphan 1 */
            record R {
                string name;
                /** orphan 2 */
            }
            /** orphan 3 */
        }
    "#;
    let warnings = parse_warnings(input);
    let warning_text: Vec<String> = warnings.iter().map(|w| format!("{w}")).collect();
    insta::assert_snapshot!(warning_text.join("\n---\n"));
}

// ==============================================================================
// Import Errors
// ==============================================================================

/// Importing a nonexistent file should produce an error during import
/// resolution. This tests the error message from `ImportContext::resolve_import`.
#[test]
fn test_error_import_nonexistent_file() {
    let input = r#"
        @namespace("test")
        protocol P {
            import schema "does_not_exist.avsc";
            record R { string name; }
        }
    "#;
    let (_idl_file, decl_items, _warnings) =
        parse_idl(input).expect("parsing the IDL text itself should succeed");

    let import_ctx = avdl::import::ImportContext::new(vec![]);
    let mut errors = Vec::new();

    for item in &decl_items {
        if let DeclItem::Import(import) = item {
            let result = import_ctx.resolve_import(&import.path, std::path::Path::new("."));
            if let Err(e) = result {
                errors.push(format!("{e}"));
            }
        }
    }

    assert!(
        !errors.is_empty(),
        "should produce an error for nonexistent import"
    );
    insta::assert_snapshot!(errors.join("\n"));
}

// ==============================================================================
// Mutation Error Tests (Slight Variations of Valid IDL)
// ==============================================================================

/// Semicolon replaced with a comma in a field declaration. This is a common
/// typo when switching between languages (e.g., Go struct syntax uses commas).
#[test]
fn test_error_semicolon_replaced_with_comma() {
    let input = "protocol P { record R { int x, } }";
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Missing semicolon between two field declarations. The parser should detect
/// that the second field's type name appears where a separator was expected.
#[test]
fn test_error_missing_field_separator() {
    let input = "protocol P { record R { int x int y; } }";
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Extra closing brace after the protocol. The parser should report that
/// unexpected input follows the valid protocol definition.
#[test]
fn test_error_extra_closing_brace() {
    let input = "protocol P { record R { int x; } } }";
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Missing record name — the opening brace appears where a name identifier
/// is expected.
#[test]
fn test_error_missing_record_name() {
    let input = "protocol P { record { int x; } }";
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Misspelled `protocol` keyword. The parser should fail because `protocl` is
/// not a recognized keyword or valid start of a schema/protocol declaration.
#[test]
fn test_error_misspelled_keyword() {
    let input = "protocl P { record R { int x; } }";
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Missing type for a field — just a bare identifier where a type name should
/// precede the field name.
#[test]
fn test_error_missing_field_type() {
    let input = "protocol P { record R { x; } }";
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Enum symbols without commas between them. Avro IDL enums require commas
/// separating symbols.
#[test]
fn test_error_missing_enum_commas() {
    let input = "protocol P { enum E { A B C } }";
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Semicolons used instead of commas in enum body — a common mistake when
/// confusing record field syntax with enum symbol syntax.
#[test]
fn test_error_semicolons_in_enum() {
    let input = "protocol P { enum E { A; B; C } }";
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Fixed declaration missing the size argument in parentheses. The parser
/// should report that a `(` or size is expected after the fixed name.
#[test]
fn test_error_missing_fixed_size() {
    let input = "protocol P { fixed F; }";
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Unclosed parenthesis in a fixed declaration — `(` is opened but `)` is
/// missing before the semicolon.
#[test]
fn test_error_unclosed_paren() {
    let input = "protocol P { fixed F(16; }";
    let error = parse_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}
