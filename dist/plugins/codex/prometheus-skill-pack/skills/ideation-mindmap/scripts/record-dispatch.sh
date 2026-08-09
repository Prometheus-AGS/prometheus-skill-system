#!/usr/bin/env bash
# record-dispatch.sh — record ONE independent generation dispatch.
#
# Usage:
#   record-dispatch.sh --session <dir> --set <n> --topic <text> [--input <file>]
#   record-dispatch.sh --session <dir> --set <n> --topic <text> --output <file>
#
# Exit: 0 ok · 1 usage · 2 the recorded input would break independence
#
# WHY THIS EXISTS
# "Generate independently" is a claim about what each dispatch RECEIVED. Prose
# cannot carry that claim — a skill can say "do not share context" and a model
# can share it anyway, and nothing downstream would know. This script writes the
# input each dispatch actually got, so `assert-independent-dispatch.sh` can check
# the property instead of trusting the instruction.
#
# THE PROPERTY: every set's input contains the topic and NOTHING drawn from
# another set. Chen et al. (2026) found multi-agent LLM ideation collapses toward
# agreement despite architectural attempts to diversify, so the only defence is
# structural — never hand set N the output of set N-1.
#
# bash 3.2 compatible. No LLM calls.
set -uo pipefail

SESSION="" SET="" TOPIC="" INPUT="" OUTPUT=""
while [ $# -gt 0 ]; do
  case "$1" in
    --session) SESSION="${2:-}"; shift 2 ;;
    --set)     SET="${2:-}";     shift 2 ;;
    --topic)   TOPIC="${2:-}";   shift 2 ;;
    --input)   INPUT="${2:-}";   shift 2 ;;
    --output)  OUTPUT="${2:-}";  shift 2 ;;
    *) echo "usage: $0 --session <dir> --set <n> --topic <text> [--input <file>] [--output <file>]" >&2; exit 1 ;;
  esac
done
[ -n "$SESSION" ] || { echo "[dispatch] ERROR: --session is required" >&2; exit 1; }
[ -n "$SET" ]     || { echo "[dispatch] ERROR: --set is required" >&2; exit 1; }
[ -n "$TOPIC" ]   || { echo "[dispatch] ERROR: --topic is required" >&2; exit 1; }
case "$SET" in ''|*[!0-9]*) echo "[dispatch] ERROR: --set must be a number" >&2; exit 1 ;; esac

mkdir -p "$SESSION/sets" 2>/dev/null || { echo "[dispatch] ERROR: cannot create $SESSION/sets" >&2; exit 1; }

# Record the OUTPUT of a set.
if [ -n "$OUTPUT" ]; then
  [ -f "$OUTPUT" ] || { echo "[dispatch] ERROR: --output file not found: $OUTPUT" >&2; exit 1; }
  cp "$OUTPUT" "$SESSION/sets/set-$SET.output" || exit 1
  echo "[dispatch] recorded output for set $SET" >&2
  exit 0
fi

# Record the INPUT. Default: the topic alone, which is the whole point — a
# dispatch that receives only the topic cannot have been contaminated.
IN_FILE="$SESSION/sets/set-$SET.input"
if [ -n "$INPUT" ]; then
  [ -f "$INPUT" ] || { echo "[dispatch] ERROR: --input file not found: $INPUT" >&2; exit 1; }
  cp "$INPUT" "$IN_FILE" || exit 1
else
  printf '%s\n' "$TOPIC" > "$IN_FILE" || exit 1
fi

# Fail at RECORD time, not only at assert time. If a caller hands set N an input
# that already contains a prior set's output, refusing here means the
# contaminated set never enters the pool at all — the cheapest possible place to
# catch it.
CONTAMINATED=""
for prior in "$SESSION"/sets/set-*.output; do
  [ -f "$prior" ] || continue
  case "$prior" in *"set-$SET.output") continue ;; esac
  # A prior output's distinctive lines appearing verbatim in this input is the
  # signature of context leaking between dispatches.
  if python3 - "$IN_FILE" "$prior" <<'PY' 2>/dev/null; then
import sys
inp = open(sys.argv[1], encoding="utf-8", errors="replace").read()
pri = open(sys.argv[2], encoding="utf-8", errors="replace").read()
# Compare on substantive lines only: short/boilerplate lines collide by chance.
lines = [l.strip() for l in pri.splitlines() if len(l.strip()) > 24]
hits = sum(1 for l in lines if l in inp)
raise SystemExit(0 if hits else 1)
PY
    CONTAMINATED="$prior"
    break
  fi
done

if [ -n "$CONTAMINATED" ]; then
  rm -f "$IN_FILE"
  echo "[dispatch] REFUSED: the input for set $SET contains content from $(basename "$CONTAMINATED")." >&2
  echo "[dispatch]   Independent generation means each dispatch receives the TOPIC ONLY." >&2
  echo "[dispatch]   Feeding one set's output into the next is the diversity-collapse" >&2
  echo "[dispatch]   failure mode this gate exists to prevent (arXiv 2604.18005)." >&2
  exit 2
fi

printf '%s\n' "$TOPIC" > "$SESSION/topic.txt"
echo "[dispatch] recorded independent input for set $SET" >&2
exit 0
