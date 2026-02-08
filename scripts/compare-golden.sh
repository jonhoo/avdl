#!/usr/bin/env bash
# compare-golden.sh — Compare avdl Rust output against golden test files.
#
# Usage:
#   scripts/compare-golden.sh idl              # all 18 .avdl files
#   scripts/compare-golden.sh idl simple       # single file
#   scripts/compare-golden.sh idl2schemata     # all idl2schemata files
#   scripts/compare-golden.sh idl2schemata echo  # single file
#   scripts/compare-golden.sh types import     # show type names for a file
#
# Environment:
#   AVRO_TOOLS_JAR  Override the path to avro-tools-1.12.1.jar. When unset,
#                   the script searches ../avro-tools-1.12.1.jar relative to
#                   the repo root, then relative to the main worktree root.
#
# Exit code: 0 if all comparisons pass, 1 if any fail.

set -euo pipefail

# ==============================================================================
# Configuration
# ==============================================================================

# Resolve paths relative to the repository root (one level up from scripts/).
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

INPUT_DIR="$REPO_ROOT/avro/lang/java/idl/src/test/idl/input"
OUTPUT_DIR="$REPO_ROOT/avro/lang/java/idl/src/test/idl/output"
CLASSPATH_DIR="$REPO_ROOT/avro/lang/java/idl/src/test/idl/putOnClassPath"

# Ensure tmp/ exists for temp directories.
mkdir -p "$REPO_ROOT/tmp"

TMPDIR_BASE=$(mktemp -d "$REPO_ROOT/tmp/compare-golden.XXXXXX")
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
# Locate the Java avro-tools JAR
# ==============================================================================

# Search order:
#   1. AVRO_TOOLS_JAR environment variable (explicit override)
#   2. $REPO_ROOT/../avro-tools-1.12.1.jar (works from the main checkout)
#   3. Main worktree's sibling (via `git worktree list`, for worktrees)
find_avro_tools_jar() {
    # 1. Explicit override.
    if [ -n "${AVRO_TOOLS_JAR:-}" ] && [ -f "$AVRO_TOOLS_JAR" ]; then
        echo "$AVRO_TOOLS_JAR"
        return 0
    fi

    # 2. Relative to this repo root (works from the main checkout).
    local candidate="$REPO_ROOT/../avro-tools-1.12.1.jar"
    if [ -f "$candidate" ]; then
        echo "$candidate"
        return 0
    fi

    # 3. Locate the main worktree root and check its sibling.
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
# Helpers
# ==============================================================================

pass_count=0
fail_count=0

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

# Return the --import-dir flags needed for a given .avdl file.
import_flags_for() {
    local file="$1"
    case "$file" in
        import.avdl|nestedimport.avdl)
            echo "--import-dir $INPUT_DIR --import-dir $CLASSPATH_DIR"
            ;;
        baseball.avdl|schema_syntax_schema.avdl)
            echo "--import-dir $INPUT_DIR"
            ;;
        *)
            echo ""
            ;;
    esac
}

# Build a Java -cp classpath that includes the JAR and any import directories
# the given .avdl file needs. The Java tool resolves imports via classpath, not
# --import-dir flags, so we must translate import dirs into classpath entries.
java_classpath_for() {
    local jar="$1"
    local file="$2"
    local cp="$jar"
    case "$file" in
        import.avdl|nestedimport.avdl)
            cp="$cp:$INPUT_DIR:$CLASSPATH_DIR"
            ;;
        baseball.avdl|schema_syntax_schema.avdl)
            cp="$cp:$INPUT_DIR"
            ;;
    esac
    echo "$cp"
}

# Return the golden output filename and extension for a given .avdl file.
# Outputs two words: extension golden_filename
golden_for() {
    local file="$1"
    case "$file" in
        schema_syntax_schema.avdl)
            echo "avsc schema_syntax.avsc"
            ;;
        status_schema.avdl)
            echo "avsc status.avsc"
            ;;
        *)
            echo "avpr ${file%.avdl}.avpr"
            ;;
    esac
}

# ==============================================================================
# idl mode — compare `cargo run -- idl` output against golden .avpr/.avsc
# ==============================================================================

run_idl() {
    local filter="${1:-}"
    local files

    if [ -n "$filter" ]; then
        files=("$INPUT_DIR/${filter}.avdl")
        if [ ! -f "${files[0]}" ]; then
            echo "${RED}Error:${RESET} $INPUT_DIR/${filter}.avdl does not exist"
            exit 1
        fi
    else
        # Glob all .avdl files in input/.
        files=("$INPUT_DIR"/*.avdl)
    fi

    echo "${BOLD}idl comparison — ${#files[@]} file(s)${RESET}"
    echo

    for input_file in "${files[@]}"; do
        local basename
        basename="$(basename "$input_file")"

        # Determine golden file.
        local golden_info
        golden_info="$(golden_for "$basename")"
        local golden_file="$OUTPUT_DIR/${golden_info#* }"

        if [ ! -f "$golden_file" ]; then
            report_fail "$basename" "golden file not found: $golden_file"
            continue
        fi

        # Determine import flags.
        local flags
        flags="$(import_flags_for "$basename")"

        # Run the Rust tool.
        local rust_output="$TMPDIR_BASE/${basename%.avdl}.rust.json"
        # shellcheck disable=SC2086
        if ! cargo run --quiet --manifest-path "$REPO_ROOT/Cargo.toml" \
                -- idl $flags "$input_file" "$rust_output" 2>"$TMPDIR_BASE/${basename}.stderr"; then
            report_fail "$basename" "cargo run failed (see $TMPDIR_BASE/${basename}.stderr)"
            continue
        fi

        # Semantic comparison (jq -S sorts keys).
        local rust_sorted="$TMPDIR_BASE/${basename%.avdl}.rust-sorted.json"
        local golden_sorted="$TMPDIR_BASE/${basename%.avdl}.golden-sorted.json"
        jq -S . "$rust_output" > "$rust_sorted" 2>/dev/null
        jq -S . "$golden_file" > "$golden_sorted" 2>/dev/null

        if diff -q "$rust_sorted" "$golden_sorted" > /dev/null 2>&1; then
            # Semantic match — also check raw byte-level match.
            if diff -q "$rust_output" "$golden_file" > /dev/null 2>&1; then
                report_pass "$basename (semantic + byte-exact)"
            else
                report_pass "$basename (semantic match; byte-level diffs exist)"
            fi
        else
            report_fail "$basename" "semantic diff found"
            # Show a short diff excerpt.
            diff --unified=3 "$golden_sorted" "$rust_sorted" | head -30
            echo
        fi
    done

    echo
    summary
}

# ==============================================================================
# idl2schemata mode — compare per-schema .avsc output
# ==============================================================================

# Files commonly tested with idl2schemata.
# All 18 .avdl test inputs. Some produce no named types (so idl2schemata
# outputs nothing), but running them all ensures we catch regressions.
IDL2SCHEMATA_FILES=(
    baseball comments cycle echo forward_ref import interop
    leading_underscore mr_events namespaces nestedimport reservedwords
    schema_syntax_schema simple status_schema unicode union uuid
)

run_idl2schemata() {
    local filter="${1:-}"
    local files

    if [ -n "$filter" ]; then
        files=("$filter")
    else
        files=("${IDL2SCHEMATA_FILES[@]}")
    fi

    echo "${BOLD}idl2schemata comparison — ${#files[@]} file(s)${RESET}"
    echo

    for name in "${files[@]}"; do
        local input_file="$INPUT_DIR/${name}.avdl"
        if [ ! -f "$input_file" ]; then
            report_fail "$name" "input file not found: $input_file"
            continue
        fi

        local outdir="$TMPDIR_BASE/idl2schemata-${name}"
        mkdir -p "$outdir"

        # Determine import flags.
        local flags
        flags="$(import_flags_for "${name}.avdl")"

        # Run idl2schemata.
        # shellcheck disable=SC2086
        if ! cargo run --quiet --manifest-path "$REPO_ROOT/Cargo.toml" \
                -- idl2schemata $flags "$input_file" "$outdir" 2>"$TMPDIR_BASE/${name}-i2s.stderr"; then
            report_fail "$name (idl2schemata)" "cargo run failed"
            continue
        fi

        # List output files.
        local count
        count=$(find "$outdir" -name '*.avsc' | wc -l)
        echo "  ${YELLOW}INFO${RESET}  $name: produced $count .avsc file(s):"
        find "$outdir" -name '*.avsc' -printf '          %f\n' | sort

        # If Java tool is available, compare against it.
        if [ -n "$AVRO_JAR" ]; then
            local java_outdir="$TMPDIR_BASE/idl2schemata-${name}-java"
            mkdir -p "$java_outdir"
            # The Java tool resolves imports via classpath, not --import-dir,
            # so we build a classpath that includes the import directories.
            local java_cp
            java_cp="$(java_classpath_for "$AVRO_JAR" "${name}.avdl")"
            if java -cp "$java_cp" org.apache.avro.tool.Main idl2schemata "$input_file" "$java_outdir" 2>/dev/null; then
                # Enable nullglob so globs expand to nothing when no files match.
                local prev_nullglob
                prev_nullglob="$(shopt -p nullglob || true)"
                shopt -s nullglob

                # Compare each Rust .avsc against Java's.
                for avsc in "$outdir"/*.avsc; do
                    local avsc_name
                    avsc_name="$(basename "$avsc")"
                    local java_avsc="$java_outdir/$avsc_name"
                    if [ -f "$java_avsc" ]; then
                        if diff -q <(jq -S . "$avsc") <(jq -S . "$java_avsc") > /dev/null 2>&1; then
                            report_pass "$name/$avsc_name (vs Java)"
                        else
                            report_fail "$name/$avsc_name (vs Java)" "semantic diff"
                        fi
                    else
                        report_fail "$name/$avsc_name" "Java did not produce this file"
                    fi
                done
                # Check for files Java produced that Rust didn't.
                for avsc in "$java_outdir"/*.avsc; do
                    local avsc_name
                    avsc_name="$(basename "$avsc")"
                    if [ ! -f "$outdir/$avsc_name" ]; then
                        report_fail "$name/$avsc_name" "Rust did not produce this file (Java did)"
                    fi
                done

                # Restore previous nullglob state.
                eval "$prev_nullglob"

                # If both tools produced 0 files, that's still a pass.
                local rust_count java_count
                rust_count=$(find "$outdir" -name '*.avsc' 2>/dev/null | wc -l)
                java_count=$(find "$java_outdir" -name '*.avsc' 2>/dev/null | wc -l)
                if [ "$rust_count" -eq 0 ] && [ "$java_count" -eq 0 ]; then
                    report_pass "$name (both produced 0 schemas)"
                fi
            else
                echo "  ${YELLOW}INFO${RESET}  Java tool failed for $name — Rust-only results shown"
            fi
        else
            echo "  ${YELLOW}INFO${RESET}  Java tool not available — Rust-only results shown"
        fi
        echo
    done

    summary
}

# ==============================================================================
# types mode — extract type names from idl output in order
# ==============================================================================

run_types() {
    local filter="${1:?types mode requires a file argument (e.g., 'types import')}"
    local input_file="$INPUT_DIR/${filter}.avdl"

    if [ ! -f "$input_file" ]; then
        echo "${RED}Error:${RESET} $input_file does not exist"
        exit 1
    fi

    local flags
    flags="$(import_flags_for "${filter}.avdl")"

    local rust_output="$TMPDIR_BASE/${filter}.json"

    # shellcheck disable=SC2086
    if ! cargo run --quiet --manifest-path "$REPO_ROOT/Cargo.toml" \
            -- idl $flags "$input_file" "$rust_output" 2>/dev/null; then
        echo "${RED}Error:${RESET} cargo run failed for ${filter}.avdl"
        exit 1
    fi

    echo "${BOLD}Type names in ${filter}.avdl (in order):${RESET}"
    echo

    # Extract type names from the protocol JSON. Types appear in the "types"
    # array; each element has a "name" field (and optionally "namespace").
    jq -r '.types[]? | if .namespace then "\(.namespace).\(.name)" else .name end' "$rust_output"

    # Also show the golden file's types for comparison.
    local golden_info
    golden_info="$(golden_for "${filter}.avdl")"
    local golden_file="$OUTPUT_DIR/${golden_info#* }"

    if [ -f "$golden_file" ]; then
        echo
        echo "${BOLD}Golden type names (expected):${RESET}"
        echo
        jq -r '.types[]? | if .namespace then "\(.namespace).\(.name)" else .name end' "$golden_file"
    fi
}

# ==============================================================================
# Summary
# ==============================================================================

summary() {
    echo "---"
    echo "${BOLD}Summary:${RESET} ${GREEN}${pass_count} passed${RESET}, ${RED}${fail_count} failed${RESET}"

    if [ "$fail_count" -gt 0 ]; then
        exit 1
    fi
}

# ==============================================================================
# Main dispatch
# ==============================================================================

usage() {
    echo "Usage: $0 <mode> [file]"
    echo
    echo "Modes:"
    echo "  idl [file]           Compare idl output against golden .avpr/.avsc"
    echo "  idl2schemata [file]  Compare idl2schemata output"
    echo "  types <file>         Show type names in order for a file"
    echo
    echo "Examples:"
    echo "  $0 idl               # all 18 .avdl files"
    echo "  $0 idl simple        # just simple.avdl"
    echo "  $0 idl2schemata      # all idl2schemata files"
    echo "  $0 types import      # type order for import.avdl"
}

if [ $# -lt 1 ]; then
    usage
    exit 1
fi

mode="$1"
shift

case "$mode" in
    idl)
        run_idl "${1:-}"
        ;;
    idl2schemata)
        run_idl2schemata "${1:-}"
        ;;
    types)
        run_types "${1:-}"
        ;;
    -h|--help|help)
        usage
        ;;
    *)
        echo "${RED}Error:${RESET} unknown mode '$mode'"
        echo
        usage
        exit 1
        ;;
esac
