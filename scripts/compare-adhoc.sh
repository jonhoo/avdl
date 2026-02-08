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
#   --batch-json     Output JSON array to stdout (for automated processing).
#                    Runs both idl and idl2schemata for each file.
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
BATCH_JSON=false
TIMEOUT_SECS=30
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
        --batch-json)
            BATCH_JSON=true
            shift
            ;;
        --timeout)
            TIMEOUT_SECS="$2"
            shift 2
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
# Batch JSON mode — structured output for automated processing
# ==============================================================================

# Escape a string for JSON output (handles quotes, backslashes, newlines).
json_escape() {
    python3 -c "import json,sys; print(json.dumps(sys.stdin.read()))" <<< "$1"
}

# Run a command with timeout, capturing exit code, stdout, and stderr.
# Sets: _exit, _stdout, _stderr
run_with_timeout() {
    local out_file="$TMPDIR_BASE/_run_stdout"
    local err_file="$TMPDIR_BASE/_run_stderr"
    timeout "${TIMEOUT_SECS}s" "$@" >"$out_file" 2>"$err_file"
    _exit=$?
    _stdout="$(cat "$out_file")"
    _stderr="$(cat "$err_file")"
    # timeout returns 124 on timeout
    if [ "$_exit" -eq 124 ]; then
        _stderr="TIMEOUT after ${TIMEOUT_SECS}s"
    fi
}

batch_json_one_file() {
    local input_file="$1"
    local bn
    bn="$(basename "$input_file" .avdl)"

    local rust_out="$TMPDIR_BASE/${bn}-rust.json"
    local java_out="$TMPDIR_BASE/${bn}-java.json"

    # --- idl mode ---
    local idl_rust_exit=0 idl_java_exit=0
    local idl_rust_stderr="" idl_java_stderr=""
    local idl_result="unknown" idl_diff_snippet=""

    # shellcheck disable=SC2086
    run_with_timeout cargo run --quiet --manifest-path "$REPO_ROOT/Cargo.toml" \
        -- idl $RUST_FLAGS "$input_file" "$rust_out"
    idl_rust_exit=$_exit
    idl_rust_stderr="$_stderr"

    if [ "$idl_rust_exit" -eq 0 ] && [ -z "$AVRO_JAR" ]; then
        idl_result="rust-pass-java-unavailable"
    elif [ "$idl_rust_exit" -ne 0 ]; then
        # Rust failed, try Java to determine result category.
        if [ -n "$AVRO_JAR" ]; then
            run_with_timeout java -cp "$JAVA_CP" org.apache.avro.tool.Main idl "$input_file" "$java_out"
            idl_java_exit=$_exit
            idl_java_stderr="$_stderr"
            if [ "$idl_java_exit" -eq 0 ]; then
                idl_result="rust-fail"
            else
                idl_result="both-fail"
            fi
        else
            idl_result="rust-fail"
        fi
    else
        # Rust succeeded. Run Java.
        if [ -n "$AVRO_JAR" ]; then
            run_with_timeout java -cp "$JAVA_CP" org.apache.avro.tool.Main idl "$input_file" "$java_out"
            idl_java_exit=$_exit
            idl_java_stderr="$_stderr"
            if [ "$idl_java_exit" -ne 0 ]; then
                idl_result="java-fail"
            else
                # Both succeeded — compare.
                local rust_sorted="$TMPDIR_BASE/${bn}-rust-s.json"
                local java_sorted="$TMPDIR_BASE/${bn}-java-s.json"
                jq -S . "$rust_out" > "$rust_sorted" 2>/dev/null
                jq -S . "$java_out" > "$java_sorted" 2>/dev/null
                if diff -q "$rust_sorted" "$java_sorted" > /dev/null 2>&1; then
                    idl_result="both-pass-match"
                else
                    idl_result="both-pass-diff"
                    idl_diff_snippet="$(diff --unified=3 "$java_sorted" "$rust_sorted" | head -20)"
                fi
            fi
        fi
    fi

    # --- idl2schemata mode (only if both idl passes succeeded) ---
    local i2s_result="skipped" i2s_diff_snippet=""
    local i2s_rust_exit="" i2s_java_exit=""
    local i2s_rust_stderr="" i2s_java_stderr=""

    if [ "$idl_result" = "both-pass-match" ] || [ "$idl_result" = "both-pass-diff" ]; then
        local i2s_rust_dir="$TMPDIR_BASE/${bn}-i2s-rust"
        local i2s_java_dir="$TMPDIR_BASE/${bn}-i2s-java"
        mkdir -p "$i2s_rust_dir" "$i2s_java_dir"

        # shellcheck disable=SC2086
        run_with_timeout cargo run --quiet --manifest-path "$REPO_ROOT/Cargo.toml" \
            -- idl2schemata $RUST_FLAGS "$input_file" "$i2s_rust_dir"
        i2s_rust_exit=$_exit
        i2s_rust_stderr="$_stderr"

        if [ "$i2s_rust_exit" -ne 0 ]; then
            run_with_timeout java -cp "$JAVA_CP" org.apache.avro.tool.Main idl2schemata "$input_file" "$i2s_java_dir"
            i2s_java_exit=$_exit
            i2s_java_stderr="$_stderr"
            if [ "$i2s_java_exit" -eq 0 ]; then
                i2s_result="rust-fail"
            else
                i2s_result="both-fail"
            fi
        else
            run_with_timeout java -cp "$JAVA_CP" org.apache.avro.tool.Main idl2schemata "$input_file" "$i2s_java_dir"
            i2s_java_exit=$_exit
            i2s_java_stderr="$_stderr"
            if [ "$i2s_java_exit" -ne 0 ]; then
                i2s_result="java-fail"
            else
                # Compare all .avsc files.
                local has_diff=false
                local combined_diff=""
                for avsc in "$i2s_rust_dir"/*.avsc "$i2s_java_dir"/*.avsc; do
                    [ -f "$avsc" ] || continue
                    local avsc_name
                    avsc_name="$(basename "$avsc")"
                    local r="$i2s_rust_dir/$avsc_name"
                    local j="$i2s_java_dir/$avsc_name"
                    if [ -f "$r" ] && [ -f "$j" ]; then
                        local rs="$TMPDIR_BASE/${bn}-i2s-${avsc_name}-rs.json"
                        local js="$TMPDIR_BASE/${bn}-i2s-${avsc_name}-js.json"
                        jq -S . "$r" > "$rs" 2>/dev/null
                        jq -S . "$j" > "$js" 2>/dev/null
                        if ! diff -q "$rs" "$js" > /dev/null 2>&1; then
                            has_diff=true
                            combined_diff+="--- $avsc_name ---"$'\n'
                            combined_diff+="$(diff --unified=3 "$js" "$rs" | head -10)"$'\n'
                        fi
                    elif [ -f "$r" ] && [ ! -f "$j" ]; then
                        has_diff=true
                        combined_diff+="Rust-only: $avsc_name"$'\n'
                    elif [ ! -f "$r" ] && [ -f "$j" ]; then
                        has_diff=true
                        combined_diff+="Java-only: $avsc_name"$'\n'
                    fi
                done
                if [ "$has_diff" = true ]; then
                    i2s_result="both-pass-diff"
                    i2s_diff_snippet="$(echo "$combined_diff" | head -20)"
                else
                    i2s_result="both-pass-match"
                fi
            fi
        fi
    fi

    # Output JSON object for this file.
    # Using python3 to build proper JSON avoids quoting issues.
    python3 -c "
import json, sys
obj = {
    'file': sys.argv[1],
    'idl_result': sys.argv[2],
    'idl_rust_exit': int(sys.argv[3]),
    'idl_java_exit': int(sys.argv[4]),
    'idl_diff_snippet': sys.argv[5],
    'idl_rust_stderr': sys.argv[6],
    'idl_java_stderr': sys.argv[7],
    'i2s_result': sys.argv[8],
    'i2s_diff_snippet': sys.argv[9],
    'i2s_rust_stderr': sys.argv[10],
    'i2s_java_stderr': sys.argv[11],
}
print(json.dumps(obj))
" \
        "$input_file" \
        "$idl_result" \
        "$idl_rust_exit" \
        "${idl_java_exit:-0}" \
        "$idl_diff_snippet" \
        "$idl_rust_stderr" \
        "$idl_java_stderr" \
        "$i2s_result" \
        "$i2s_diff_snippet" \
        "${i2s_rust_stderr:-}" \
        "${i2s_java_stderr:-}"
}

# ==============================================================================
# Main loop
# ==============================================================================

if [ "$BATCH_JSON" = true ]; then
    # Batch JSON mode: output a JSON array, one object per file.
    echo "["
    first=true
    for file in "${FILES[@]}"; do
        if [ "$first" = true ]; then
            first=false
        else
            echo ","
        fi
        if [ ! -f "$file" ]; then
            python3 -c "import json; print(json.dumps({'file': '$file', 'idl_result': 'file-not-found', 'idl_rust_exit': -1, 'idl_java_exit': -1, 'idl_diff_snippet': '', 'idl_rust_stderr': 'file not found', 'idl_java_stderr': '', 'i2s_result': 'skipped', 'i2s_diff_snippet': '', 'i2s_rust_stderr': '', 'i2s_java_stderr': ''}))"
        else
            batch_json_one_file "$file"
        fi
    done
    echo
    echo "]"
    exit 0
fi

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
