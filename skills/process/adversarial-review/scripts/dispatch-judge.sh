#!/usr/bin/env bash
# dispatch-judge.sh — send the review packet to a fresh-context LLM judge via
# liter-llm, enforcing judge != producer wherever an alternative model exists.
#
# Usage:
#   dispatch-judge.sh --mode diff|artifact --packet <packet.json> \
#     [--mandate <mandate.md>] [--feedback <rejection.md>] [--out <findings.json>]
#
# Exit codes:
#   0  findings written (isolation_mode=liter-llm)
#   2  judge responded but output failed schema-shape validation
#   3  liter-llm unavailable — caller must fall back to a harness-native
#      fresh-context subagent (mandate + packet ONLY) and record
#      isolation_mode=harness-native
#   4  no judge possible — caller records adversarial_review: SKIPPED
#
# bash 3.2 compatible (no mapfile, no declare -A).
set -uo pipefail

MODE="" PACKET="" MANDATE="" FEEDBACK="" OUT=""
while [ $# -gt 0 ]; do
  case "$1" in
    --mode)     MODE="${2:-}"; shift 2 ;;
    --packet)   PACKET="${2:-}"; shift 2 ;;
    --mandate)  MANDATE="${2:-}"; shift 2 ;;
    --feedback) FEEDBACK="${2:-}"; shift 2 ;;
    --out)      OUT="${2:-}"; shift 2 ;;
    *) echo "usage: $0 --mode diff|artifact --packet <json> [--mandate <md>] [--feedback <md>] [--out <json>]" >&2; exit 4 ;;
  esac
done
case "$MODE" in diff|artifact) ;; *) echo "[judge] ERROR: --mode must be diff or artifact" >&2; exit 4 ;; esac
[ -f "$PACKET" ] || { echo "[judge] ERROR: packet not found: $PACKET" >&2; exit 4; }
command -v python3 >/dev/null 2>&1 || { echo "[judge] ERROR: python3 required" >&2; exit 4; }

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
[ -n "$MANDATE" ] || MANDATE="$SCRIPT_DIR/../assets/reviewer-mandate-$MODE.md"
[ -f "$MANDATE" ] || { echo "[judge] ERROR: mandate not found: $MANDATE" >&2; exit 4; }

# --- judge transport ----------------------------------------------------------
# The judge talks to an OpenAI-compatible /v1/chat/completions endpoint. That is
# the stable contract: `liter-llm` ships `api` and `mcp` subcommands (it is a
# proxy SERVER), so the previous `liter-llm complete` call could never succeed —
# and because the guard only checked that the BINARY existed, the failure was
# reported as "liter-llm unavailable" rather than as the CLI-contract mismatch
# it actually was. Probing the endpoint keeps the diagnosis honest.
#
# LITER_LLM_BASE_URL overrides; otherwise try the local openai-proxy, then a
# liter-llm `api` server on its default port.
# Endpoint + model resolution live in ONE shared library so a config change never
# requires editing this script (and never an edit inside a plugin cache, which the
# next install silently destroys). See shared/scripts/lib/kbd-model-resolve.sh.
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

probe_endpoint() {
  # --noproxy '*': an ambient HTTP(S)_PROXY must never intercept loopback.
  curl -s -o /dev/null --max-time 5 --noproxy '*' "$1/models" 2>/dev/null
}

JUDGE_BASE_URL="${LITER_LLM_BASE_URL:-}"
if [ -z "$JUDGE_BASE_URL" ] && command -v kbd_resolve_gateway >/dev/null 2>&1; then
  JUDGE_BASE_URL="$(kbd_resolve_gateway 2>/dev/null || true)"
fi
if [ -z "$JUDGE_BASE_URL" ]; then
  for cand in "http://localhost:8181/v1" "http://localhost:4000/v1"; do
    if probe_endpoint "$cand"; then JUDGE_BASE_URL="$cand"; break; fi
  done
fi

if [ -z "$JUDGE_BASE_URL" ]; then
  echo "[judge] WARN: no OpenAI-compatible endpoint reachable (set" >&2
  echo "[judge]       LITER_LLM_BASE_URL, or start one) — fall back to a" >&2
  echo "[judge]       harness-native fresh-context subagent (prompt = mandate +" >&2
  echo "[judge]       packet, nothing else) and record isolation_mode=harness-native" >&2
  exit 3
fi

# --- model resolution + collision check --------------------------------------
PRODUCER="$(python3 - "$PACKET" <<'PY' 2>/dev/null || echo unknown
import json, sys
print(json.load(open(sys.argv[1])).get("producer_model") or "unknown")
PY
)"

# Resolve judge, then critic as the collision escape hatch: an imperfect-tier
# DIFFERENT model beats a same-model self-grade.
JUDGE_MODEL=""
if command -v kbd_resolve_role >/dev/null 2>&1; then
  JUDGE_MODEL="$(kbd_resolve_role judge 2>/dev/null || true)"
  _alt_model="$(kbd_resolve_role critic 2>/dev/null || true)"
fi
[ -n "$JUDGE_MODEL" ] || JUDGE_MODEL="kbd-judge"

# Loose comparison: producer may be a bare id or provider/model.
_same_model() {
  [ "${1##*/}" = "${2##*/}" ]
}

if _same_model "$JUDGE_MODEL" "$PRODUCER"; then
  if [ -n "${_alt_model:-}" ] && ! _same_model "$_alt_model" "$PRODUCER"; then
    echo "[judge] NOTE: judge model matched producer — switching to '$_alt_model'" >&2
    JUDGE_MODEL="$_alt_model"
  else
    echo "[judge] WARN: JUDGE_MODEL_COLLISION — every configured model matches producer" >&2
    echo "[judge]       ($PRODUCER); proceeding same-model. Configure a second provider" >&2
    echo "[judge]       to restore the cross-model guarantee." >&2
  fi
fi

# A producer of "unknown" makes the collision check pass trivially — every one of
# the 8 historical reviews did exactly this, so judge!=producer was never actually
# enforced. Surface it rather than let it read as a clean cross-model review.
if [ "$PRODUCER" = "unknown" ]; then
  echo "[judge] WARN: PRODUCER_UNKNOWN — packet carries no producer_model, so the" >&2
  echo "[judge]       judge!=producer check cannot be enforced for this review." >&2
fi

echo "[MODEL_ROUTING] phase=adv-review-judge class=frontier model=$JUDGE_MODEL producer=$PRODUCER" >&2

# --- build prompts ------------------------------------------------------------
SYSTEM_PROMPT="$(cat "$MANDATE")"
if [ -n "$FEEDBACK" ] && [ -f "$FEEDBACK" ]; then
  SYSTEM_PROMPT="$SYSTEM_PROMPT

## Previous report rejected — address this feedback

$(cat "$FEEDBACK")"
fi
USER_PROMPT="$(cat "$PACKET")"

# --- dispatch (fresh context: the judge sees ONLY mandate + packet) -----------
# The request body is assembled by python3, not by shell interpolation: the
# packet contains arbitrary source text (quotes, backslashes, newlines) and any
# string-built JSON would corrupt on the first awkward diff.
REQ_BODY="$(JUDGE_MODEL="$JUDGE_MODEL" SYS="$SYSTEM_PROMPT" USR="$USER_PROMPT" python3 <<'PY'
import json, os
print(json.dumps({
    "model": os.environ["JUDGE_MODEL"],
    "messages": [
        {"role": "system", "content": os.environ.get("SYS", "")},
        {"role": "user", "content": os.environ.get("USR", "")},
    ],
    "temperature": 0,
}))
PY
)" || { echo "[judge] ERROR: failed to build request body" >&2; exit 4; }

# liter-llm requires a Bearer token on every /v1/* route (a config with no
# master_key answers 401 to everything); openai-proxy ignores the value but still
# needs the header. Prefer the gateway master key over a personal OPENAI_API_KEY —
# the latter is the wrong credential for a local liter-llm front door.
if command -v kbd_gateway_auth >/dev/null 2>&1; then
  AUTH_HEADER="Authorization: Bearer $(kbd_gateway_auth)"
else
  AUTH_HEADER="Authorization: Bearer ${LITER_LLM_MASTER_KEY:-${OPENAI_API_KEY:-sk-local}}"
fi

# Capture the HTTP status separately. `curl -s` without `-f` exits 0 on 4xx/5xx,
# so the old `|| exit 3` branch was near-dead and a non-JSON 502 from a reverse
# proxy degraded to the generic "unavailable" message this script exists to avoid.
_resp_file="$(mktemp)"
HTTP_CODE="$(curl -s --max-time "${ADV_JUDGE_TIMEOUT:-300}" \
  -o "$_resp_file" -w '%{http_code}' \
  --noproxy '*' \
  "$JUDGE_BASE_URL/chat/completions" \
  -H 'content-type: application/json' -H "$AUTH_HEADER" \
  --data-binary "$REQ_BODY" 2>/dev/null)" || HTTP_CODE="000"
RESPONSE="$(cat "$_resp_file" 2>/dev/null)"
rm -f "$_resp_file"

if [ "$HTTP_CODE" = "000" ]; then
  echo "[judge] WARN: judge request to $JUDGE_BASE_URL did not complete (connection" >&2
  echo "[judge]       failed or timed out after ${ADV_JUDGE_TIMEOUT:-300}s) — unavailable (exit 3)" >&2
  exit 3
fi

case "$HTTP_CODE" in
  2*) ;;
  401|403)
    echo "[judge] ERROR: gateway rejected the credential (HTTP $HTTP_CODE) at $JUDGE_BASE_URL." >&2
    echo "[judge]        liter-llm requires [general] master_key (or [[keys]]) and a matching" >&2
    echo "[judge]        Bearer token. Repair with: /liter-llm-bridge configure" >&2
    echo "[judge]        body: $(printf '%s' "$RESPONSE" | head -c 200)" >&2
    exit 3
    ;;
  *)
    echo "[judge] ERROR: gateway returned HTTP $HTTP_CODE at $JUDGE_BASE_URL" >&2
    echo "[judge]        body: $(printf '%s' "$RESPONSE" | head -c 200)" >&2
    exit 3
    ;;
esac

# Surface the endpoint's own error text. A silent empty completion here reads as
# "the judge found nothing", which is the most dangerous possible failure mode
# for a review gate — it turns an outage into a false all-clear.
RAW="$(RESPONSE="$RESPONSE" python3 <<'PY'
import json, os, sys
try:
    data = json.loads(os.environ.get("RESPONSE", ""))
except Exception:
    sys.exit(1)
if isinstance(data, dict) and data.get("error"):
    err = data["error"]
    msg = err.get("message") if isinstance(err, dict) else str(err)
    print(f"[judge-endpoint-error] {msg}", file=sys.stderr)
    sys.exit(1)
try:
    print(data["choices"][0]["message"]["content"])
except Exception:
    sys.exit(1)
PY
)" || {
  echo "[judge] WARN: judge returned no usable completion — unavailable (exit 3)" >&2
  exit 3
}

# --- normalize + shape-check the findings -------------------------------------
FINDINGS="$(ADV_RAW="$RAW" JUDGE_MODEL="$JUDGE_MODEL" MODE="$MODE" \
  PRODUCER="$PRODUCER" JUDGE_BASE_URL="$JUDGE_BASE_URL" python3 <<'PY'
import json, os, re, sys
raw = os.environ.get("ADV_RAW", "")

def extract(text):
    try:
        return json.loads(text)
    except Exception:
        pass
    m = re.search(r"```(?:json)?\s*(\{.*?\})\s*```", text, re.S)
    if m:
        try:
            return json.loads(m.group(1))
        except Exception:
            pass
    start = text.find("{")
    if start >= 0:
        try:
            return json.loads(text[start:text.rfind("}") + 1])
        except Exception:
            pass
    return None

data = extract(raw)
if not isinstance(data, dict) or "findings" not in data:
    sys.stderr.write("[judge] ERROR: judge output is not shape-valid findings JSON\n")
    raise SystemExit(1)

sev_ok = {"CRITICAL", "WARNING", "SUGGESTION"}
findings = []
for f in data.get("findings") or []:
    if not isinstance(f, dict):
        continue
    if f.get("severity") not in sev_ok:
        continue
    if not f.get("claim") or not f.get("evidence"):
        continue
    findings.append(f)

# isolation_mode must describe what ACTUALLY answered, not what we hoped would.
# It was previously the hardcoded literal "liter-llm" regardless of endpoint, so a
# same-family self-grade was indistinguishable from a genuine cross-model review in
# the stored artifact. Record the real endpoint, and state plainly whether the
# judge was verified distinct from the producer.
producer = os.environ.get("PRODUCER", "unknown")
judge = os.environ["JUDGE_MODEL"]
endpoint = os.environ.get("JUDGE_BASE_URL", "")

if producer == "unknown":
    cross = "unverified-producer-unknown"
elif judge.rsplit("/", 1)[-1] == producer.rsplit("/", 1)[-1]:
    cross = "same-model-collision"
else:
    cross = "verified-distinct"

out = {
    "mode": os.environ["MODE"],
    "verdict": "BLOCK" if any(f["severity"] == "CRITICAL" for f in findings) else "PASS",
    "judge_model": judge,
    "producer_model": producer,
    "isolation_mode": "rest-gateway:%s" % endpoint if endpoint else "rest-gateway",
    "cross_model_check": cross,
    "findings": findings,
}
# A zero-finding report must carry its due-diligence trail (mandate rule);
# the anti-theater gate rejects empty findings without checked_classes.
checked = data.get("checked_classes")
if isinstance(checked, list):
    out["checked_classes"] = [str(c) for c in checked if c]
print(json.dumps(out, indent=2))
PY
)" || { echo "[judge] ERROR: unusable judge output" >&2; exit 2; }

if [ -n "$OUT" ]; then
  mkdir -p "$(dirname "$OUT")"
  printf '%s\n' "$FINDINGS" > "$OUT"
  echo "[judge] wrote $OUT" >&2
else
  printf '%s\n' "$FINDINGS"
fi
