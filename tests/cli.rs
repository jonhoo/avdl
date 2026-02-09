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
use serde_json::Value;

const INPUT_DIR: &str = "avro/lang/java/idl/src/test/idl/input";
const OUTPUT_DIR: &str = "avro/lang/java/idl/src/test/idl/output";
const CLASSPATH_DIR: &str = "avro/lang/java/idl/src/test/idl/putOnClassPath";

/// Helper to construct a `Command` for the `avdl` binary built by this crate.
#[allow(deprecated)] // cargo_bin() warns about custom build-dir; acceptable here
fn avdl_cmd() -> Command {
    Command::cargo_bin("avdl").expect("avdl binary should be built by cargo")
}

/// Load the golden `.avpr` output file and parse it as JSON for semantic
/// comparison.
fn load_golden(filename: &str) -> Value {
    let path = PathBuf::from(OUTPUT_DIR).join(filename);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read golden file {}: {e}", path.display()));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("failed to parse golden JSON {}: {e}", path.display()))
}

// ==============================================================================
// `idl` Subcommand Tests
// ==============================================================================

/// Run `avdl idl` on `simple.avdl` and verify the protocol JSON written to
/// stdout is semantically identical to the golden `.avpr` file.
#[test]
fn test_cli_idl_file_to_stdout() {
    let output = avdl_cmd()
        .args(["idl", &format!("{INPUT_DIR}/simple.avdl")])
        .output()
        .expect("run avdl idl");
    assert!(output.status.success(), "avdl idl should exit 0");

    let actual: Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let expected = load_golden("simple.avpr");
    assert_eq!(
        actual, expected,
        "CLI stdout should match golden simple.avpr"
    );
}

/// Run `avdl idl` writing to a temp output file, then verify the file is
/// semantically identical to the golden `.avpr` file.
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
    let actual: Value = serde_json::from_str(&content).expect("output file should be valid JSON");
    let expected = load_golden("simple.avpr");
    assert_eq!(
        actual, expected,
        "output file should match golden simple.avpr"
    );
}

/// Run `avdl idl` on `import.avdl` with `--import-dir` flags for both the input
/// directory and the classpath directory, verifying that imports resolve
/// correctly and the output matches the golden file.
#[test]
fn test_cli_idl_import_dir() {
    let output = avdl_cmd()
        .args([
            "idl",
            "--import-dir",
            INPUT_DIR,
            "--import-dir",
            CLASSPATH_DIR,
            &format!("{INPUT_DIR}/import.avdl"),
        ])
        .output()
        .expect("run avdl idl with --import-dir");
    assert!(
        output.status.success(),
        "avdl idl with --import-dir should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let actual: Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    let expected = load_golden("import.avpr");
    assert_eq!(
        actual, expected,
        "CLI stdout should match golden import.avpr"
    );
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
/// created in the output directory with valid JSON content.
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
        // Each file should contain valid JSON.
        let content =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {filename}: {e}"));
        let json: Value = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("{filename} should be valid JSON: {e}"));
        // Every schema file should have a "type" key and a "name" key.
        assert!(
            json.get("type").is_some(),
            "{filename} should have a \"type\" key"
        );
        assert!(
            json.get("name").is_some(),
            "{filename} should have a \"name\" key"
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

/// Run `avdl` with no arguments and verify it exits with a non-zero code and
/// prints a usage hint to stderr.
#[test]
fn test_cli_no_subcommand() {
    avdl_cmd()
        .assert()
        .failure()
        .stderr(predicates::str::contains("subcommand is required"))
        .stderr(predicates::str::contains("Usage"));
}

/// Run `avdl` with an unknown subcommand and verify it exits with a non-zero
/// code and prints the unrecognized name to stderr.
#[test]
fn test_cli_unknown_subcommand() {
    avdl_cmd()
        .args(["bogus"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("bogus"))
        .stderr(predicates::str::contains("Usage"));
}
