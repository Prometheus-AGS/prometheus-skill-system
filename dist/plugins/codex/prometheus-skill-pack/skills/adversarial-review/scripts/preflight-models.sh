#!/usr/bin/env bash
# preflight-models.sh — verify liter-llm is installed and configured with
# enough distinct models for cross-model judging (judge != producer).
#
# Detects provider keys from canonical env vars (delegating to
# liter-llm-bridge's detect-providers.sh when present), checks
# ~/.config/liter-llm/config.toml for an [aliases] table, and caches the
# result at .kbd-orchestrator/model-preflight.json.
#
# NEVER writes config.toml and NEVER touches API keys — config generation is
# /liter-llm-bridge configure's job; keys stay in the environment.
#
# Output: the preflight JSON on stdout. Exit 0 always (the status field
# carries the verdict) — preflight must never block the pipeline.
#
# status: ok | degraded | needs_configure | no_providers | unavailable
#
# bash 3.2 compatible (no mapfile, no declare -A).
set -uo pipefail

FORCE=0
[ "${1:-}" = "--force" ] && FORCE=1

# --- locate .kbd-orchestrator (walk up from cwd) ------------------------------
find_kbd_root() {
  local d="$PWD"
  while [ "$d" != "/" ]; do
    [ -d "$d/.kbd-orchestrator" ] && { printf '%s' "$d/.kbd-orchestrator"; return 0; }
    d="$(dirname "$d")"
  done
  return 1
}
KBD_ROOT="$(find_kbd_root 2>/dev/null || true)"
CACHE=""
[ -n "$KBD_ROOT" ] && CACHE="$KBD_ROOT/model-preflight.json"

# liter-llm's REAL config. The previous default here was
# ~/.config/liter-llm/config.toml with a flat [aliases] TABLE — a shape liter-llm
# cannot load (its schema is an [[aliases]] ARRAY in liter-llm-proxy.toml), so the
# parse silently produced nothing and the judge fell back to a literal class name.
CONFIG="${LITER_LLM_CONFIG:-$HOME/.config/liter-llm/liter-llm-proxy.toml}"

# Role resolution comes from the shared library so precedence lives in one place.
RESOLVE_LIB=""
for _cand_lib in \
  "$(cd "$(dirname "$0")" && pwd)/../../../../shared/scripts/lib/kbd-model-resolve.sh" \
  "${CLAUDE_PLUGIN_ROOT:-}/shared/scripts/lib/kbd-model-resolve.sh" \
  "${PLUGIN_ROOT:-}/shared/scripts/lib/kbd-model-resolve.sh"; do
  if [ -n "$_cand_lib" ] && [ -f "$_cand_lib" ]; then RESOLVE_LIB="$_cand_lib"; break; fi
done
if [ -n "$RESOLVE_LIB" ]; then
  # shellcheck source=/dev/null
  . "$RESOLVE_LIB"
fi

# Resolve the three roles up front so the report can state, per role, both the
# model and WHICH layer supplied it — "where did that model come from?" should
# never require reading a script.
ADV_ROLE_JUDGE=""; ADV_ROLE_CRITIC=""; ADV_ROLE_GENERATOR=""
ADV_SRC_JUDGE=""; ADV_SRC_CRITIC=""; ADV_SRC_GENERATOR=""
ADV_GATEWAY=""
if command -v kbd_resolve_role >/dev/null 2>&1; then
  ADV_ROLE_JUDGE="$(kbd_resolve_role judge 2>/dev/null || true)"
  ADV_ROLE_CRITIC="$(kbd_resolve_role critic 2>/dev/null || true)"
  ADV_ROLE_GENERATOR="$(kbd_resolve_role generator 2>/dev/null || true)"
  ADV_SRC_JUDGE="$(kbd_resolve_source judge 2>/dev/null || true)"
  ADV_SRC_CRITIC="$(kbd_resolve_source critic 2>/dev/null || true)"
  ADV_SRC_GENERATOR="$(kbd_resolve_source generator 2>/dev/null || true)"
  ADV_GATEWAY="$(kbd_resolve_gateway 2>/dev/null || true)"
fi
export ADV_ROLE_JUDGE ADV_ROLE_CRITIC ADV_ROLE_GENERATOR
export ADV_SRC_JUDGE ADV_SRC_CRITIC ADV_SRC_GENERATOR ADV_GATEWAY

command -v python3 >/dev/null 2>&1 || { echo '{"status":"unavailable","reason":"python3 missing"}'; exit 0; }

# --- cache freshness (<24h, config not newer than cache, no --force) ----------
if [ "$FORCE" -eq 0 ] && [ -n "$CACHE" ] && [ -f "$CACHE" ]; then
  fresh="$(python3 - "$CACHE" "$CONFIG" <<'PY' 2>/dev/null || echo no
import json, os, sys, time
cache, config = sys.argv[1], sys.argv[2]
try:
    if time.time() - os.path.getmtime(cache) > 86400:
        raise SystemExit
    if os.path.exists(config) and os.path.getmtime(config) > os.path.getmtime(cache):
        raise SystemExit
    json.load(open(cache))
    print("yes")
except Exception:
    print("no")
PY
)"
  if [ "$fresh" = "yes" ]; then
    cat "$CACHE"
    exit 0
  fi
fi

echo "[MODEL_ROUTING] phase=adv-review-preflight class=small" >&2

# --- 1. binary check ----------------------------------------------------------
STATUS="ok"
command -v liter-llm >/dev/null 2>&1 || STATUS="unavailable"

# --- 2. provider detection ----------------------------------------------------
# Delegate to liter-llm-bridge's canonical scanner when available; otherwise
# inline-scan the same env var table (liter-llm-bridge
# references/provider-env-vars.md is the canonical source).
DETECT=""
for root in "${CLAUDE_PLUGIN_ROOT:-}" "${PLUGIN_ROOT:-}"; do
  [ -n "$root" ] || continue
  cand="$root/skills/process/liter-llm-bridge/scripts/detect-providers.sh"
  [ -f "$cand" ] && { DETECT="$cand"; break; }
done

if [ -n "$DETECT" ]; then
  ADV_PROVIDERS_JSON="$(bash "$DETECT" 2>/dev/null || echo '{}')"
else
  ADV_PROVIDERS_JSON="$(python3 <<'PY' 2>/dev/null || echo '{}'
import json, os
TABLE = [
    ("anthropic", "ANTHROPIC_API_KEY", ["frontier", "medium"]),
    ("openai", "OPENAI_API_KEY", ["frontier", "medium", "small"]),
    ("google", "GOOGLE_API_KEY", ["frontier", "medium"]),
    ("gemini", "GEMINI_API_KEY", ["frontier", "medium"]),
    ("groq", "GROQ_API_KEY", ["small", "medium"]),
    ("together", "TOGETHER_API_KEY", ["small", "medium"]),
    ("mistral", "MISTRAL_API_KEY", ["small", "medium", "frontier"]),
    ("cohere", "COHERE_API_KEY", ["small", "medium"]),
    ("fireworks", "FIREWORKS_API_KEY", ["small", "medium"]),
    ("openrouter", "OPENROUTER_API_KEY", ["small", "medium", "frontier"]),
    ("ollama", "OLLAMA_HOST", ["small"]),
    ("vllm", "VLLM_BASE_URL", ["small", "medium"]),
    ("lmstudio", "LMSTUDIO_BASE_URL", ["small"]),
    ("llamacpp", "LLAMA_CPP_SERVER", ["small"]),
]
providers, coverage = {}, {"small": [], "medium": [], "frontier": []}
for pid, var, classes in TABLE:
    present = bool(os.environ.get(var))
    providers[pid] = {"key_var": var, "present": present, "classes": classes}
    if present:
        for c in classes:
            coverage[c].append(pid)
print(json.dumps({"providers": providers, "coverage": coverage}))
PY
)"
fi
export ADV_PROVIDERS_JSON

# --- 3+4. config + distinct-model check, emit report -------------------------
REPORT="$(python3 - "$CONFIG" "$STATUS" <<'PY'
import json, os, re, sys, time

config, status = sys.argv[1], sys.argv[2]
try:
    detected = json.loads(os.environ.get("ADV_PROVIDERS_JSON") or "{}")
except Exception:
    detected = {}

providers = [p for p, v in detected.get("providers", {}).items() if v.get("present")]
coverage = detected.get("coverage", {})
classes_available = [c for c in ("small", "medium", "frontier") if coverage.get(c)]

# Roles come from the shared resolver (exported by the shell above), not from
# re-parsing TOML here. Each carries the layer that supplied it.
roles = {
    "judge":     {"model": os.environ.get("ADV_ROLE_JUDGE") or "",
                  "source": os.environ.get("ADV_SRC_JUDGE") or ""},
    "critic":    {"model": os.environ.get("ADV_ROLE_CRITIC") or "",
                  "source": os.environ.get("ADV_SRC_CRITIC") or ""},
    "generator": {"model": os.environ.get("ADV_ROLE_GENERATOR") or "",
                  "source": os.environ.get("ADV_SRC_GENERATOR") or ""},
}
gateway = os.environ.get("ADV_GATEWAY") or ""

# What matters is not "how many models exist" but "can the judge differ from the
# producer". Count the distinct dispatchable models (judge + critic); the generator
# is the harness itself and is never dispatched through the gateway.
dispatchable = {roles[r]["model"] for r in ("judge", "critic") if roles[r]["model"]}
distinct = len(dispatchable)

# The two omissions that made the shipped config answer 401 to everything and
# refuse loopback. Report them by name — they are the difference between "no
# config" and "a config that cannot serve a single request".
config_defects = []
if os.path.exists(config):
    try:
        raw = open(config, encoding="utf-8", errors="replace").read()
    except Exception:
        raw = ""
    if not re.search(r"(?m)^\s*master_key\s*=", raw) and not re.search(r"(?m)^\[\[keys\]\]", raw):
        config_defects.append("missing [general] master_key — every /v1/* route will 401")
    if re.search(r"(?m)^\s*base_url\s*=.*(localhost|127\.0\.0\.1)", raw) and \
       not re.search(r"(?m)^\s*outbound_policy\s*=", raw):
        config_defects.append(
            "localhost base_url without [security] outbound_policy — deny_private blocks loopback")

if status != "unavailable":
    if not gateway:
        status = "no_gateway"
    elif config_defects:
        status = "config_broken"
    elif not roles["judge"]["model"]:
        status = "needs_configure"
    elif distinct < 2:
        status = "degraded"

report = {
    "status": status,
    "gateway": gateway,
    "roles": roles,
    "providers_detected": providers,
    "classes_available": classes_available,
    "distinct_models": distinct,
    "config_path": config,
    "config_exists": os.path.exists(config),
    "config_defects": config_defects,
    "checked_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
}
print(json.dumps(report, indent=2))

hints = {
    "unavailable": "liter-llm binary not found — run /liter-llm-bridge install "
                   "(cross-model judging degrades to harness-native fallback until then)",
    "no_gateway": "no OpenAI-compatible endpoint answered. Start the local proxy "
                  "(openai-proxy on :8181) or `liter-llm api --config "
                  "~/.config/liter-llm/liter-llm-proxy.toml`, or set LITER_LLM_BASE_URL. "
                  "NOTE: liter-llm never searches $HOME for its config — always pass "
                  "--config <abs path>, or it starts with zero models",
    "config_broken": "the liter-llm config exists but cannot serve a request — see "
                     "config_defects. Repair with /liter-llm-bridge configure (merges, "
                     "never clobbers)",
    "no_providers": "no provider API keys found in the environment — ask the user which "
                    "providers to configure and instruct them to export the key env var "
                    "(see liter-llm-bridge references/provider-env-vars.md); never collect "
                    "or store key values",
    "needs_configure": "no judge role resolves — run /liter-llm-bridge configure to seed "
                       "~/.prometheus/kbd/models.toml and the matching [[models]] entries",
    "degraded": "only one distinct dispatchable model — judge may equal producer "
                "(JUDGE_MODEL_COLLISION expected); configure a second provider/model",
}
if status in hints:
    sys.stderr.write("[preflight] WARN %s: %s\n" % (status, hints[status]))
PY
)"

printf '%s\n' "$REPORT"
if [ -n "$CACHE" ]; then
  printf '%s\n' "$REPORT" > "$CACHE" 2>/dev/null || true
fi
exit 0
