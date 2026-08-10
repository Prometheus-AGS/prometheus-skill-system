#!/usr/bin/env bash
# install-liter-llm.sh — clone (or update) GQAdonis/liter-llm fork and
# install the CLI binary to ~/.local/bin/liter-llm.
#
# Usage:
#   install-liter-llm.sh           # track main, build latest commit
#   install-liter-llm.sh --release # check out latest tag instead
#   install-liter-llm.sh --dry-run # report what would happen, do nothing
#
# Idempotent: re-running updates the source tree and rebuilds only if needed.

set -euo pipefail

REPO_URL="https://github.com/GQAdonis/liter-llm.git"
SRC_DIR="${LITER_LLM_SRC:-$HOME/.local/share/liter-llm/src}"
INSTALL_ROOT="${LITER_LLM_INSTALL_ROOT:-$HOME/.local}"

DRY_RUN=0
USE_RELEASE=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    --release) USE_RELEASE=1 ;;
    *) echo "Unknown arg: $arg" >&2; exit 2 ;;
  esac
done

run() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "[dry-run] $*"
  else
    "$@"
  fi
}

# 1. Clone or update
if [[ -d "$SRC_DIR/.git" ]]; then
  echo "Updating $SRC_DIR..."
  run git -C "$SRC_DIR" fetch --tags --prune
  if [[ "$USE_RELEASE" -eq 1 ]]; then
    LATEST_TAG="$(git -C "$SRC_DIR" tag --sort=-v:refname | head -1)"
    if [[ -n "$LATEST_TAG" ]]; then
      run git -C "$SRC_DIR" checkout "$LATEST_TAG"
    fi
  else
    run git -C "$SRC_DIR" checkout main
    run git -C "$SRC_DIR" pull --ff-only
  fi
else
  echo "Cloning $REPO_URL → $SRC_DIR..."
  run mkdir -p "$(dirname "$SRC_DIR")"
  run git clone "$REPO_URL" "$SRC_DIR"
  if [[ "$USE_RELEASE" -eq 1 ]]; then
    LATEST_TAG="$(git -C "$SRC_DIR" tag --sort=-v:refname | head -1)"
    [[ -n "$LATEST_TAG" ]] && run git -C "$SRC_DIR" checkout "$LATEST_TAG"
  fi
fi

# 2. Locate the CLI crate
CLI_CRATE="$SRC_DIR/crates/liter-llm-cli"
if [[ ! -d "$CLI_CRATE" ]]; then
  echo "ERROR: CLI crate not found at $CLI_CRATE" >&2
  echo "       The fork's layout may have changed." >&2
  exit 3
fi

# 3. Build and install
echo "Building liter-llm-cli (this may take several minutes)..."
run cargo install --path "$CLI_CRATE" --locked --root "$INSTALL_ROOT"

# 4. Verify
BIN="$INSTALL_ROOT/bin/liter-llm"
if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "[dry-run] would verify: $BIN --version"
  exit 0
fi

if [[ ! -x "$BIN" ]]; then
  echo "ERROR: install completed but $BIN is not executable" >&2
  exit 4
fi

VERSION="$("$BIN" --version 2>/dev/null || echo unknown)"
echo "INSTALL COMPLETE: $VERSION at $BIN"

# 5. PATH check
if ! command -v liter-llm >/dev/null 2>&1; then
  echo "WARNING: $INSTALL_ROOT/bin is not on PATH."
  echo "         Add to your shell rc: export PATH=\"$INSTALL_ROOT/bin:\$PATH\""
fi

# 6. MCP subcommand sanity check
if ! "$BIN" mcp --help >/dev/null 2>&1; then
  echo "WARNING: 'liter-llm mcp' subcommand not available — old version?"
  exit 5
fi

echo "MCP transport available."
