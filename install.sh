#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if ! command -v node >/dev/null 2>&1; then
    echo "Prometheus installer requires Node.js 18 or newer." >&2
    exit 1
fi

node -e 'const major=Number(process.versions.node.split(".")[0]); if(major<18) process.exit(1)' || {
    echo "Prometheus installer requires Node.js 18 or newer; found $(node --version)." >&2
    exit 1
}

exec node "$ROOT/scripts/install-system.js" --source-root "$ROOT" "$@"
