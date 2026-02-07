#!/usr/bin/env bash
# compare-adhoc.sh — Compare Rust and Java avro-tools output for ad-hoc .avdl files.
#
# Compiles one or more .avdl files through both the Rust tool and the Java
# avro-tools JAR, then performs a semantic (jq -S) comparison of the JSON
# output. Useful for testing edge cases beyond the golden test suite.
#
# Usage:
#   scripts/compare-adhoc.sh tmp/edge-case.avdl
#   scripts/compare-adhoc.sh tmp/edge-*.avdl
#   scripts/compare-adhoc.sh --idl2schemata tmp/edge-case.avdl
#
# Options:
#   --idl2schemata   Use idl2schemata mode instead of idl mode
#   --import-dir D   Pass --import-dir to the Rust tool (and add to Java classpath)
#   --show-output    Print the Rust and Java JSON output for diffs (not just the diff)
#   --rust-only      Only run the Rust tool (skip Java comparison)
#
# Environment:
#   AVRO_TOOLS_JAR   Override the path to avro-tools-1.12.1.jar. When unset,
#                    searched relative to the repo root (same logic as
#                    compare-golden.sh).
#
# Exit code: 0 if all comparisons pass, 1 if any fail.

set -euo pipefail

# ==============================================================================
# Configuration
# ==============================================================================

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

mkdir -p "$REPO_ROOT/tmp"
TMPDIR_BASE=$(mktemp -d "$REPO_ROOT/tmp/compare-adhoc.XXXXXX")
trap 'rm -rf "$TMPDIR_BASE"' EXIT

# ==============================================================================
# Color output (only when stdout is a tty)
# ==============================================================================

if [ -t 1 ]; then
    GREEN=$'\033[32m'
    RED=$'\033[31m'
    YELLOW=$'\033[33m'
    BOLD=$'\033[1m'
    RESET=$'\033[0m'
else
    GREEN=""
    RED=""
    YELLOW=""
    BOLD=""
    RESET=""
fi

# ==============================================================================
# Locate the Java avro-tools JAR (same logic as compare-golden.sh)
# ==============================================================================

find_avro_tools_jar() {
    if [ -n "${AVRO_TOOLS_JAR:-}" ] && [ -f "$AVRO_TOOLS_JAR" ]; then
        echo "$AVRO_TOOLS_JAR"
        return 0
    fi

    local candidate="$REPO_ROOT/../avro-tools-1.12.1.jar"
    if [ -f "$candidate" ]; then
        echo "$candidate"
        return 0
    fi

    if command -v git >/dev/null 2>&1; then
        local main_worktree
        main_worktree="$(git -C "$REPO_ROOT" worktree list --porcelain 2>/dev/null \
            | head -1 | sed 's/^worktree //')"
        if [ -n "$main_worktree" ]; then
            candidate="$main_worktree/../avro-tools-1.12.1.jar"
            if [ -f "$candidate" ]; then
                echo "$candidate"
                return 0
            fi
        fi
    fi

    return 1
}

AVRO_JAR="$(find_avro_tools_jar)" || AVRO_JAR=""

# ==============================================================================
# Parse arguments
# ==============================================================================

MODE="idl"
IMPORT_DIRS=()
SHOW_OUTPUT=false
RUST_ONLY=false
FILES=()

while [ $# -gt 0 ]; do
    case "$1" in
        --idl2schemata)
            MODE="idl2schemata"
            shift
            ;;
        --import-dir)
            IMPORT_DIRS+=("$2")
            shift 2
            ;;
        --show-output)
            SHOW_OUTPUT=true
            shift
            ;;
        --rust-only)
            RUST_ONLY=true
            shift
            ;;
        -h|--help)
            sed -n '2,/^$/{ s/^# \?//; p }' "$0"
            exit 0
            ;;
        -*)
            echo "${RED}Error:${RESET} unknown option '$1'"
            exit 1
            ;;
        *)
            FILES+=("$1")
            shift
            ;;
    esac
done

if [ ${#FILES[@]} -eq 0 ]; then
    echo "${RED}Error:${RESET} no .avdl files specified"
    echo "Usage: $0 [options] file.avdl [file2.avdl ...]"
    exit 1
fi

# ==============================================================================
# Build flag strings
# ==============================================================================

RUST_FLAGS=""
for dir in "${IMPORT_DIRS[@]+"${IMPORT_DIRS[@]}"}"; do
    RUST_FLAGS="$RUST_FLAGS --import-dir $dir"
done

JAVA_CP="$AVRO_JAR"
for dir in "${IMPORT_DIRS[@]+"${IMPORT_DIRS[@]}"}"; do
    JAVA_CP="$JAVA_CP:$dir"
done

# ==============================================================================
# Comparison logic
# ==============================================================================

pass_count=0
fail_count=0
skip_count=0

report_pass() {
    echo "  ${GREEN}PASS${RESET}  $1"
    : $((pass_count += 1))
}

report_fail() {
    echo "  ${RED}FAIL${RESET}  $1"
    if [ -n "${2:-}" ]; then
        echo "        $2"
    fi
    : $((fail_count += 1))
}

report_skip() {
    echo "  ${YELLOW}SKIP${RESET}  $1${2:+ ($2)}"
    : $((skip_count += 1))
}

compare_idl() {
    local input_file="$1"
    local basename
    basename="$(basename "$input_file" .avdl)"

    local rust_output="$TMPDIR_BASE/${basename}-rust.json"
    local java_output="$TMPDIR_BASE/${basename}-java.json"

    # Run Rust tool.
    # shellcheck disable=SC2086
    if ! cargo run --quiet --manifest-path "$REPO_ROOT/Cargo.toml" \
            -- idl $RUST_FLAGS "$input_file" "$rust_output" 2>"$TMPDIR_BASE/${basename}-rust.stderr"; then
        report_fail "$basename" "Rust tool failed"
        if [ "$SHOW_OUTPUT" = true ]; then
            echo "        $(cat "$TMPDIR_BASE/${basename}-rust.stderr" | head -5)"
        fi
        return
    fi

    # Optionally skip Java.
    if [ "$RUST_ONLY" = true ]; then
        report_skip "$basename" "Rust-only mode"
        if [ "$SHOW_OUTPUT" = true ]; then
            jq . "$rust_output"
        fi
        return
    fi

    # Check Java availability.
    if [ -z "$AVRO_JAR" ]; then
        report_skip "$basename" "Java tool not available"
        return
    fi

    # Run Java tool.
    if ! java -cp "$JAVA_CP" org.apache.avro.tool.Main idl "$input_file" "$java_output" 2>"$TMPDIR_BASE/${basename}-java.stderr"; then
        # Java failed but Rust succeeded -- behavioral difference.
        local java_err
        java_err="$(head -1 "$TMPDIR_BASE/${basename}-java.stderr")"
        report_fail "$basename" "Rust succeeded, Java failed: $java_err"
        if [ "$SHOW_OUTPUT" = true ]; then
            echo "  Rust output:"
            jq . "$rust_output"
        fi
        return
    fi

    # Semantic comparison.
    local rust_sorted="$TMPDIR_BASE/${basename}-rust-sorted.json"
    local java_sorted="$TMPDIR_BASE/${basename}-java-sorted.json"
    jq -S . "$rust_output" > "$rust_sorted" 2>/dev/null
    jq -S . "$java_output" > "$java_sorted" 2>/dev/null

    if diff -q "$rust_sorted" "$java_sorted" > /dev/null 2>&1; then
        report_pass "$basename"
    else
        report_fail "$basename" "semantic diff"
        diff --unified=3 "$java_sorted" "$rust_sorted" | head -40
        if [ "$SHOW_OUTPUT" = true ]; then
            echo
            echo "  Rust:"
            jq . "$rust_output"
            echo "  Java:"
            jq . "$java_output"
        fi
    fi
}

compare_idl2schemata() {
    local input_file="$1"
    local basename
    basename="$(basename "$input_file" .avdl)"

    local rust_outdir="$TMPDIR_BASE/${basename}-i2s-rust"
    local java_outdir="$TMPDIR_BASE/${basename}-i2s-java"
    mkdir -p "$rust_outdir" "$java_outdir"

    # Run Rust tool.
    # shellcheck disable=SC2086
    if ! cargo run --quiet --manifest-path "$REPO_ROOT/Cargo.toml" \
            -- idl2schemata $RUST_FLAGS "$input_file" "$rust_outdir" 2>"$TMPDIR_BASE/${basename}-i2s-rust.stderr"; then
        report_fail "$basename (idl2schemata)" "Rust tool failed"
        return
    fi

    local rust_files
    rust_files=$(find "$rust_outdir" -name '*.avsc' -printf '%f\n' | sort)
    echo "  ${YELLOW}INFO${RESET}  $basename: Rust produced $(echo "$rust_files" | wc -w) .avsc file(s)"

    if [ "$RUST_ONLY" = true ] || [ -z "$AVRO_JAR" ]; then
        report_skip "$basename (idl2schemata)" "${RUST_ONLY:+Rust-only}${AVRO_JAR:+Java unavailable}"
        return
    fi

    # Run Java tool.
    if ! java -cp "$JAVA_CP" org.apache.avro.tool.Main idl2schemata "$input_file" "$java_outdir" 2>/dev/null; then
        report_fail "$basename (idl2schemata)" "Java tool failed"
        return
    fi

    # Compare each .avsc file.
    for avsc in "$rust_outdir"/*.avsc; do
        local avsc_name
        avsc_name="$(basename "$avsc")"
        local java_avsc="$java_outdir/$avsc_name"
        if [ -f "$java_avsc" ]; then
            if diff -q <(jq -S . "$avsc") <(jq -S . "$java_avsc") > /dev/null 2>&1; then
                report_pass "$basename/$avsc_name"
            else
                report_fail "$basename/$avsc_name" "semantic diff"
                diff --unified=3 <(jq -S . "$java_avsc") <(jq -S . "$avsc") | head -30
            fi
        else
            report_fail "$basename/$avsc_name" "Java did not produce this file"
        fi
    done

    # Check for files Java produced that Rust didn't.
    for avsc in "$java_outdir"/*.avsc; do
        [ -f "$avsc" ] || continue
        local avsc_name
        avsc_name="$(basename "$avsc")"
        if [ ! -f "$rust_outdir/$avsc_name" ]; then
            report_fail "$basename/$avsc_name" "Rust did not produce this file (Java did)"
        fi
    done
}

# ==============================================================================
# Main loop
# ==============================================================================

echo "${BOLD}${MODE} comparison — ${#FILES[@]} file(s)${RESET}"
echo

for file in "${FILES[@]}"; do
    if [ ! -f "$file" ]; then
        report_fail "$(basename "$file")" "file not found: $file"
        continue
    fi

    case "$MODE" in
        idl)          compare_idl "$file" ;;
        idl2schemata) compare_idl2schemata "$file" ;;
    esac
done

echo
echo "---"
echo "${BOLD}Summary:${RESET} ${GREEN}${pass_count} passed${RESET}, ${RED}${fail_count} failed${RESET}, ${YELLOW}${skip_count} skipped${RESET}"

if [ "$fail_count" -gt 0 ]; then
    exit 1
fi
