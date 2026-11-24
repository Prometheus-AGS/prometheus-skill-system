#!/bin/bash
set -eu
scripts=$(cd "$(dirname "$0")" && pwd -P)
exec python3 "$scripts/tests/kbd-intersession-packet.py" "$@"
