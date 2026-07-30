#!/usr/bin/env bash
# regenerate-antlr.sh — Regenerate the ANTLR parser/lexer from Idl.g4.
#
# The generated Rust files in src/generated/ are checked in so that
# downstream users don't need any generation tooling. This script is only
# needed when the grammar (Idl.g4) changes.
#
# Generation is pure Rust: the parser/lexer are produced by the
# `antlr4-rust-gen` binary shipped with the antlr-rust-runtime crate this
# project depends on — no Java and no ANTLR tool JAR. The generator is
# installed at the exact runtime version locked in Cargo.lock, so the generated
# code always matches the runtime the crate compiles against.
#
# Prerequisites:
#   - A Rust toolchain (the same one used to build this crate)
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

# Project-local install root for the generator binary (keeps ~/.cargo/bin clean).
GENERATOR_ROOT="$REPO_ROOT/tmp/antlr-generator"

# ==============================================================================
# Validate prerequisites
# ==============================================================================

if ! command -v cargo &>/dev/null; then
    echo "Error: cargo not found in PATH." >&2
    exit 1
fi

if [ ! -f "$GRAMMAR" ]; then
    echo "Error: Grammar file not found at $GRAMMAR" >&2
    echo "Make sure the avro submodule is initialized." >&2
    exit 1
fi

# ==============================================================================
# Resolve the antlr-rust-runtime version so the generator matches the runtime
# the crate compiles against. The generator binary (`antlr4-rust-gen`)
# ships inside the runtime crate itself, so installing the exact locked version
# is enough — a published version is immutable, which makes regeneration
# byte-reproducible.
#
# `cargo pkgid` prints the resolved spec, e.g.
#   registry+https://github.com/rust-lang/crates.io-index#antlr-rust-runtime@0.22.0
# Query the real crate name, not the `antlr4_runtime` rename in Cargo.toml.
# ==============================================================================

RUNTIME_VERSION="$(cargo pkgid --manifest-path "$REPO_ROOT/Cargo.toml" antlr-rust-runtime | sed 's/.*@//')"

echo "==> Runtime generator: antlr-rust-runtime $RUNTIME_VERSION (crates.io)"

# ==============================================================================
# Install the generator at the locked version into a project-local root, so we
# neither touch the user's ~/.cargo/bin nor rebuild it on every run.
# ==============================================================================

GENERATOR="$GENERATOR_ROOT/bin/antlr4-rust-gen"

if [ -x "$GENERATOR" ] && "$GENERATOR" --version 2>/dev/null | grep -qF "$RUNTIME_VERSION"; then
    echo "==> Reusing cached generator."
else
    echo "==> Installing antlr4-rust-gen $RUNTIME_VERSION..."
    cargo install antlr-rust-runtime \
        --version "$RUNTIME_VERSION" \
        --locked \
        --features codegen \
        --bin antlr4-rust-gen \
        --root "$GENERATOR_ROOT"
fi

# ==============================================================================
# Run the pure-Rust generator.
#
# --out-dir writes idl_lexer.rs / idl_parser.rs / mod.rs directly into
# src/generated/, plus two informational manifests (semantics.json listing
# semantic predicate/action coordinates, decisions.json reporting each parser
# decision's lookahead tier). Only the Rust sources are checked in.
# ==============================================================================

echo "==> Generating Rust parser/lexer from Idl.g4..."
"$GENERATOR" \
    "$GRAMMAR" \
    --out-dir "$GENERATED_DIR"

# ==============================================================================
# Clean up: the generator's JSON manifests are diagnostics, not build inputs,
# and stray artifacts ANTLR-style tooling leaves next to the grammar.
# ==============================================================================

rm -f "$GENERATED_DIR"/semantics.json "$GENERATED_DIR"/decisions.json
rm -f "$GRAMMAR_DIR"/*.interp "$GRAMMAR_DIR"/*.tokens

# ==============================================================================
# Smoke test
# ==============================================================================

echo "==> Running cargo build as smoke test..."
cargo build --manifest-path "$REPO_ROOT/Cargo.toml"

echo
echo "Done. Generated files updated in src/generated/."
