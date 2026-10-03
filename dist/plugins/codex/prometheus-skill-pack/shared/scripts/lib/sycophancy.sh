#!/usr/bin/env bash
# sycophancy.sh — shared library for sycophancy detection over PMPO artifacts.
# Source this file; it does not run anything on import.
#
#   source "$(dirname "$0")/lib/sycophancy.sh"
#
# Exposes:
#   syco_find_bin                       → prints MCP binary path, or returns 1
#   syco_map_strictness <in>            → prints permissive|standard|strict
#   syco_analyze <text> <strictness>    → prints raw MCP JSON-RPC response
#   syco_score <response>               → prints sycophancy_score, or empty
#   syco_critical <response>            → prints "id:sev ..." for high/critical
#   syco_counter_path <key>             → per-artifact rejection counter path
#
# All functions degrade gracefully (empty output / non-zero) when the binary or
# python3 is unavailable; callers decide how to handle that.

syco_find_bin() {
  if command -v sycophancy-correction >/dev/null 2>&1; then
    command -v sycophancy-correction
    return 0
  fi
  local root="${CLAUDE_PLUGIN_ROOT:-}"
  [ -n "$root" ] || return 1
  local cand="${root}/skills/imported/sycophancy-correction/target/release/sycophancy-correction"
  [ -x "$cand" ] && { printf '%s' "$cand"; return 0; }
  return 1
}

syco_map_strictness() {
  case "${1:-strict}" in
    adversarial) printf 'strict' ;;
    loose)       printf 'permissive' ;;
    *)           printf '%s' "${1:-strict}" ;;
  esac
}

# syco_analyze <text> <mcp-strictness>
syco_analyze() {
  local artifact="$1" strictness="$2"
  local bin; bin="$(syco_find_bin 2>/dev/null)" || return 1
  command -v python3 >/dev/null 2>&1 || return 1

  local escaped
  escaped="$(printf '%s' "$artifact" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read()))' 2>/dev/null)"
  [ -n "$escaped" ] || return 1

  local fifo; fifo="$(mktemp -u /tmp/sycophancy_lib_XXXXX)"
  mkfifo "$fifo" || return 1
  # shellcheck disable=SC2064
  trap "rm -f '$fifo'" RETURN

  local skill_toml="" root="${CLAUDE_PLUGIN_ROOT:-}"
  if [ -n "$root" ] && [ -f "${root}/skills/imported/sycophancy-correction/skill.toml" ]; then
    skill_toml="${root}/skills/imported/sycophancy-correction/skill.toml"
  fi

  (
    printf '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"sycophancy-lib","version":"0.1.0"}}}\n'
    sleep 0.2
    printf '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}\n'
    sleep 0.1
    printf '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"detect_sycophancy","arguments":{"content":%s,"target":"completion","strictness":"%s"}}}\n' \
      "$escaped" "$strictness"
    sleep 3
  ) > "$fifo" &

  local response=""
  if command -v timeout >/dev/null 2>&1; then
    if [ -n "$skill_toml" ]; then
      response="$(timeout 30 "$bin" --config "$skill_toml" < "$fifo" 2>/dev/null)" || true
    else
      response="$(timeout 30 "$bin" < "$fifo" 2>/dev/null)" || true
    fi
  else
    if [ -n "$skill_toml" ]; then
      response="$("$bin" --config "$skill_toml" < "$fifo" 2>/dev/null)" || true
    else
      response="$("$bin" < "$fifo" 2>/dev/null)" || true
    fi
  fi
  printf '%s' "$response"
}

syco_score() {
  printf '%s' "$1" | python3 -c '
import sys, json
for line in sys.stdin.read().splitlines():
    try:
        obj = json.loads(line)
        r = obj.get("result", {})
        if isinstance(r, dict):
            content = r.get("content", [])
            if content and isinstance(content, list):
                data = json.loads(content[0].get("text", "{}"))
                score = data.get("sycophancy_score", -1)
                if score >= 0:
                    print(score); break
    except Exception:
        pass
' 2>/dev/null || true
}

syco_critical() {
  printf '%s' "$1" | python3 -c '
import sys, json
for line in sys.stdin.read().splitlines():
    try:
        obj = json.loads(line)
        r = obj.get("result", {})
        if isinstance(r, dict):
            content = r.get("content", [])
            if content and isinstance(content, list):
                data = json.loads(content[0].get("text", "{}"))
                classes = data.get("classifications", [])
                hc = [c.get("pattern_id","?")+":"+c.get("severity","?")
                      for c in classes if c.get("severity","") in ("high","critical")]
                if hc:
                    print(" ".join(hc))
    except Exception:
        pass
' 2>/dev/null || true
}

# syco_should_reject <score> <high/critical-patterns> → prints "1" or "0" on
# line 1, then a human-readable reason on line 2 (empty when accepted).
#
# <high/critical-patterns> is the output of syco_critical: space-separated
# "ID:severity" tokens for every high- and critical-severity classification.
#
# Decision rule (replaces the old "reject on score>=0.4 OR ANY critical"):
#   reject iff  score >= SCORE_THRESHOLD
#          or   any high/critical pattern present AND score >= CRITICAL_FLOOR
#          or   an always-reject pattern is present (default: S-08)
#
# Rationale:
#  * The old rule rejected any single critical regardless of score. A structured
#    reflection that legitimately surfaces no caveat words produced one low-weight
#    heuristic hit (score ~0.125) and was wrongly rejected. Requiring the score to
#    clear a small CRITICAL_FLOOR stops those false rejects while still catching
#    artifacts where patterns compound past the floor.
#  * S-08 (Reflect Phase Inversion) is the purpose-built "this reflection is a
#    success summary instead of an analysis" detector. For these reflection /
#    assessment gates that is precisely the failure mode we exist to catch, so an
#    S-08 hit rejects on its own even at a modest score. This is what stops a
#    short, low-scoring "everything went perfectly" summary from sailing through.
#
# Tunable via env (all optional):
#   PROMETHEUS_SYCO_SCORE_THRESHOLD   default 0.4   (hard score gate)
#   PROMETHEUS_SYCO_CRITICAL_FLOOR    default 0.3   (min score for high/critical reject)
#   PROMETHEUS_SYCO_ALWAYS_REJECT     default S-08  (comma/space list of pattern ids
#                                                    that reject regardless of score;
#                                                    empty string disables)
#   PROMETHEUS_SYCO_CRITICAL_ALWAYS   default 0     (set 1 to restore legacy
#                                                    "any high/critical → reject")
syco_should_reject() {
  local score="$1" crit="$2"
  local threshold="${PROMETHEUS_SYCO_SCORE_THRESHOLD:-0.4}"
  local floor="${PROMETHEUS_SYCO_CRITICAL_FLOOR:-0.3}"
  local always_reject="${PROMETHEUS_SYCO_ALWAYS_REJECT-S-08}"
  local crit_always="${PROMETHEUS_SYCO_CRITICAL_ALWAYS:-0}"

  command -v python3 >/dev/null 2>&1 || { printf '0\n\n'; return 0; }
  SYCO_SCORE="$score" SYCO_CRIT="$crit" SYCO_THRESHOLD="$threshold" \
  SYCO_FLOOR="$floor" SYCO_ALWAYS="$always_reject" SYCO_CRIT_ALWAYS="$crit_always" python3 -c '
import os, re
raw = os.environ.get("SYCO_SCORE", "").strip()
crit = os.environ.get("SYCO_CRIT", "").strip()
threshold = float(os.environ.get("SYCO_THRESHOLD", "0.4"))
floor = float(os.environ.get("SYCO_FLOOR", "0.3"))
crit_always = os.environ.get("SYCO_CRIT_ALWAYS", "0").strip() == "1"
always_ids = [t for t in re.split(r"[,\s]+", os.environ.get("SYCO_ALWAYS", "")) if t]
try:
    score = float(raw) if raw else None
except ValueError:
    score = None

# Pattern ids present in the high/critical token list ("S-08:high" -> "S-08").
present_ids = {tok.split(":", 1)[0] for tok in crit.split()} if crit else set()

reasons = []
reject = False
if score is not None and score >= threshold:
    reject = True
    reasons.append(f"sycophancy_score={score:g} (>= threshold {threshold:g})")

hit_always = [i for i in always_ids if i in present_ids]
if hit_always:
    reject = True
    reasons.append(f"always-reject pattern(s) present: {{}}".format(", ".join(hit_always)))

if crit:
    if crit_always or (score is not None and score >= floor):
        reject = True
        why = "any-critical mode" if crit_always else f"score {score:g} >= critical-floor {floor:g}"
        reasons.append(f"high/critical [{crit}] with {why}")
    # else: lone high/critical below the floor (and not an always-reject id) is
    # reported but does NOT reject on its own.

print("1" if reject else "0")
print("; ".join(reasons))
' 2>/dev/null || printf '0\n\n'
}

# Per-artifact rejection counter so reflection and artifact gates do not share
# one global counter. Key is typically a sha1 of the artifact path.
syco_counter_path() {
  printf '%s/.prometheus/reflect-rejections/%s.txt' "$HOME" "${1:-default}"
}
