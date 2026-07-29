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
probe_endpoint() {
  curl -s -o /dev/null --max-time 5 "$1/models" 2>/dev/null
}

JUDGE_BASE_URL="${LITER_LLM_BASE_URL:-}"
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
CONFIG="${LITER_LLM_CONFIG:-$HOME/.config/liter-llm/config.toml}"
PRODUCER="$(python3 - "$PACKET" <<'PY' 2>/dev/null || echo unknown
import json, sys
print(json.load(open(sys.argv[1])).get("producer_model") or "unknown")
PY
)"

# Judge class is frontier. Candidate order: frontier alias, then any other
# distinct alias value (medium, small) as a collision escape hatch — an
# imperfect-tier different model beats a same-model self-grade.
JUDGE_MODEL="$(PRODUCER="$PRODUCER" python3 - "$CONFIG" <<'PY' 2>/dev/null || true
import os, re, sys
config, producer = sys.argv[1], os.environ.get("PRODUCER", "unknown")
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
order = [aliases.get(k) for k in ("frontier", "medium", "small") if aliases.get(k)]
if not order:
    print("frontier")          # no alias table: pass the class name through
    raise SystemExit
for cand in order:
    # producer may be a bare model id or provider/model — compare loosely
    if cand != producer and cand.split("/")[-1] != producer.split("/")[-1]:
        print(cand)
        raise SystemExit
print("COLLISION:" + order[0])
PY
)"
[ -n "$JUDGE_MODEL" ] || JUDGE_MODEL="frontier"

case "$JUDGE_MODEL" in
  COLLISION:*)
    JUDGE_MODEL="${JUDGE_MODEL#COLLISION:}"
    echo "[judge] WARN: JUDGE_MODEL_COLLISION — every configured model matches producer" >&2
    echo "[judge]       ($PRODUCER); proceeding same-model. Configure a second provider" >&2
    echo "[judge]       to restore the cross-model guarantee." >&2
    ;;
esac

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

AUTH_HEADER="Authorization: Bearer ${OPENAI_API_KEY:-${LITER_LLM_MASTER_KEY:-sk-local}}"

RESPONSE="$(curl -s --max-time "${ADV_JUDGE_TIMEOUT:-300}" \
  "$JUDGE_BASE_URL/chat/completions" \
  -H 'content-type: application/json' -H "$AUTH_HEADER" \
  --data-binary "$REQ_BODY" 2>/dev/null)" || {
  echo "[judge] WARN: judge request to $JUDGE_BASE_URL failed — unavailable (exit 3)" >&2
  exit 3
}

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
FINDINGS="$(ADV_RAW="$RAW" JUDGE_MODEL="$JUDGE_MODEL" MODE="$MODE" python3 <<'PY'
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

out = {
    "mode": os.environ["MODE"],
    "verdict": "BLOCK" if any(f["severity"] == "CRITICAL" for f in findings) else "PASS",
    "judge_model": os.environ["JUDGE_MODEL"],
    "isolation_mode": "liter-llm",
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
