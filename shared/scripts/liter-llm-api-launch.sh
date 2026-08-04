#!/usr/bin/env bash
# liter-llm-api-launch.sh — launchd entry point for the liter-llm gateway.
#
# launchd's EnvironmentVariables dict cannot source a file, and the project's
# secrets (KIMI_CODING_KEY, MINIMAX_KEY, LITER_LLM_MASTER_KEY, ...) intentionally
# live ONLY in ~/.prometheus/kbd/secrets.env (0600) — never duplicated into a
# plist, which is typically 644. This wrapper sources that file and execs the
# real binary so the plist itself never contains a secret value.
set -euo pipefail

SECRETS="$HOME/.prometheus/kbd/secrets.env"
if [ -f "$SECRETS" ]; then
    set -a
    # shellcheck disable=SC1090
    . "$SECRETS"
    set +a
fi

LITER_LLM_BIN="$(command -v liter-llm || echo /usr/local/bin/liter-llm)"
exec "$LITER_LLM_BIN" api --config "$HOME/.config/liter-llm/liter-llm-proxy.toml"
