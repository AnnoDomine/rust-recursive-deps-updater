#!/usr/bin/env bash
set -euo pipefail

# Resolve repository root directory and switch to it
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${REPO_DIR}"

# Ensure cargo user-space binaries are in PATH for this session
CARGO_BIN_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"
if [[ ":$PATH:" != *":${CARGO_BIN_DIR}:"* ]]; then
    export PATH="${CARGO_BIN_DIR}:${PATH}"
fi

# 1. Check if Rust is installed and matches the required version
RRDU_RUST_VERSION=$(grep -m1 '^rust-version = ' "${REPO_DIR}/Cargo.toml" | cut -d '"' -f2)
RRDU_VERSION=$(grep -m1 '^version = ' "${REPO_DIR}/Cargo.toml" | cut -d '"' -f2)

echo "==> [1/3] Checking Rust installation (required version: ${RRDU_RUST_VERSION})..."
if ! command -v rustc >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
    echo "Error: Rust is not installed. Please install Rust (e.g., via https://rustup.rs)."
    exit 1
fi

INSTALLED_RUST_VERSION=$(rustc --version | awk '{print $2}')
if [ "${INSTALLED_RUST_VERSION}" != "${RRDU_RUST_VERSION}" ]; then
    echo "Required Rust version ${RRDU_RUST_VERSION}. Please install Rust with this version (current: ${INSTALLED_RUST_VERSION})."
    exit 1
fi
echo "    Rust ${INSTALLED_RUST_VERSION} is active."

# 2. Check and install required development CLI tools
echo "==> [2/3] Checking development CLI tools..."

# mdbook
if ! command -v mdbook >/dev/null 2>&1; then
    echo "    Installing mdbook..."
    cargo install mdbook --locked
else
    echo "    mdbook is installed ($(mdbook --version))."
fi

# mdbook-variables
if ! command -v mdbook-variables >/dev/null 2>&1; then
    echo "    Installing mdbook-variables..."
    cargo install mdbook-variables --locked
else
    echo "    mdbook-variables is installed."
fi

# cargo-audit
if ! command -v cargo-audit >/dev/null 2>&1 && ! cargo audit --version >/dev/null 2>&1; then
    echo "    Installing cargo-audit..."
    cargo install cargo-audit --locked
else
    echo "    cargo-audit is installed."
fi

# 3. Run CI checks to verify development environment is ready
echo "==> [3/3] Running local CI checks to verify development environment..."

echo "--> Checking compilation (cargo check)..."
cargo check

echo "--> Scanning dependencies for vulnerabilities (cargo audit)..."
cargo audit

echo "--> Running test suite (cargo test)..."
cargo test --all-targets --verbose

echo "--> Checking code formatting (cargo fmt)..."
cargo fmt --all -- --check

echo "--> Running Clippy linter (cargo clippy)..."
cargo clippy --all-targets -- -D warnings

echo "--> Checking Rustdoc documentation (cargo doc)..."
RUSTDOCFLAGS="-D warnings -D missing_docs" cargo doc --no-deps

echo "--> Building mdBook documentation..."
RRDU_VERSION="${RRDU_VERSION}" mdbook build docs

echo "--> Verifying micro-benchmarks compile (smoke test)..."
cargo bench --bench rrdu_benchmarks -- --test

echo "==> Development environment is successfully initialized and verified for rrdu v${RRDU_VERSION}!"