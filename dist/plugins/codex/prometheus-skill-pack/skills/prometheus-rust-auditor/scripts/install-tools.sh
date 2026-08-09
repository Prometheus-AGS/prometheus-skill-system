#!/usr/bin/env bash
set -euo pipefail

# Install the prometheus-rust-auditor binary and its optional tool dependencies.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../../../" && pwd)"
AUDITOR_DIR="${REPO_ROOT}/tools/prometheus-rust-auditor"

echo "==> Installing prometheus-rust-auditor"
cargo install --path "${AUDITOR_DIR}" --locked

echo "==> Installing optional audit tools"

if ! cargo deny --version &>/dev/null 2>&1; then
    echo "  -> cargo-deny"
    cargo install cargo-deny --locked
else
    echo "  -> cargo-deny already installed ($(cargo deny --version))"
fi

if ! cargo audit --version &>/dev/null 2>&1; then
    echo "  -> cargo-audit"
    cargo install cargo-audit --locked
else
    echo "  -> cargo-audit already installed ($(cargo audit --version))"
fi

if ! cargo geiger --version &>/dev/null 2>&1; then
    echo "  -> cargo-geiger"
    cargo install cargo-geiger --locked
else
    echo "  -> cargo-geiger already installed ($(cargo geiger --version))"
fi

echo ""
echo "Installation complete. Run 'prometheus-rust-auditor --help' to get started."
