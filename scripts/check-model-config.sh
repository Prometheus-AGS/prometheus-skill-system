#!/usr/bin/env bash
# check-model-config.sh — verify KBD adversarial model routing and the active
# content-addressed plugin generation.
#
# Usage:
#   bash scripts/check-model-config.sh          human-readable report
#   bash scripts/check-model-config.sh --json   machine-readable
#
# Exit: 0 all good · 1 routing broken · 2 active generation invalid

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
JSON=false
[ "${1:-}" = "--json" ] && JSON=true

LITER_CFG="${LITER_LLM_CONFIG:-$HOME/.config/liter-llm/liter-llm-proxy.toml}"
KBD_MODELS="${PROMETHEUS_KBD_MODELS_CONFIG:-$HOME/.prometheus/kbd/models.toml}"
KBD_SECRETS="${PROMETHEUS_KBD_SECRETS:-$HOME/.prometheus/kbd/secrets.env}"
MCP_LAUNCHER="$HOME/.local/bin/liter-llm-mcp"

# secrets_define <VAR> — true when the secrets file gives VAR a non-empty value.
# Runs in a subshell and prints nothing: only the variable NAME is ever handled.
secrets_define() {
    [ -r "$KBD_SECRETS" ] || return 1
    (
        set -a
        # shellcheck disable=SC1090
        . "$KBD_SECRETS" >/dev/null 2>&1 </dev/null || exit 1
        eval "[ -n \"\${$1:-}\" ]"
    )
}

FAIL=0
DRIFT=0
NOTES=""

note() { NOTES="${NOTES}$1
"; }

# --- 1. active generation integrity ----------------------------------------
PLUGIN_ROOT="${PROMETHEUS_PLUGIN_ROOT:-$HOME/.prometheus/plugins/prometheus-skill-pack}"
if [ -e "$PLUGIN_ROOT/current" ]; then
    if ! node "$REPO_ROOT/scripts/install-plugin-generation.js" \
        --plugin-root "$PLUGIN_ROOT" --verify >/dev/null 2>&1; then
        DRIFT=1
        note "INVALID: active plugin generation failed its content-hash manifest verification"
    fi
else
    note "WARN: no active plugin generation; run bash scripts/install-skills-flat.sh --skills-only"
fi

# --- 2. liter-llm config sanity ---------------------------------------------
CFG_OK=true
if [ ! -f "$LITER_CFG" ]; then
    CFG_OK=false; FAIL=1
    note "MISSING: $LITER_CFG — run /liter-llm-bridge configure repair"
else
    if ! grep -qE '^[[:space:]]*master_key[[:space:]]*=' "$LITER_CFG" 2>/dev/null \
       && ! grep -qE '^\[\[keys\]\]' "$LITER_CFG" 2>/dev/null; then
        CFG_OK=false; FAIL=1
        note "BROKEN: no [general] master_key — every /v1/* request answers 401"
    fi
    if grep -qE '^[[:space:]]*base_url[[:space:]]*=.*(localhost|127\.0\.0\.1)' "$LITER_CFG" 2>/dev/null \
       && ! grep -qE '^[[:space:]]*outbound_policy[[:space:]]*=' "$LITER_CFG" 2>/dev/null; then
        CFG_OK=false; FAIL=1
        note "BROKEN: localhost base_url with no [security] outbound_policy — deny_private blocks loopback"
    fi
    # ${VAR} refs that are not set expand to "" and 401 much later.
    for v in $(sed -E 's/#.*$//' "$LITER_CFG" 2>/dev/null \
                 | grep -oE '\$\{[A-Z_][A-Z0-9_]*\}' 2>/dev/null | tr -d '${}' | sort -u); do
        eval "val=\${$v:-}"
        if [ -z "$val" ]; then
            CFG_OK=false; FAIL=1
            if secrets_define "$v"; then
                # The launchers source the secrets file, so the MCP server and the
                # gateway service still get it. Shell callers (kbd_complete,
                # dispatch-judge) read this shell's environment and do not.
                note "UNSET IN THIS SHELL: \${$v} is defined in $KBD_SECRETS but not exported here — the liter-llm launchers load it, shell callers do not (set -a; . $KBD_SECRETS; set +a)"
            else
                note "UNSET: \${$v} referenced by the config but not exported and not defined in $KBD_SECRETS (expands to \"\")"
            fi
        fi
    done
fi

# --- 2b. liter-llm MCP launcher ---------------------------------------------
# Harness MCP entries run the launcher, not the binary. Its --check performs the
# start-up checks of a real launch. It runs here in an EMPTY environment, which
# is how a harness started from the Dock or Finder sees it: the keys must come
# from the secrets file, not from this shell.
LAUNCHER_OK=true
if [ ! -x "$MCP_LAUNCHER" ]; then
    LAUNCHER_OK=false; FAIL=1
    note "MISSING: MCP launcher $MCP_LAUNCHER — run node scripts/install-liter-llm-mcp-launcher.mjs"
else
    _launch_msg="$(env -i HOME="$HOME" PATH="/usr/bin:/bin:/usr/sbin:/sbin" \
        LITER_LLM_CONFIG="$LITER_CFG" PROMETHEUS_KBD_SECRETS="$KBD_SECRETS" \
        "$MCP_LAUNCHER" --check 2>&1)" || {
        LAUNCHER_OK=false; FAIL=1
        note "MCP SERVER WOULD NOT START: $_launch_msg"
    }
fi
if command -v node >/dev/null 2>&1 \
   && ! node "$REPO_ROOT/scripts/install-liter-llm-mcp-launcher.mjs" --check >/dev/null 2>&1; then
    note "WARN: the installed MCP launcher differs from this checkout, or a harness config still starts liter-llm directly — run node scripts/install-liter-llm-mcp-launcher.mjs"
fi

# --- 3. role resolution -----------------------------------------------------
LIB="$REPO_ROOT/shared/scripts/lib/kbd-model-resolve.sh"
R_JUDGE=""; R_CRITIC=""; R_GEN=""; GATEWAY=""; ISO="unknown"
if [ -f "$LIB" ]; then
    # shellcheck source=/dev/null
    . "$LIB"
    R_JUDGE="$(kbd_resolve_role judge 2>/dev/null || true)"
    R_CRITIC="$(kbd_resolve_role critic 2>/dev/null || true)"
    R_GEN="$(kbd_resolve_role generator 2>/dev/null || true)"
    GATEWAY="$(kbd_resolve_gateway 2>/dev/null || true)"

    [ -n "$GATEWAY" ] || { FAIL=1; note "NO GATEWAY: nothing answered /v1/models (start openai-proxy or liter-llm api)"; }
    [ -n "$R_JUDGE" ]  || { FAIL=1; note "UNRESOLVED: judge role"; }

    if [ -n "$R_JUDGE" ] && [ "$R_JUDGE" = "$R_CRITIC" ]; then
        FAIL=1
        note "COLLISION: judge and critic resolve to the same model ($R_JUDGE) — no cross-model check possible"
    fi
    # A role pointing at a model liter-llm does not declare fails at dispatch time.
    for pair in "judge:$R_JUDGE" "critic:$R_CRITIC"; do
        _r="${pair%%:*}"; _m="${pair#*:}"
        [ -n "$_m" ] || continue
        if ! kbd_model_declared "$_m" 2>/dev/null; then
            note "WARN: role $_r -> '$_m' has no [[models]] entry in $LITER_CFG"
        fi
    done
    [ -n "$GATEWAY" ] && ISO="rest-gateway:$GATEWAY"
else
    FAIL=1; note "MISSING: $LIB"
fi

[ -f "$KBD_MODELS" ] || note "WARN: $KBD_MODELS absent — roles fall back to built-in defaults"

# --- report -----------------------------------------------------------------
if $JSON; then
    NOTES_J="$(printf '%s' "$NOTES" | python3 -c '
import json,sys
print(json.dumps([l for l in sys.stdin.read().split("\n") if l.strip()]))' 2>/dev/null || echo '[]')"
    printf '{\n'
    printf '  "gateway": "%s",\n' "$GATEWAY"
    printf '  "roles": {"judge": "%s", "critic": "%s", "generator": "%s"},\n' "$R_JUDGE" "$R_CRITIC" "$R_GEN"
    printf '  "isolation_mode_would_be": "%s",\n' "$ISO"
    printf '  "config_ok": %s,\n' "$($CFG_OK && echo true || echo false)"
    printf '  "mcp_launcher_ok": %s,\n' "$($LAUNCHER_OK && echo true || echo false)"
    printf '  "active_generation_invalid": %s,\n' "$([ "$DRIFT" -eq 1 ] && echo true || echo false)"
    printf '  "findings": %s\n' "$NOTES_J"
    printf '}\n'
else
    echo "KBD adversarial model routing"
    echo "  gateway              : ${GATEWAY:-<none reachable>}"
    echo "  judge                : ${R_JUDGE:-<unresolved>}"
    echo "  critic               : ${R_CRITIC:-<unresolved>}"
    echo "  generator (producer) : ${R_GEN:-<unresolved>}"
    echo "  isolation_mode would be: $ISO"
    echo "  liter-llm config     : $LITER_CFG"
    echo "  MCP launcher         : $MCP_LAUNCHER ($($LAUNCHER_OK && echo ok || echo 'NOT OK'))"
    echo "  role map             : $KBD_MODELS"
    if [ -n "$NOTES" ]; then
        echo ""
        echo "Findings:"
        printf '%s' "$NOTES" | sed 's/^/  - /'
        if [ "$DRIFT" -eq 1 ]; then
            echo ""
            echo "  Roll back to the previous verified generation or reinstall from source."
        fi
    else
        echo ""
        echo "  ✅ no findings"
    fi
fi

[ "$DRIFT" -eq 1 ] && exit 2
exit "$FAIL"
