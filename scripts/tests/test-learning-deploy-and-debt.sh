#!/bin/bash
# Sole local integration coordinator. Explicit candidates; no live-home defaults.
set -u
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec python3 -B "$HERE/learning-deploy-and-debt.py" "$@"
