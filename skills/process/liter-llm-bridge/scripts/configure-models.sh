#!/usr/bin/env bash
# configure-models.sh — generate/repair liter-llm + KBD model configuration.
#
# Usage:
#   configure-models.sh check                 report current state, change nothing
#   configure-models.sh repair                add ONLY the mandatory missing pieces
#   configure-models.sh add-provider <name>   add a provider's [[models]] entry
#   configure-models.sh verify                live 1-token completion per role
#   configure-models.sh migrate               retire the invented config.toml
#
# WHY THIS EXISTS
# Configuring the adversarial judge used to be painful enough that a previous
# session "fixed" it by editing pack scripts inside the plugin cache — edits the
# next install destroys and git never sees. Everything this script writes lives in
# the two files that own it, so a config change never requires touching a script:
#
#   ~/.prometheus/kbd/models.toml              role -> model NAME   (KBD owns)
#   ~/.config/liter-llm/liter-llm-proxy.toml   NAME -> provider+url (liter-llm owns)
#
# HARD-WON CONTRACTS (verified 2026-07-30 against liter-llm at tools/liter-llm)
#   * /v1/* sits behind an unconditional Bearer check. A config with no
#     [general] master_key and no [[keys]] answers 401 to EVERYTHING, /v1/models
#     included. The template shipped before today omitted it.
#   * [security].outbound_policy defaults to deny_private, which REFUSES loopback.
#     Any localhost base_url needs "off" or an explicit allowlist.
#   * ProxyConfig::discover() walks the CWD upward only — it never looks in $HOME.
#     Callers MUST pass --config <abs path> or liter-llm starts with zero models.
#   * [[models]] uses serde(deny_unknown_fields): fields are exactly name,
#     provider_model, api_key, base_url, timeout_secs, fallbacks. Any extra key is
#     a hard parse error.
#   * Env interpolation is ${VAR} ONLY (no ${VAR:-default}) and an UNSET var
#     silently becomes "" — which surfaces much later as a confusing 401. This
#     script therefore verifies every referenced var is actually set.
#
# Secrets are NEVER written into the TOML. Keys go to ~/.prometheus/kbd/secrets.env
# (0600) and the config references them as ${VAR}.

set -uo pipefail

LITER_CFG="${LITER_LLM_CONFIG:-$HOME/.config/liter-llm/liter-llm-proxy.toml}"
KBD_DIR="$HOME/.prometheus/kbd"
KBD_MODELS="$KBD_DIR/models.toml"
SECRETS="$KBD_DIR/secrets.env"
LEGACY_CFG="$HOME/.config/liter-llm/config.toml"

say()  { printf '%s\n' "$*"; }
ok()   { printf '  ✅ %s\n' "$*"; }
warn() { printf '  ⚠️  %s\n' "$*" >&2; }
err()  { printf '  ❌ %s\n' "$*" >&2; }

# --- provider table ---------------------------------------------------------
# base_url values are load-bearing. Z.ai's docs are explicit that a GLM Coding
# Plan MUST use /api/coding/paas/v4 — the general /api/paas/v4 does not draw on
# subscription quota. The *-coding-plan identifiers in liter-llm's catalog.json
# are metadata, NOT routable providers, so coding plans keep a routable prefix
# and override the endpoint instead.
#
# Format: name|provider_model|base_url|key_var
provider_row() {
    case "$1" in
      local-proxy)   echo 'kbd-judge|openai/gpt-5.6-sol|http://localhost:8181/v1|' ;;
      kimi)          echo 'kbd-kimi|moonshot/kimi-k2.5|https://api.moonshot.ai/v1|MOONSHOT_API_KEY' ;;
      minimax)       echo 'kbd-minimax|minimax/MiniMax-M2.5|https://api.minimax.io/v1|MINIMAX_API_KEY' ;;
      qwen)          echo 'kbd-qwen|dashscope/qwen3-coder-plus|https://dashscope-intl.aliyuncs.com/compatible-mode/v1|DASHSCOPE_API_KEY' ;;
      glm)           echo 'kbd-glm|zai/glm-4.7|https://api.z.ai/api/paas/v4|ZAI_API_KEY' ;;
      glm-coding)    echo 'kbd-glm|zai/glm-5.2|https://api.z.ai/api/coding/paas/v4|ZAI_API_KEY' ;;
      kimi-coding)   echo 'kbd-kimi|moonshot/kimi-k2.5|https://api.moonshot.ai/v1|MOONSHOT_API_KEY' ;;
      *)             return 1 ;;
    esac
}

provider_names() { echo "local-proxy kimi minimax qwen glm glm-coding kimi-coding"; }

# --- helpers ----------------------------------------------------------------
cfg_has_master_key() {
    [ -f "$LITER_CFG" ] || return 1
    grep -qE '^[[:space:]]*master_key[[:space:]]*=' "$LITER_CFG" 2>/dev/null \
      || grep -qE '^\[\[keys\]\]' "$LITER_CFG" 2>/dev/null
}

cfg_has_outbound_policy() {
    [ -f "$LITER_CFG" ] || return 1
    grep -qE '^[[:space:]]*outbound_policy[[:space:]]*=' "$LITER_CFG" 2>/dev/null
}

cfg_uses_loopback() {
    [ -f "$LITER_CFG" ] || return 1
    grep -qE '^[[:space:]]*base_url[[:space:]]*=.*(localhost|127\.0\.0\.1)' "$LITER_CFG" 2>/dev/null
}

cfg_has_model() {
    [ -f "$LITER_CFG" ] || return 1
    grep -qE "^[[:space:]]*name[[:space:]]*=[[:space:]]*\"$1\"[[:space:]]*\$" "$LITER_CFG" 2>/dev/null
}

# Every ${VAR} the config references must actually be set, or liter-llm expands it
# to "" and the failure appears later as an unexplained 401.
unset_referenced_vars() {
    [ -f "$LITER_CFG" ] || return 0
    # Strip comments first: these configs document the ${VAR} rule in prose, and a
    # naive scan reports the literal word "VAR" from the comment as an unset var.
    sed -E 's/#.*$//' "$LITER_CFG" 2>/dev/null \
      | grep -oE '\$\{[A-Z_][A-Z0-9_]*\}' 2>/dev/null \
      | tr -d '${}' | sort -u | while read -r v; do
        [ -n "$v" ] || continue
        eval "val=\${$v:-}"
        [ -n "$val" ] || printf '%s\n' "$v"
    done
}

gateway_url() {
    if [ -n "${LITER_LLM_BASE_URL:-}" ]; then printf '%s\n' "$LITER_LLM_BASE_URL"; return 0; fi
    for c in "http://localhost:8181/v1" "http://localhost:4000/v1"; do
        if curl -s -o /dev/null --max-time 5 --noproxy '*' "$c/models" 2>/dev/null; then
            printf '%s\n' "$c"; return 0
        fi
    done
    return 1
}

# --- check ------------------------------------------------------------------
cmd_check() {
    say "liter-llm / KBD model configuration"
    say ""
    say "liter-llm config: $LITER_CFG"
    if [ ! -f "$LITER_CFG" ]; then
        warn "absent — run: $0 repair"
    else
        cfg_has_master_key && ok "[general] master_key present" \
            || err "master_key MISSING — every /v1/* request will 401"
        if cfg_uses_loopback; then
            cfg_has_outbound_policy && ok "[security] outbound_policy present" \
                || err "outbound_policy MISSING — deny_private blocks the localhost base_url"
        fi
        _u="$(unset_referenced_vars)"
        if [ -n "$_u" ]; then
            for v in $_u; do err "\${$v} referenced but NOT set — expands to \"\" (401 later)"; done
        else
            ok "all \${VAR} references resolve"
        fi
    fi

    say ""
    say "KBD role map: $KBD_MODELS"
    if [ ! -f "$KBD_MODELS" ]; then
        warn "absent — run: $0 repair"
    else
        for r in generator critic judge; do
            _m="$(grep -E "^[[:space:]]*$r[[:space:]]*=" "$KBD_MODELS" 2>/dev/null \
                  | head -1 | sed -E 's/^[^=]*=[[:space:]]*"?([^"]*)"?.*/\1/')"
            printf '  %-10s -> %s\n' "$r" "${_m:-<unset>}"
        done
    fi

    say ""
    _gw="$(gateway_url || true)"
    if [ -n "$_gw" ]; then ok "gateway reachable: $_gw"; else err "no gateway reachable"; fi

    if [ -f "$LEGACY_CFG" ]; then
        say ""
        warn "legacy $LEGACY_CFG exists — its [endpoint]/[aliases] shape is NOT a"
        warn "  schema liter-llm can load. Retire it with: $0 migrate"
    fi
}

# --- repair -----------------------------------------------------------------
# Adds ONLY what is missing. Never rewrites or reorders existing entries: the file
# may carry real user models and keys.
cmd_repair() {
    mkdir -p "$KBD_DIR" "$(dirname "$LITER_CFG")"

    if [ ! -f "$SECRETS" ]; then
        _key="sk-kbd-$(head -c 24 /dev/urandom | base64 | tr -dc 'a-zA-Z0-9' | head -c 32)"
        cat > "$SECRETS" <<EOF
# KBD / liter-llm secrets — machine-local, 0600, never committed.
# Source before starting liter-llm or running adversarial review:
#   set -a; . ~/.prometheus/kbd/secrets.env; set +a
export LITER_LLM_MASTER_KEY="${_key}"
# export ZAI_API_KEY=""
# export MOONSHOT_API_KEY=""
# export MINIMAX_API_KEY=""
# export DASHSCOPE_API_KEY=""
EOF
        chmod 0600 "$SECRETS"
        ok "created $SECRETS (0600) with a generated gateway key"
    else
        ok "$SECRETS already present — left untouched"
    fi

    if [ ! -f "$LITER_CFG" ]; then
        cat > "$LITER_CFG" <<'EOF'
# liter-llm proxy / MCP config. Generated by /liter-llm-bridge configure.
#
# liter-llm never searches $HOME for this file — ProxyConfig::discover() walks the
# CWD upward. Callers MUST pass --config <abs path> or it loads ZERO models.

[general]
# REQUIRED: /v1/* is behind an unconditional Bearer check. Without this every
# request — including /v1/models — answers 401.
master_key = "${LITER_LLM_MASTER_KEY}"
default_timeout_secs = 120
max_retries = 3

[security]
# REQUIRED for localhost base_urls: the default deny_private REFUSES loopback.
outbound_policy = "off"

[[models]]
name = "kbd-judge"
provider_model = "openai/gpt-5.6-sol"
api_key = "sk-proxy-local"
base_url = "http://localhost:8181/v1"

[[models]]
name = "kbd-critic"
provider_model = "openai/gpt-5.5"
api_key = "sk-proxy-local"
base_url = "http://localhost:8181/v1"
fallbacks = ["kbd-judge"]
EOF
        ok "created $LITER_CFG"
    else
        # Surgical append of only the missing mandatory sections.
        if ! cfg_has_master_key; then
            printf '\n[general]\nmaster_key = "${LITER_LLM_MASTER_KEY}"\n' >> "$LITER_CFG"
            ok "appended [general] master_key"
            warn "if [general] already existed, merge the duplicate table by hand"
        fi
        if cfg_uses_loopback && ! cfg_has_outbound_policy; then
            printf '\n[security]\noutbound_policy = "off"\n' >> "$LITER_CFG"
            ok "appended [security] outbound_policy (deny_private was blocking loopback)"
        fi
        cfg_has_master_key && cfg_has_outbound_policy && ok "$LITER_CFG already complete"
    fi

    if [ ! -f "$KBD_MODELS" ]; then
        cat > "$KBD_MODELS" <<'EOF'
# KBD adversarial role map. Repoint a role HERE — never by editing a pack script,
# and never inside a plugin cache (those copies are overwritten on next install).

[gateway]
candidates = ["http://localhost:8181/v1", "http://localhost:4000/v1"]

[roles]
# The producer is the harness itself, so this is intentionally not a [[models]]
# entry. It exists so the collision check has something concrete to compare
# against instead of the literal "unknown", which made the check pass trivially.
generator = "kbd-frontier"
critic = "kbd-critic"
judge = "kbd-judge"
EOF
        ok "created $KBD_MODELS"
    else
        ok "$KBD_MODELS already present — left untouched"
    fi

    # A config where critic == generator defeats the whole point.
    _c="$(grep -E '^[[:space:]]*critic[[:space:]]*=' "$KBD_MODELS" 2>/dev/null | sed -E 's/.*"([^"]*)".*/\1/')"
    _g="$(grep -E '^[[:space:]]*generator[[:space:]]*=' "$KBD_MODELS" 2>/dev/null | sed -E 's/.*"([^"]*)".*/\1/')"
    if [ -n "$_c" ] && [ "$_c" = "$_g" ]; then
        err "critic == generator ($_c) — the critic would share the producer's blind spots"
        err "edit $KBD_MODELS so they differ"
        return 1
    fi

    say ""
    say "Next: source the secrets, then verify:"
    say "  set -a; . $SECRETS; set +a"
    say "  $0 verify"
}

# --- add-provider -----------------------------------------------------------
cmd_add_provider() {
    _p="${1:-}"
    if [ -z "$_p" ]; then
        err "usage: $0 add-provider <$(provider_names | tr ' ' '|')>"
        return 2
    fi
    _row="$(provider_row "$_p")" || { err "unknown provider: $_p"; return 2; }

    _name="$(printf '%s' "$_row" | cut -d'|' -f1)"
    _pm="$(printf '%s'   "$_row" | cut -d'|' -f2)"
    _url="$(printf '%s'  "$_row" | cut -d'|' -f3)"
    _var="$(printf '%s'  "$_row" | cut -d'|' -f4)"

    [ -f "$LITER_CFG" ] || { err "no config at $LITER_CFG — run: $0 repair"; return 1; }
    if cfg_has_model "$_name"; then
        warn "$_name already declared in $LITER_CFG — not modifying it"
        return 0
    fi

    if [ -n "$_var" ]; then
        _key_line="api_key = \"\${$_var}\""
        eval "val=\${$_var:-}"
        if [ -z "$val" ]; then
            warn "\$$_var is not set. Add it to $SECRETS, then re-source:"
            warn "  echo 'export $_var=\"<your-key>\"' >> $SECRETS"
            warn "liter-llm expands an unset \${VAR} to \"\" — the request would 401."
        fi
    else
        # openai-proxy needs no inbound key, but liter-llm always sends the header,
        # so the value must be present and non-empty.
        _key_line='api_key = "sk-proxy-local"'
    fi

    cat >> "$LITER_CFG" <<EOF

[[models]]
name = "$_name"
provider_model = "$_pm"
$_key_line
base_url = "$_url"
EOF
    ok "added [[models]] $_name -> $_pm @ $_url"
    case "$_p" in
      glm-coding) say "  (GLM Coding Plan path — /api/coding/paas/v4 draws subscription quota)" ;;
    esac
    say "  point a role at it:  judge = \"$_name\"  in $KBD_MODELS"
}

# --- verify -----------------------------------------------------------------
# Never claim success on an unverified write: send one real completion per role.
cmd_verify() {
    _gw="$(gateway_url)" || { err "no gateway reachable — start openai-proxy or liter-llm api"; return 1; }
    ok "gateway: $_gw"

    _u="$(unset_referenced_vars)"
    if [ -n "$_u" ]; then
        for v in $_u; do err "\${$v} unset — fix before verifying"; done
        return 1
    fi

    _tok="${LITER_LLM_MASTER_KEY:-${OPENAI_API_KEY:-sk-local}}"
    _code="$(curl -s -o /dev/null -w '%{http_code}' --max-time 10 --noproxy '*' \
              -H "Authorization: Bearer $_tok" "$_gw/models" 2>/dev/null)"
    if [ "$_code" != "200" ]; then
        err "GET $_gw/models -> HTTP $_code (expected 200)"
        [ "$_code" = "401" ] && err "  the gateway rejected the token: check master_key vs \$LITER_LLM_MASTER_KEY"
        return 1
    fi
    ok "GET /v1/models -> 200"

    _fail=0
    for r in judge critic; do
        _m="$(grep -E "^[[:space:]]*$r[[:space:]]*=" "$KBD_MODELS" 2>/dev/null \
              | head -1 | sed -E 's/^[^=]*=[[:space:]]*"?([^"]*)"?.*/\1/')"
        [ -n "$_m" ] || { warn "$r unresolved in $KBD_MODELS"; continue; }
        _body="$(printf '{"model":"%s","messages":[{"role":"user","content":"ok"}],"max_tokens":1}' "$_m")"
        _out="$(curl -s --max-time 90 --noproxy '*' -H "Authorization: Bearer $_tok" \
                 -H 'content-type: application/json' --data-binary "$_body" \
                 "$_gw/chat/completions" 2>/dev/null)"
        if printf '%s' "$_out" | grep -q '"choices"'; then
            ok "$r ($_m) responded"
        else
            err "$r ($_m) failed: $(printf '%s' "$_out" | head -c 160)"
            _fail=1
        fi
    done
    return $_fail
}

# --- migrate ----------------------------------------------------------------
# Retire the invented config.toml. Not deleted: renamed with a pointer, so the
# old aliases stay readable while nothing reads the file any more.
cmd_migrate() {
    [ -f "$LEGACY_CFG" ] || { ok "no legacy $LEGACY_CFG — nothing to migrate"; return 0; }
    say "legacy aliases found in $LEGACY_CFG:"
    grep -E '^[[:space:]]*[a-z_]+[[:space:]]*=' "$LEGACY_CFG" 2>/dev/null | sed 's/^/    /'
    say ""
    say "These were read by adversarial-review only. Roles now live in $KBD_MODELS"
    say "and model definitions in $LITER_CFG."
    _dest="${LEGACY_CFG}.superseded"
    {
        echo "# SUPERSEDED $(date -u +%Y-%m-%dT%H:%M:%SZ)."
        echo "# The [endpoint]/[aliases] shape here is NOT a schema liter-llm can load."
        echo "# Roles: $KBD_MODELS   Models: $LITER_CFG"
        echo "#"
        cat "$LEGACY_CFG"
    } > "$_dest"
    rm -f "$LEGACY_CFG"
    ok "moved -> $_dest"
}

case "${1:-}" in
  check)         shift; cmd_check "$@" ;;
  repair)        shift; cmd_repair "$@" ;;
  add-provider)  shift; cmd_add_provider "$@" ;;
  verify)        shift; cmd_verify "$@" ;;
  migrate)       shift; cmd_migrate "$@" ;;
  ""|-h|--help)
    sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
    say ""
    say "providers: $(provider_names)"
    ;;
  *) err "unknown command: $1"; exit 2 ;;
esac
