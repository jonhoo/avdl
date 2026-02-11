// ==============================================================================
// Shared Test Helpers
// ==============================================================================
//
// Common utility functions used across multiple integration test files.
//
// Each test file that imports this module compiles its own copy, so not every
// function is used in every binary. Suppress the resulting dead_code warnings.
#![allow(dead_code)]
// Import this module in each test file with:
//
//     mod common;
//     use common::{normalize_crlf, render_diagnostic, render_diagnostics};

use std::fmt::Write;

use miette::{GraphicalReportHandler, GraphicalTheme};
use serde_json::Value;

/// Recursively replace `\r\n` with `\n` in all JSON string values. This is a
/// no-op on Linux/macOS; on Windows, Git checks out `.avdl` files with `\r\n`
/// line endings, which causes doc-comment strings to differ from the golden
/// `.avpr` files that use `\n`.
///
/// The Java test suite applies the same normalization before comparing output:
/// - `TestIdlReader.java:232` uses `output.replace("\r", "")`
/// - `TestIdlTool.readFileAsString` (lines 102-104) uses
///   `BufferedReader.lines().collect(joining("\n"))`, which strips `\r`.
pub fn normalize_crlf(value: Value) -> Value {
    match value {
        Value::String(s) => Value::String(s.replace("\r\n", "\n")),
        Value::Array(arr) => Value::Array(arr.into_iter().map(normalize_crlf).collect()),
        Value::Object(obj) => Value::Object(
            obj.into_iter()
                .map(|(k, v)| (k, normalize_crlf(v)))
                .collect(),
        ),
        other => other,
    }
}

/// Render a single diagnostic to a deterministic string for snapshot tests.
/// Uses non-unicode theme at 80 columns.
pub fn render_diagnostic(report: &miette::Report) -> String {
    let handler = GraphicalReportHandler::new_themed(GraphicalTheme::none()).with_width(80);
    let mut buf = String::new();
    handler
        .render_report(&mut buf, report.as_ref())
        .expect("render to String is infallible");
    buf
}

/// Render multiple diagnostics, separated by blank lines.
pub fn render_diagnostics(reports: &[miette::Report]) -> String {
    let mut buf = String::new();
    for (i, r) in reports.iter().enumerate() {
        if i > 0 {
            writeln!(buf).expect("write to String is infallible");
        }
        buf.push_str(&render_diagnostic(r));
    }
    buf
}
