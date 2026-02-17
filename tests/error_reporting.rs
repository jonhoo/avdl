// ==============================================================================
// Error Reporting Snapshot Tests
// ==============================================================================
//
// These tests verify the *content* and *quality* of error messages produced by
// malformed or invalid `.avdl` inputs. Each test feeds an inline `.avdl` string
// to the builder API, captures the rendered error output, and snapshots it with
// `insta` so that changes to error messages are reviewed explicitly.
//
// The helper functions render errors through `miette`'s `GraphicalReportHandler`
// (with Unicode and color disabled for reproducible snapshots) so that we test
// what the user actually sees, including source spans and labels.

mod common;

use avdl::Idl;
use common::render_diagnostics;
use miette::{Diagnostic, GraphicalReportHandler, GraphicalTheme};

// ==============================================================================
// Test Helpers
// ==============================================================================

/// Compile an inline `.avdl` string through the builder and return the rendered
/// error message.
///
/// Uses `miette`'s `GraphicalReportHandler` to render rich diagnostics (with
/// source spans and labels) when available, falling back to plain `Display`
/// output for errors without source location info.
///
/// Returns `None` if compilation succeeds.
fn compile_error(input: &str) -> Option<String> {
    compile_error_with_width(input, 80)
}

/// Like [`compile_error`], but allows overriding the miette rendering width.
///
/// Useful when error messages embed absolute paths that would be split across
/// lines at the default 80-column width, making post-processing (e.g., CWD
/// redaction) unreliable.
fn compile_error_with_width(input: &str, width: usize) -> Option<String> {
    match Idl::new().convert_str(input) {
        Ok(_) => None,
        Err(e) => {
            let mut buf = String::new();
            let handler =
                GraphicalReportHandler::new_themed(GraphicalTheme::none()).with_width(width);

            let diag: &dyn Diagnostic = e.as_ref();
            if diag.source_code().is_some()
                && handler.render_report(&mut buf, diag).is_ok()
            {
                return Some(buf);
            }

            Some(format!("{e}"))
        }
    }
}

/// Compile a `.avdl` file from disk through the builder and return the
/// rendered error message.
///
/// Like [`compile_error`], but reads from a file path so that imports resolve
/// relative to the file's directory. Uses a wide rendering width to avoid
/// line-wrapping absolute paths that would break redaction.
fn compile_file_error(path: &std::path::Path) -> Option<String> {
    match Idl::new().convert(path) {
        Ok(_) => None,
        Err(e) => {
            let mut buf = String::new();
            let handler =
                GraphicalReportHandler::new_themed(GraphicalTheme::none()).with_width(300);

            let diag: &dyn Diagnostic = e.as_ref();
            if diag.source_code().is_some()
                && handler.render_report(&mut buf, diag).is_ok()
            {
                return Some(buf);
            }

            Some(format!("{e}"))
        }
    }
}

/// Compile an inline `.avdl` string and return warnings from a successful
/// compilation.
///
/// Panics if compilation fails, since warning tests require a successful parse.
fn compile_warnings(input: &str) -> Vec<miette::Report> {
    let output = Idl::new()
        .convert_str(input)
        .expect("warning test input should compile successfully");
    output.warnings
}

// ==============================================================================
// Syntax Errors (ANTLR Parse Failures)
// ==============================================================================

/// Missing semicolon after a field declaration.
#[test]
fn test_error_missing_semicolon() {
    let input = "protocol P { record R { int x } }";
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Missing closing brace for the protocol.
#[test]
fn test_error_unclosed_brace() {
    let input = "protocol P { record R { int x; }";
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Malformed union with a double comma.
#[test]
fn test_error_malformed_union() {
    let input = "protocol P { record R { union { int, , string } x; } }";
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Invalid token in type position.
#[test]
fn test_error_invalid_token_in_type_position() {
    let input = "protocol P { record R { 123 x; } }";
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Malformed fixed declaration with a non-numeric size argument.
#[test]
fn test_error_malformed_fixed_size() {
    let input = "protocol P { fixed F(not_a_number); }";
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

// ==============================================================================
// Semantic / Validation Errors
// ==============================================================================

/// Two records with the same fully-qualified name should produce an error.
#[test]
fn test_error_duplicate_type_name() {
    let input = r#"
        @namespace("org.test")
        protocol P {
            record Dup { string name; }
            record Dup { int id; }
        }
    "#;
    let error = compile_error(input).expect("should produce a registration error");
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
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Nested unions are forbidden by the Avro specification.
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
    let error = compile_error(input).expect("should produce an error");
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
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// A type name that collides with an Avro built-in type should be rejected.
#[test]
fn test_error_reserved_type_name() {
    let input = r#"record `int` { string value; }"#;
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

/// Referencing an undefined type should produce an error.
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
    let error = compile_error(input).expect("should produce an error for undefined type");
    insta::assert_snapshot!(error);
}

// ==============================================================================
// Warning Tests
// ==============================================================================

/// A doc comment before a closing brace should produce a warning.
#[test]
fn test_warning_out_of_place_doc_comment() {
    let input = r#"
        @namespace("test")
        protocol P {
            record R {
                string name;
                /** This doc comment is orphaned — nothing follows it. */
            }
        }
    "#;
    let warnings = compile_warnings(input);
    assert!(
        !warnings.is_empty(),
        "expected at least one warning for out-of-place doc comment"
    );
    insta::assert_snapshot!(render_diagnostics(&warnings));
}

/// Multiple out-of-place doc comments should each generate a separate warning.
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
    let warnings = compile_warnings(input);
    insta::assert_snapshot!(render_diagnostics(&warnings));
}

// ==============================================================================
// Import Errors
// ==============================================================================

/// Importing a nonexistent file should produce an error.
#[test]
#[cfg_attr(windows, ignore)]
fn test_error_import_nonexistent_file() {
    let input = r#"
        @namespace("test")
        protocol P {
            import schema "does_not_exist.avsc";
            record R { string name; }
        }
    "#;
    // Use a wide rendering width so miette does not line-wrap the absolute CWD path that
    // appears in the error message, which would prevent `str::replace` from matching it.
    let error = compile_error_with_width(input, 300)
        .expect("should produce an error for nonexistent import");

    // Redact the absolute CWD path so the snapshot is portable across machines and worktrees.
    let cwd = std::env::current_dir().expect("current_dir is available during tests");
    let error = error.replace(&cwd.display().to_string(), "[CWD]");
    insta::assert_snapshot!(error);
}

/// Importing a `.avsc` file with invalid JSON should produce an error that
/// includes the source span of the `import` statement in the calling `.avdl`
/// file.
#[test]
#[cfg_attr(windows, ignore)]
fn test_error_import_bad_avsc_json() {
    let avdl_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/testdata/import_bad_avsc.avdl");
    let error = compile_file_error(&avdl_path).expect("should produce an error for bad .avsc");

    // Redact the absolute path prefix so the snapshot is portable.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let error = error.replace(manifest_dir, "[ROOT]");
    insta::assert_snapshot!(error);
}

/// Importing a `.avpr` file with invalid JSON should produce an error that
/// includes the source span of the `import` statement in the calling `.avdl`
/// file.
#[test]
#[cfg_attr(windows, ignore)]
fn test_error_import_bad_avpr_json() {
    let avdl_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/testdata/import_bad_avpr.avdl");
    let error = compile_file_error(&avdl_path).expect("should produce an error for bad .avpr");

    // Redact the absolute path prefix so the snapshot is portable.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let error = error.replace(manifest_dir, "[ROOT]");
    insta::assert_snapshot!(error);
}

// ==============================================================================
// Mutation Error Tests (Slight Variations of Valid IDL)
// ==============================================================================

#[test]
fn test_error_semicolon_replaced_with_comma() {
    let input = "protocol P { record R { int x, } }";
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

#[test]
fn test_error_missing_field_separator() {
    let input = "protocol P { record R { int x int y; } }";
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

#[test]
fn test_error_extra_closing_brace() {
    let input = "protocol P { record R { int x; } } }";
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

#[test]
fn test_error_missing_record_name() {
    let input = "protocol P { record { int x; } }";
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

#[test]
fn test_error_misspelled_keyword() {
    let input = "protocl P { record R { int x; } }";
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

#[test]
fn test_error_missing_field_type() {
    let input = "protocol P { record R { x; } }";
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

#[test]
fn test_error_missing_enum_commas() {
    let input = "protocol P { enum E { A B C } }";
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

#[test]
fn test_error_semicolons_in_enum() {
    let input = "protocol P { enum E { A; B; C } }";
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

#[test]
fn test_error_missing_fixed_size() {
    let input = "protocol P { fixed F; }";
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

#[test]
fn test_error_unclosed_paren() {
    let input = "protocol P { fixed F(16; }";
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

// ==============================================================================
// Additional Semantic Error Tests
// ==============================================================================

#[test]
fn test_error_duplicate_message_param() {
    let input = r#"
        @namespace("test")
        protocol P {
            record Msg { string text; }
            void test(Msg x, Msg x);
        }
    "#;
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

#[test]
fn test_error_empty_idl_file() {
    let input = "/* nothing */";
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

#[test]
fn test_error_invalid_order_value() {
    let input = r#"
        @namespace("test")
        protocol P {
            record R {
                string @order("BAD") x;
            }
        }
    "#;
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

#[test]
fn test_error_invalid_aliases_type() {
    let input = r#"
        @namespace("test")
        protocol P {
            @aliases(123) record R { string name; }
        }
    "#;
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

#[test]
fn test_error_oneway_nonvoid() {
    let input = r#"
        @namespace("test")
        protocol P {
            record Msg { string text; }
            Msg send(Msg m) oneway;
        }
    "#;
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}

#[test]
fn test_error_annotated_type_reference() {
    let input = r#"
        @namespace("test")
        protocol P {
            fixed MD5(16);
            record R {
                @foo("bar") MD5 hash;
            }
        }
    "#;
    let error = compile_error(input).expect("should produce an error");
    insta::assert_snapshot!(error);
}
