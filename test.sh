#!/bin/bash
# SPDX-License-Identifier: GPL-2.0

# Rust Kernel Test Script

set -e

echo "=== Rust Kernel Build Test ==="

# Check if rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo not found. Please install Rust first."
    exit 1
fi

echo "Rust version:"
rustc --version
cargo --version

echo ""
echo "=== Building kernel ==="
cargo check --workspace

echo ""
echo "=== Running tests ==="
cargo test --workspace --lib

echo ""
echo "=== Checking formatting ==="
cargo fmt --check || echo "Warning: Code formatting issues found"

echo ""
echo "=== Running clippy ==="
cargo clippy --workspace -- -D warnings || echo "Warning: Clippy found issues"

echo ""
echo "=== Building release version ==="
cargo build --release

echo ""
echo "=== Build completed successfully! ==="
echo "Kernel artifacts can be found in target/release/"
