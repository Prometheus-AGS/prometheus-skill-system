#!/usr/bin/env bash
set -euo pipefail

exec prometheus doctor "$@"
