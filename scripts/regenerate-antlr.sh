#!/usr/bin/env bash
# regenerate-antlr.sh — Regenerate the ANTLR parser/lexer from Idl.g4.
#
# The generated Rust files in src/generated/ are checked in so that
# downstream users don't need Java. This script is only needed when
# the grammar (Idl.g4) changes.
#
# The ANTLR tool JAR is downloaded from the antlr4rust fork's GitHub
# release and cached locally in tmp/antlr4-tool.jar. It is downloaded
# automatically on first run.
#
# Prerequisites:
#   - Java (tested with 21)
#
# Usage:
#   scripts/regenerate-antlr.sh

set -euo pipefail

# ==============================================================================
# Configuration
# ==============================================================================

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

GRAMMAR="$REPO_ROOT/avro/share/idl_grammar/org/apache/avro/idl/Idl.g4"
GRAMMAR_DIR="$(dirname "$GRAMMAR")"
GENERATED_DIR="$REPO_ROOT/src/generated"
JAR_URL="https://github.com/antlr4rust/antlr4/releases/download/v0.5.0/antlr4-4.13.3-SNAPSHOT-complete.jar"
JAR="$REPO_ROOT/tmp/antlr4-tool.jar"

# The four .rs files ANTLR generates from the grammar.
GENERATED_FILES=(idlparser.rs idllexer.rs idllistener.rs idlbaselistener.rs)

# ==============================================================================
# Validate prerequisites
# ==============================================================================

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
# Download the ANTLR tool JAR if not cached
# ==============================================================================

if [ ! -f "$JAR" ]; then
    mkdir -p "$(dirname "$JAR")"
    echo "==> Downloading ANTLR tool JAR..."
    curl -fSL -o "$JAR" "$JAR_URL"
fi

# ==============================================================================
# Run the ANTLR generator
#
# We use -Xexact-output-dir so files are written directly into
# src/generated/ rather than next to the grammar file.
# ==============================================================================

echo "==> Generating Rust parser/lexer from Idl.g4..."
java -jar "$JAR" -Dlanguage=Rust -o "$GENERATED_DIR" -Xexact-output-dir "$GRAMMAR"

# ==============================================================================
# Normalize absolute path in generated comments
#
# The generator embeds the absolute grammar path in a comment. We normalize
# it to a relative path for reproducibility. The comment may be on line 1
# or line 2 (some files have #![allow] on line 1), so we replace globally.
# ==============================================================================

GRAMMAR_REL="avro/share/idl_grammar/org/apache/avro/idl/Idl.g4"

for file in "${GENERATED_FILES[@]}"; do
    sed -i "s|$GRAMMAR|$GRAMMAR_REL|" "$GENERATED_DIR/$file"
done

# ==============================================================================
# Clean up non-Rust artifacts
#
# ANTLR writes .interp and .tokens files alongside the generated code.
# With -Xexact-output-dir these land in both the output directory and
# (sometimes) next to the grammar. Remove them from both locations.
# ==============================================================================

rm -f "$GRAMMAR_DIR"/*.rs "$GRAMMAR_DIR"/*.interp "$GRAMMAR_DIR"/*.tokens
rm -f "$GENERATED_DIR"/*.interp "$GENERATED_DIR"/*.tokens

# ==============================================================================
# Smoke test
# ==============================================================================

echo "==> Running cargo build as smoke test..."
cargo build --manifest-path "$REPO_ROOT/Cargo.toml"

echo
echo "Done. Generated files updated in src/generated/."
