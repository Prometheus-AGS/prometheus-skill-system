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

CONFIG="${LITER_LLM_CONFIG:-$HOME/.config/liter-llm/config.toml}"

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

aliases = {}
if os.path.exists(config):
    section = None
    for line in open(config, encoding="utf-8", errors="replace"):
        line = line.strip()
        m = re.match(r"^\[(.+)\]$", line)
        if m:
            section = m.group(1)
            continue
        if section == "aliases":
            m = re.match(r'^(\w+)\s*=\s*"([^"]+)"', line)
            if m:
                aliases[m.group(1)] = m.group(2)

distinct = len(set(aliases.values()))

if status != "unavailable":
    if not providers and not aliases:
        status = "no_providers"
    elif not aliases:
        status = "needs_configure"
    elif distinct < 2:
        status = "degraded"

report = {
    "status": status,
    "providers_detected": providers,
    "classes_available": classes_available,
    "aliases": aliases,
    "distinct_models": distinct,
    "config_path": config,
    "config_exists": os.path.exists(config),
    "checked_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
}
print(json.dumps(report, indent=2))

hints = {
    "unavailable": "liter-llm binary not found — run /liter-llm-bridge install "
                   "(cross-model judging degrades to harness-native fallback until then)",
    "no_providers": "no provider API keys found in the environment — ask the user which "
                    "providers to configure and instruct them to export the key env var "
                    "(see liter-llm-bridge references/provider-env-vars.md); never collect "
                    "or store key values",
    "needs_configure": "provider keys detected but no [aliases] table in config — run "
                       "/liter-llm-bridge configure to generate it (fills gaps only, never "
                       "overwrites pinned aliases)",
    "degraded": "only one distinct model configured — judge may equal producer "
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
