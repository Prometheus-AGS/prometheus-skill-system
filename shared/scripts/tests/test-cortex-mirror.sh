#!/bin/bash
# Real Cortex integration; explicit scratch state, missing prerequisites exit 2.
set -u
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec python3 -B "$HERE/cortex-mirror-integration.py" "$@"
