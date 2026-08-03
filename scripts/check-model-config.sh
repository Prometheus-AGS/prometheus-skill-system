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
            note "UNSET: \${$v} referenced by the config but not exported (expands to \"\")"
        fi
    done
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
