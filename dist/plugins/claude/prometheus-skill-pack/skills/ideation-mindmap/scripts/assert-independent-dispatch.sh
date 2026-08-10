#!/usr/bin/env bash
# assert-independent-dispatch.sh — prove candidate sets were generated independently.
#
# Usage:
#   assert-independent-dispatch.sh --session <dir> [--min-sets N]
#
# Exit: 0 independence holds · 1 usage · 2 NOT independent
#
# WHAT THIS CHECKS, AND WHY IT IS NOT A PROMPT REVIEW
# A skill can instruct "generate independently" and a model can ignore it, with
# nothing downstream the wiser. So this reads what each dispatch ACTUALLY
# RECEIVED — the recorded inputs written by record-dispatch.sh — and asserts:
#
#   1. at least N sets exist                     (default 3)
#   2. no set's input contains another set's output
#   3. every set's input references the topic     (same question, not N questions)
#   4. outputs are not byte-identical             (a copied set is not a second sample)
#
# Reading the SKILL.md prose to confirm it says "independently" would be exactly
# the mistake this script exists to avoid: checking the instruction rather than
# the behaviour.
#
# bash 3.2 compatible. No LLM calls.
set -uo pipefail

SESSION="" MIN_SETS=3
while [ $# -gt 0 ]; do
  case "$1" in
    --session)  SESSION="${2:-}"; shift 2 ;;
    --min-sets) MIN_SETS="${2:-3}"; shift 2 ;;
    *) echo "usage: $0 --session <dir> [--min-sets N]" >&2; exit 1 ;;
  esac
done
[ -n "$SESSION" ] || { echo "[independence] ERROR: --session is required" >&2; exit 1; }
[ -d "$SESSION/sets" ] || { echo "[independence] ERROR: no sets/ under $SESSION" >&2; exit 1; }
case "$MIN_SETS" in ''|*[!0-9]*) echo "[independence] ERROR: --min-sets must be a number" >&2; exit 1 ;; esac
command -v python3 >/dev/null 2>&1 || { echo "[independence] ERROR: python3 required" >&2; exit 1; }

FAIL=0
say_fail() { echo "[independence] FAIL: $1" >&2; FAIL=1; }

# --- 1. enough sets ----------------------------------------------------------
N=0
for f in "$SESSION"/sets/set-*.input; do [ -f "$f" ] && N=$((N + 1)); done
if [ "$N" -lt "$MIN_SETS" ]; then
  say_fail "only $N recorded set(s); at least $MIN_SETS required."
  echo "[independence]   One pass is the single-sample case that diversity" >&2
  echo "[independence]   enforcement exists to replace." >&2
else
  echo "[independence] ok: $N independent sets recorded" >&2
fi

# --- 2/3/4. cross-contamination, topic anchoring, duplicate outputs ----------
RESULT="$(SESSION="$SESSION" python3 <<'PY' 2>/dev/null || echo "PYFAIL"
import glob, hashlib, os, re, sys

session = os.environ["SESSION"]
sets = {}
for p in sorted(glob.glob(os.path.join(session, "sets", "set-*.input"))):
    n = re.search(r"set-(\d+)\.input$", p).group(1)
    out = os.path.join(session, "sets", "set-%s.output" % n)
    sets[n] = {
        "input": open(p, encoding="utf-8", errors="replace").read(),
        "output": open(out, encoding="utf-8", errors="replace").read()
                  if os.path.exists(out) else None,
    }

topic = ""
tp = os.path.join(session, "topic.txt")
if os.path.exists(tp):
    topic = open(tp, encoding="utf-8", errors="replace").read().strip()

problems = []

# 2. no set's input may contain another set's output.
for a, da in sets.items():
    for b, db in sets.items():
        if a == b or not db["output"]:
            continue
        # Substantive lines only — short lines collide by chance and a false
        # positive here would be as damaging as a miss.
        lines = [l.strip() for l in db["output"].splitlines() if len(l.strip()) > 24]
        if any(l in da["input"] for l in lines):
            problems.append("set %s input contains set %s output" % (a, b))

# 3. every input must reference the topic — otherwise these are N different
#    questions, and pooling them is meaningless rather than diverse.
if topic:
    key = [w for w in re.findall(r"[A-Za-z]{4,}", topic.lower())][:3]
    for n, d in sets.items():
        low = d["input"].lower()
        if key and not any(w in low for w in key):
            problems.append("set %s input does not reference the topic" % n)

# 4. byte-identical outputs are one sample recorded twice.
digests = {}
for n, d in sets.items():
    if not d["output"]:
        continue
    h = hashlib.sha256(d["output"].encode("utf-8", "replace")).hexdigest()
    if h in digests:
        problems.append("set %s output is byte-identical to set %s" % (n, digests[h]))
    digests[h] = n

print("OK" if not problems else "\n".join(problems))
PY
)"

if [ "$RESULT" = "PYFAIL" ]; then
  say_fail "could not read the recorded dispatches."
elif [ "$RESULT" != "OK" ]; then
  printf '%s\n' "$RESULT" | while IFS= read -r line; do
    [ -n "$line" ] && echo "[independence] FAIL: $line" >&2
  done
  FAIL=1
else
  echo "[independence] ok: no cross-contamination, all sets on-topic, outputs distinct" >&2
fi

if [ "$FAIL" -eq 0 ]; then
  echo "[independence] PASS: candidate sets were generated independently." >&2
  exit 0
fi
echo "[independence] REJECTED: independence is not established, so pooling these" >&2
echo "[independence]   sets would present correlated output as diverse." >&2
exit 2
