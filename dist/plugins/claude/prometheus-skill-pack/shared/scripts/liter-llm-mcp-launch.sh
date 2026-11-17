#!/usr/bin/env bash
# liter-llm-mcp-launch.sh — stdio entry point for the liter-llm MCP server.
#
# Installed as ~/.local/bin/liter-llm-mcp by scripts/install-liter-llm-mcp-launcher.mjs.
# Every harness MCP entry the pack writes runs that copy, with no arguments.
#
# Why a wrapper: the liter-llm config reads its keys as ${VAR} references
# (master_key = "${LITER_LLM_MASTER_KEY}", one per provider). The values live
# ONLY in ~/.prometheus/kbd/secrets.env (0600). A harness started from the Dock,
# Finder or a stale shell does not have them in its environment, and liter-llm
# then exits at once because an empty master key would authenticate every
# request. This wrapper sources the secrets file and execs the real binary, the
# same way liter-llm-api-launch.sh does for the gateway service, so no harness
# config ever holds a secret value.
#
# Contract:
#   - stdout is the MCP channel. Nothing is written to it before the exec; the
#     secrets file is sourced with stdout sent to stderr and stdin closed.
#   - Every failure is a message on stderr and a non-zero exit, never a hang:
#       66  the liter-llm config file is missing
#       78  the master key variable is empty (the message names the secrets file)
#       127 the liter-llm binary is missing
#   - `liter-llm-mcp --check` runs the same checks and exits without starting
#     the server (0 = it would start).
#
# Overrides: LITER_LLM_BIN, LITER_LLM_CONFIG, PROMETHEUS_KBD_SECRETS.
# bash 3.2 compatible (macOS /bin/bash): harnesses may run this with any PATH.

set -u

die() {
    _code="$1"
    shift
    printf 'liter-llm-mcp: %s\n' "$*" >&2
    exit "$_code"
}

CHECK_ONLY=0
case "${1:-}" in
    --check) CHECK_ONLY=1 ;;
    "") ;;
    *) die 64 "unknown argument '$1' (usage: liter-llm-mcp [--check])" ;;
esac

[ -n "${HOME:-}" ] || die 78 "HOME is not set, so the secrets file and config cannot be located"

SECRETS="${PROMETHEUS_KBD_SECRETS:-$HOME/.prometheus/kbd/secrets.env}"
CONFIG="${LITER_LLM_CONFIG:-$HOME/.config/liter-llm/liter-llm-proxy.toml}"

SECRETS_STATE="absent"
if [ -f "$SECRETS" ]; then
    if [ -r "$SECRETS" ]; then
        # `set +u` while sourcing: a secrets file that reads an unset variable
        # must not kill this shell before it can report anything.
        set +u
        set -a
        # shellcheck disable=SC1090
        if . "$SECRETS" </dev/null 1>&2; then
            SECRETS_STATE="loaded"
        else
            SECRETS_STATE="failed"
        fi
        set +a
        set -u
    else
        SECRETS_STATE="unreadable"
    fi
fi

# Prefer the user-local install over an older system-wide copy. The fixed
# locations matter: a harness started from the Dock has a minimal PATH.
BIN="${LITER_LLM_BIN:-}"
if [ -z "$BIN" ]; then
    for _candidate in "$HOME/.local/bin/liter-llm" "$(command -v liter-llm 2>/dev/null || true)" \
        /usr/local/bin/liter-llm /opt/homebrew/bin/liter-llm "$HOME/.cargo/bin/liter-llm"; do
        if [ -n "$_candidate" ] && [ -x "$_candidate" ]; then
            BIN="$_candidate"
            break
        fi
    done
fi
if [ -z "$BIN" ] || [ ! -x "$BIN" ]; then
    die 127 "liter-llm binary not found${BIN:+ at $BIN} (looked in $HOME/.local/bin, PATH, /usr/local/bin, /opt/homebrew/bin, $HOME/.cargo/bin). Install it with scripts/install-binaries.sh from the Prometheus skill pack, or set LITER_LLM_BIN."
fi

[ -f "$CONFIG" ] || die 66 "liter-llm config not found at $CONFIG. Create it with /liter-llm-bridge configure (or scripts/install-binaries.sh), or set LITER_LLM_CONFIG."

# liter-llm refuses to start when master_key interpolates to "". Its own message
# does not say where the value is expected to come from, so check here and name
# the file. Only the variable NAME is ever read or printed.
MASTER_VAR="$(sed -E 's/#.*$//' "$CONFIG" 2>/dev/null \
    | sed -n -E 's/^[[:space:]]*master_key[[:space:]]*=[[:space:]]*"\$\{([A-Za-z_][A-Za-z0-9_]*)\}".*$/\1/p' \
    | head -n 1)"
if [ -n "$MASTER_VAR" ]; then
    eval "_master_value=\${$MASTER_VAR:-}"
    if [ -z "$_master_value" ]; then
        case "$SECRETS_STATE" in
            absent) _why="the secrets file $SECRETS does not exist" ;;
            unreadable) _why="the secrets file $SECRETS is not readable by this user" ;;
            failed) _why="the secrets file $SECRETS could not be sourced (see the shell error above)" ;;
            *) _why="the secrets file $SECRETS does not define it" ;;
        esac
        die 78 "\${$MASTER_VAR} is empty and $CONFIG uses it as the master key: $_why. Define $MASTER_VAR there (mode 0600) or run /liter-llm-bridge configure."
    fi
    unset _master_value
fi

if [ "$CHECK_ONLY" -eq 1 ]; then
    printf 'liter-llm-mcp: ok (binary %s, config %s, secrets file %s)\n' "$BIN" "$CONFIG" "$SECRETS_STATE" >&2
    exit 0
fi

# --config is required: liter-llm never searches $HOME for its config, and
# [mcp] stdio_trust_local lives in that file.
exec "$BIN" mcp --transport stdio --config "$CONFIG"
