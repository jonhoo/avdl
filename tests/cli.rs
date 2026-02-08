// ==============================================================================
// CLI Integration Tests: Exercise the `avdl` Binary via Subprocess
// ==============================================================================
//
// These tests run the compiled `avdl` binary as a subprocess using `assert_cmd`,
// verifying exit codes, stdout/stderr content, and output file creation. They
// complement the library-level integration tests in `integration.rs` by testing
// the full CLI surface (argument parsing, file I/O, error reporting).

use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;

const INPUT_DIR: &str = "avro/lang/java/idl/src/test/idl/input";
const CLASSPATH_DIR: &str = "avro/lang/java/idl/src/test/idl/putOnClassPath";

/// Helper to construct a `Command` for the `avdl` binary built by this crate.
#[allow(deprecated)] // cargo_bin() warns about custom build-dir; acceptable here
fn avdl_cmd() -> Command {
    Command::cargo_bin("avdl").expect("avdl binary should be built by cargo")
}

// ==============================================================================
// `idl` Subcommand Tests
// ==============================================================================

/// Run `avdl idl` on `simple.avdl` and verify the protocol JSON is written to
/// stdout with exit code 0.
#[test]
fn test_cli_idl_file_to_stdout() {
    avdl_cmd()
        .args(["idl", &format!("{INPUT_DIR}/simple.avdl")])
        .assert()
        .success()
        .stdout(predicates::str::contains("\"protocol\""));
}

/// Run `avdl idl` writing to a temp output file, then verify the file exists
/// and contains valid JSON.
#[test]
fn test_cli_idl_file_to_file() {
    let out_dir = PathBuf::from("tmp/cli-test-idl-file-to-file");
    fs::create_dir_all(&out_dir).expect("create test output directory");
    let out_path = out_dir.join("simple.avpr");

    avdl_cmd()
        .args([
            "idl",
            &format!("{INPUT_DIR}/simple.avdl"),
            out_path.to_str().expect("valid UTF-8 path"),
        ])
        .assert()
        .success();

    let content = fs::read_to_string(&out_path).expect("read output file");
    let json: serde_json::Value =
        serde_json::from_str(&content).expect("output should be valid JSON");
    assert_eq!(
        json.get("protocol").and_then(|v| v.as_str()),
        Some("Simple"),
        "output JSON should contain protocol name 'Simple'"
    );
}

/// Run `avdl idl` on `import.avdl` with --import-dir flags for both the input
/// directory and the classpath directory, verifying that imports resolve
/// correctly.
#[test]
fn test_cli_idl_import_dir() {
    avdl_cmd()
        .args([
            "idl",
            "--import-dir",
            INPUT_DIR,
            "--import-dir",
            CLASSPATH_DIR,
            &format!("{INPUT_DIR}/import.avdl"),
        ])
        .assert()
        .success();
}

/// Run `avdl idl` on a nonexistent file and verify a non-zero exit code with
/// a useful error message on stderr.
#[test]
fn test_cli_idl_nonexistent_file() {
    avdl_cmd()
        .args(["idl", "nonexistent.avdl"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("nonexistent.avdl"));
}

// ==============================================================================
// `idl2schemata` Subcommand Tests
// ==============================================================================

/// Run `avdl idl2schemata` on `simple.avdl` and verify that `.avsc` files are
/// created in the output directory.
#[test]
fn test_cli_idl2schemata() {
    let out_dir = PathBuf::from("tmp/cli-test-idl2schemata");
    // Clean up from any previous run to avoid stale files.
    let _ = fs::remove_dir_all(&out_dir);
    fs::create_dir_all(&out_dir).expect("create test output directory");

    avdl_cmd()
        .args([
            "idl2schemata",
            &format!("{INPUT_DIR}/simple.avdl"),
            out_dir.to_str().expect("valid UTF-8 path"),
        ])
        .assert()
        .success();

    // simple.avdl defines Kind (enum), Status (enum), TestRecord (record),
    // MD5 (fixed), and TestError (error). Each should produce a .avsc file.
    let expected_files = [
        "Kind.avsc",
        "Status.avsc",
        "TestRecord.avsc",
        "MD5.avsc",
        "TestError.avsc",
    ];
    for filename in &expected_files {
        let path = out_dir.join(filename);
        assert!(
            path.exists(),
            "expected {filename} to exist in output directory, got: {:?}",
            fs::read_dir(&out_dir).map(|entries| entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect::<Vec<_>>())
        );
    }
}

/// Run `avdl idl2schemata` with no arguments and verify a non-zero exit code,
/// since clap requires the input argument.
#[test]
fn test_cli_idl2schemata_missing_input() {
    avdl_cmd().args(["idl2schemata"]).assert().failure();
}

// ==============================================================================
// General CLI Tests
// ==============================================================================

/// Run `avdl --help` and verify exit code 0 with usage information.
#[test]
fn test_cli_help() {
    avdl_cmd()
        .args(["--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage"));
}
