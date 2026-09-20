#!/usr/bin/env bash
# Render every agent-context target from rules/src/. See rules/build.py.
#   bash rules/build.sh            render
#   bash rules/build.sh --check    verify only; exit 1 on drift or a budget breach (CI, pre-commit)
#   bash rules/build.sh --with-cursor
set -euo pipefail
command -v python3 >/dev/null 2>&1 || { echo "rules/build: python3 is required" >&2; exit 2; }
exec python3 "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/build.py" "$@"
