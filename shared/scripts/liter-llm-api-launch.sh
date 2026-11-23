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

# Prefer the user-local update over an older system-wide copy on launchd PATH.
# An explicit service override still wins.
if [ -z "${LITER_LLM_BIN:-}" ]; then
    if [ -x "$HOME/.local/bin/liter-llm" ]; then
        LITER_LLM_BIN="$HOME/.local/bin/liter-llm"
    else
        LITER_LLM_BIN="$(command -v liter-llm || echo /usr/local/bin/liter-llm)"
    fi
fi
# Bind to loopback by default. liter-llm's own default is 0.0.0.0, which exposes the
# gateway (and every provider key behind the master key) to the whole LAN; the config
# file has no host setting, so it must be passed here. Set LITER_LLM_HOST=0.0.0.0 in
# the LaunchAgent only if another machine really needs to reach this gateway.
exec "$LITER_LLM_BIN" api --host "${LITER_LLM_HOST:-127.0.0.1}" \
    --port "${LITER_LLM_PORT:-4000}" --config "${LITER_LLM_CONFIG:-$HOME/.config/liter-llm/liter-llm-proxy.toml}"
