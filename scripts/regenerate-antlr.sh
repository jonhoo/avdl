#!/usr/bin/env bash
# regenerate-antlr.sh — Regenerate the ANTLR parser/lexer from Idl.g4.
#
# The generated Rust files in src/generated/ are checked in so that
# downstream users don't need Java or Maven. This script is only needed
# when the grammar (Idl.g4) changes or the antlr4rust submodule is
# updated.
#
# Prerequisites:
#   - Java (tested with 21)
#   - Maven (only if --rebuild-jar is used)
#   - The antlr4rust submodule must be initialized. This is a fork of
#     ANTLR4 that adds Rust target support — the upstream ANTLR4 project
#     does not support Rust. The pre-built JAR in the submodule handles
#     the common case; Maven is only needed to rebuild it.
#
# Usage:
#   scripts/regenerate-antlr.sh                # regenerate using existing JAR
#   scripts/regenerate-antlr.sh --rebuild-jar  # rebuild JAR from source first

set -euo pipefail

# ==============================================================================
# Configuration
# ==============================================================================

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

GRAMMAR="$REPO_ROOT/avro/share/idl_grammar/org/apache/avro/idl/Idl.g4"
GRAMMAR_DIR="$(dirname "$GRAMMAR")"
JAR="$REPO_ROOT/antlr4rust/tool/target/antlr4-4.13.3-SNAPSHOT-complete.jar"
GENERATED_DIR="$REPO_ROOT/src/generated"

# The four .rs files ANTLR generates from the grammar.
GENERATED_FILES=(idlparser.rs idllexer.rs idllistener.rs idlbaselistener.rs)

# ==============================================================================
# Parse arguments
# ==============================================================================

rebuild_jar=false

for arg in "$@"; do
    case "$arg" in
        --rebuild-jar)
            rebuild_jar=true
            ;;
        -h|--help)
            sed -n '2,/^$/{ s/^# //; s/^#$//; p }' "$0"
            exit 0
            ;;
        *)
            echo "Unknown argument: $arg" >&2
            echo "Usage: $0 [--rebuild-jar]" >&2
            exit 1
            ;;
    esac
done

# ==============================================================================
# Optionally rebuild the ANTLR JAR
# ==============================================================================

if [ "$rebuild_jar" = true ]; then
    echo "==> Rebuilding ANTLR JAR from antlr4rust submodule..."
    (cd "$REPO_ROOT/antlr4rust" && MAVEN_OPTS="-Xmx1G" mvn package -DskipTests)
    echo
fi

# ==============================================================================
# Validate prerequisites
# ==============================================================================

if [ ! -f "$JAR" ]; then
    echo "Error: ANTLR JAR not found at $JAR" >&2
    echo "Either initialize the antlr4rust submodule or run with --rebuild-jar." >&2
    exit 1
fi

if ! command -v java &>/dev/null; then
    echo "Error: java not found in PATH." >&2
    exit 1
fi

if [ ! -f "$GRAMMAR" ]; then
    echo "Error: Grammar file not found at $GRAMMAR" >&2
    echo "Make sure the avro submodule is initialized." >&2
    exit 1
fi

# ==============================================================================
# Run the ANTLR generator
# ==============================================================================

echo "==> Generating Rust parser/lexer from Idl.g4..."
java -jar "$JAR" -Dlanguage=Rust "$GRAMMAR"

# ==============================================================================
# Copy generated files into src/generated/
#
# The generated files include their own #![allow(...)] inner attributes
# to suppress warnings. mod.rs adds outer #[allow(...)] for the few warnings
# the generator doesn't cover (clippy, unused_parens, unused_variables).
#
# The generator embeds the absolute grammar path in a comment. We normalize
# it to a relative path for reproducibility. The comment may be on line 1
# or line 2 (some files have #![allow] on line 1), so we replace globally.
# ==============================================================================

echo "==> Copying generated files to $GENERATED_DIR..."

# Path prefix to strip from the generated comment (convert absolute to relative).
GRAMMAR_REL="avro/share/idl_grammar/org/apache/avro/idl/Idl.g4"

for file in "${GENERATED_FILES[@]}"; do
    src="$GRAMMAR_DIR/$file"
    dst="$GENERATED_DIR/$file"

    if [ ! -f "$src" ]; then
        echo "Error: Expected generated file not found: $src" >&2
        exit 1
    fi

    # Replace the absolute path in the generated comment with the relative one.
    sed "s|$GRAMMAR|$GRAMMAR_REL|" "$src" > "$dst"
    echo "    $file"
done

# ==============================================================================
# Clean up generated artifacts next to the grammar
# ==============================================================================

echo "==> Cleaning up artifacts next to grammar..."
rm -f "$GRAMMAR_DIR"/*.rs "$GRAMMAR_DIR"/*.interp "$GRAMMAR_DIR"/*.tokens

# ==============================================================================
# Smoke test
# ==============================================================================

echo "==> Running cargo build as smoke test..."
cargo build --manifest-path "$REPO_ROOT/Cargo.toml"

echo
echo "Done. Generated files updated in src/generated/."
