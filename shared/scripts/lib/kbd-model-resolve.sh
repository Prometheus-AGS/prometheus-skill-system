#!/usr/bin/env bash
# kbd-model-resolve.sh — single source of truth for KBD adversarial model routing.
# Source this file in any script that needs a role's model or the gateway URL:
#   RESOLVE_LIB="$(dirname "$0")/../../../../shared/scripts/lib/kbd-model-resolve.sh"
#   [ -f "$RESOLVE_LIB" ] && . "$RESOLVE_LIB"
#
# Functions exposed:
#   kbd_resolve_gateway              -> prints base URL (e.g. http://localhost:8181/v1)
#   kbd_resolve_role   <role>        -> prints model name for generator|critic|judge
#   kbd_resolve_source <role>        -> prints which layer supplied it (for diagnostics)
#   kbd_gateway_auth                 -> prints the Bearer token to use
#   kbd_model_routing_log <phase> <class> <model> <producer>
#
# WHY THIS FILE EXISTS
# Before it, the same TOML-regex parser was duplicated in dispatch-judge.sh and
# preflight-models.sh, and both read a hand-written ~/.config/liter-llm/config.toml
# whose [endpoint]/[aliases] shape liter-llm cannot load. That mismatch is why the
# judge silently fell back to passing the literal string "frontier" as a model id.
# Keep precedence in ONE place so a config change never requires a script edit —
# and never an edit inside a plugin cache, which the next install destroys.
#
# Resolution order per role (AWS-CLI convention, highest wins):
#   1. explicit argument passed by the caller
#   2. PROMETHEUS_KBD_<ROLE>_MODEL
#   3. ~/.prometheus/kbd/models.toml   [roles]
#   4. .kbd-orchestrator/project.json  model_policy  (repo-local, lower precedence)
#   5. built-in default
#
# bash 3.2 compatible: no mapfile, no declare -A, no ${var^^}. macOS /bin/bash is
# 3.2 and launchd invokes it directly, where those constructs fail with exit 127.

_KBD_MODELS_TOML="${PROMETHEUS_KBD_MODELS_CONFIG:-${HOME}/.prometheus/kbd/models.toml}"
_KBD_LITER_CONFIG="${LITER_LLM_CONFIG:-${HOME}/.config/liter-llm/liter-llm-proxy.toml}"

# Built-in defaults — used only when nothing else resolves. These are model NAMES
# that must exist as [[models]] entries in liter-llm-proxy.toml.
_KBD_DEFAULT_JUDGE="kbd-judge"
_KBD_DEFAULT_CRITIC="kbd-critic"
_KBD_DEFAULT_GENERATOR="kbd-frontier"

# ---------------------------------------------------------------------------
# kbd_toml_get <file> <section> <key>
# Minimal TOML scalar read for a [section] key = "value". Deliberately not a
# general TOML parser: it only needs flat string values from files we generate.
# Returns empty (non-zero) when absent.
kbd_toml_get() {
    _f="$1"; _sec="$2"; _key="$3"
    [ -f "$_f" ] || return 1
    awk -v sec="$_sec" -v key="$_key" '
        /^[[:space:]]*#/ { next }
        /^[[:space:]]*\[/ {
            line=$0
            gsub(/^[[:space:]]*\[+/, "", line)
            gsub(/\]+[[:space:]]*$/, "", line)
            cur=line
            next
        }
        cur == sec {
            # match:  key = "value"   /   key = value
            if ($0 ~ "^[[:space:]]*" key "[[:space:]]*=") {
                sub(/^[^=]*=[[:space:]]*/, "", $0)
                gsub(/^"|"[[:space:]]*$/, "", $0)
                gsub(/[[:space:]]*$/, "", $0)
                sub(/[[:space:]]*#.*$/, "", $0)
                gsub(/"/, "", $0)
                print $0
                exit
            }
        }
    ' "$_f"
}

# ---------------------------------------------------------------------------
# kbd_model_declared <name> — is <name> a [[models]] entry in liter-llm's config?
# Used to fail loudly on a role pointing at a model liter-llm cannot route,
# instead of discovering it as a 404 mid-review.
kbd_model_declared() {
    [ -f "$_KBD_LITER_CONFIG" ] || return 1
    grep -qE "^[[:space:]]*name[[:space:]]*=[[:space:]]*\"$1\"[[:space:]]*$" "$_KBD_LITER_CONFIG"
}

# ---------------------------------------------------------------------------
# kbd_resolve_gateway — first candidate answering GET /v1/models wins.
# LITER_LLM_BASE_URL short-circuits the probe entirely.
kbd_resolve_gateway() {
    if [ -n "${LITER_LLM_BASE_URL:-}" ]; then
        printf '%s\n' "$LITER_LLM_BASE_URL"
        return 0
    fi

    _cands="$(kbd_toml_get "$_KBD_MODELS_TOML" "gateway" "candidates" 2>/dev/null)"
    if [ -z "$_cands" ]; then
        _cands="http://localhost:8181/v1 http://localhost:4000/v1"
    else
        # "[a, b]" -> "a b"
        _cands="$(printf '%s' "$_cands" | tr -d '[]' | tr ',' ' ')"
    fi

    for _c in $_cands; do
        [ -n "$_c" ] || continue
        # --noproxy '*': a corporate HTTP(S)_PROXY must never intercept loopback.
        if curl -s -o /dev/null --max-time 5 --noproxy '*' "$_c/models" 2>/dev/null; then
            printf '%s\n' "$_c"
            return 0
        fi
    done
    return 1
}

# ---------------------------------------------------------------------------
# kbd_gateway_auth — the Bearer token for the gateway.
# liter-llm requires a token on every /v1/* route; openai-proxy ignores it but
# still needs the header present. Order mirrors what each server accepts.
kbd_gateway_auth() {
    if [ -n "${LITER_LLM_MASTER_KEY:-}" ]; then
        printf '%s\n' "$LITER_LLM_MASTER_KEY"
    elif [ -n "${OPENAI_API_KEY:-}" ]; then
        printf '%s\n' "$OPENAI_API_KEY"
    else
        printf '%s\n' "sk-local"
    fi
}

# ---------------------------------------------------------------------------
# _kbd_role_env <role> — PROMETHEUS_KBD_JUDGE_MODEL for "judge", etc.
# tr instead of ${var^^} for bash 3.2.
_kbd_role_env() {
    _upper="$(printf '%s' "$1" | tr '[:lower:]' '[:upper:]')"
    eval "printf '%s\n' \"\${PROMETHEUS_KBD_${_upper}_MODEL:-}\""
}

# ---------------------------------------------------------------------------
# _kbd_role_from_project_json <role> — lower-precedence repo-local layer.
# Five skills declare `policy_source: .kbd-orchestrator/project.json ->
# model_policy` but nothing ever read it, so phase logs recorded "project.json
# has no model_policy block" and silently defaulted. Honour it if present.
_kbd_role_from_project_json() {
    _pj="${KBD_ROOT:-.}/.kbd-orchestrator/project.json"
    [ -f "$_pj" ] || return 1
    ROLE="$1" python3 - "$_pj" <<'PY' 2>/dev/null
import json, os, sys
role = os.environ["ROLE"]
try:
    with open(sys.argv[1]) as fh:
        doc = json.load(fh)
except Exception:
    sys.exit(1)
mp = doc.get("model_policy") or {}
# Prefer an explicit roles map; fall back to registry[class][active_environment].
name = (mp.get("roles") or {}).get(role)
if not name:
    cls = {"judge": "frontier", "critic": "frontier", "generator": "frontier"}.get(role)
    env = mp.get("active_environment")
    if cls and env:
        name = ((mp.get("registry") or {}).get(cls) or {}).get(env)
if not name:
    sys.exit(1)
print(name)
PY
}

# ---------------------------------------------------------------------------
# kbd_resolve_role <role> [explicit]
kbd_resolve_role() {
    _role="$1"; _explicit="${2:-}"

    if [ -n "$_explicit" ]; then printf '%s\n' "$_explicit"; return 0; fi

    _v="$(_kbd_role_env "$_role")"
    if [ -n "$_v" ]; then printf '%s\n' "$_v"; return 0; fi

    _v="$(kbd_toml_get "$_KBD_MODELS_TOML" "roles" "$_role" 2>/dev/null)"
    if [ -n "$_v" ]; then printf '%s\n' "$_v"; return 0; fi

    _v="$(_kbd_role_from_project_json "$_role" 2>/dev/null)"
    if [ -n "$_v" ]; then printf '%s\n' "$_v"; return 0; fi

    case "$_role" in
        judge)     printf '%s\n' "$_KBD_DEFAULT_JUDGE" ;;
        critic)    printf '%s\n' "$_KBD_DEFAULT_CRITIC" ;;
        generator) printf '%s\n' "$_KBD_DEFAULT_GENERATOR" ;;
        *)         return 1 ;;
    esac
}

# ---------------------------------------------------------------------------
# kbd_resolve_source <role> — which layer supplied the value. Diagnostics only;
# preflight prints this so "where did that model come from?" is never a guess.
kbd_resolve_source() {
    _role="$1"
    [ -n "$(_kbd_role_env "$_role")" ] && { printf 'env\n'; return 0; }
    [ -n "$(kbd_toml_get "$_KBD_MODELS_TOML" "roles" "$_role" 2>/dev/null)" ] \
        && { printf '%s\n' "$_KBD_MODELS_TOML"; return 0; }
    [ -n "$(_kbd_role_from_project_json "$_role" 2>/dev/null)" ] \
        && { printf 'project.json\n'; return 0; }
    printf 'built-in-default\n'
}

# ---------------------------------------------------------------------------
# kbd_model_routing_log <phase> <class> <model> <producer>
# Preserves the existing [MODEL_ROUTING] stderr format other tooling greps for.
kbd_model_routing_log() {
    printf '[MODEL_ROUTING] phase=%s class=%s model=%s producer=%s\n' \
        "$1" "$2" "$3" "$4" >&2
}
