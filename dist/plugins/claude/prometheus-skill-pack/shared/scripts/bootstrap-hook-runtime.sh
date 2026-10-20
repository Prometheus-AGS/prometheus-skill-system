#!/usr/bin/env bash
# Compatibility entry for cached hook commands. Hook execution must never
# install, upgrade, or recover an operator-owned plugin installation.
set -euo pipefail
printf '%s\n' '{"status":"NOT_ACTIVATED","reason":"automatic activation disabled; explicitly install the intended payload with install-plugin-generation.js, then restart the session"}' >&2
exit 78
