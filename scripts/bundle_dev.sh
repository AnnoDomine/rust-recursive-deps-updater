#!/usr/bin/env bash
set -euo pipefail

# Resolve repository root directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "==> Building rrdu in release mode..."
cargo build --release --manifest-path "${REPO_DIR}/Cargo.toml"

# Determine user-space binary destination directory
INSTALL_DIR="${CARGO_HOME:-$HOME/.cargo}/bin"
mkdir -p "${INSTALL_DIR}"

DEST_BIN="${INSTALL_DIR}/dev_rrdu"
SOURCE_BIN="${REPO_DIR}/target/release/rrdu"

echo "==> Installing binary as 'dev_rrdu' to ${DEST_BIN}..."
cp -f "${SOURCE_BIN}" "${DEST_BIN}"
chmod +x "${DEST_BIN}"

# Ensure INSTALL_DIR is in PATH for the current shell session
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
    export PATH="${INSTALL_DIR}:${PATH}"
fi

echo "==> Verifying installation:"
dev_rrdu --version

echo "==> Installation complete! 'dev_rrdu' is globally available."
