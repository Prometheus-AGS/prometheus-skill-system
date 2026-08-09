#!/usr/bin/env bash
# Assert the bootstrapped structure is intact and enforcing.
#
# PASS / FAIL / SKIP. SKIP is never counted as PASS: a check that could not run
# is unverified, and reporting it as passing is how a gate becomes decorative.
#
# Exit 0 = no FAIL. Exit 1 = at least one FAIL.

set -uo pipefail

project_path="."
while [[ $# -gt 0 ]]; do
  case "$1" in
    --path) project_path="${2:?--path requires a value}"; shift 2 ;;
    -h|--help) echo "Usage: verify.sh [--path <root>]"; exit 0 ;;
    *) echo "verify: unknown flag: $1" >&2; exit 1 ;;
  esac
done
[[ -d "$project_path" ]] || { echo "verify: not a directory: $project_path" >&2; exit 1; }
project_path="$(cd "$project_path" && pwd)"

pass=0; fail=0; skip=0; warn=0
ok()   { printf 'PASS  %-34s %s\n' "$1" "${2:-}"; pass=$((pass+1)); }
no()   { printf 'FAIL  %-34s %s\n' "$1" "${2:-}"; fail=$((fail+1)); }
sk()   { printf 'SKIP  %-34s %s\n' "$1" "${2:-}"; skip=$((skip+1)); }
# WARN is for conditions the repo cannot fix — machine-wide or environment-scoped.
# Reporting them as FAIL makes every repo red forever, and a gate that always
# fails stops being read.
wr()   { printf 'WARN  %-34s %s\n' "$1" "${2:-}"; warn=$((warn+1)); }

echo "Verifying $project_path"
echo

# --- AGENTS.md and its managed region ---
A="$project_path/AGENTS.md"
if [[ ! -f "$A" ]]; then
  no "AGENTS.md" "absent — run bootstrap.sh"
else
  s="$(grep -cF '<!-- prometheus-base:start v1 -->' "$A" 2>/dev/null; :)"
  e="$(grep -cF '<!-- prometheus-base:end -->' "$A" 2>/dev/null; :)"
  if [[ "$s" == "1" && "$e" == "1" ]]; then
    ok "AGENTS.md markers" "well-formed"
  else
    no "AGENTS.md markers" "$s start / $e end — repair by hand"
  fi

  words="$(wc -w < "$A" | tr -d ' ')"
  # Tool-owned regions carried from a migration are not part of the managed
  # region's budget — they are another tool's contract. Count them separately.
  carried="$(awk '/<!-- prometheus-base:end -->/{f=1;next} f' "$A" 2>/dev/null | wc -w | tr -d ' ')"
  managed=$(( words - carried ))
  prof="$(grep -o '<!-- profile: [a-z]* ' "$A" 2>/dev/null | head -1 | awk '{print $3}')"
  prof="${prof:-unknown}"
  case "$prof" in
    lean)  ceiling=900  ;;
    *)     ceiling=1500 ;;
  esac
  if [[ "$managed" -le "$ceiling" ]]; then
    ok "AGENTS.md size" "$managed managed words (profile $prof, ceiling $ceiling)$( [[ $carried -gt 0 ]] && printf ' + %s carried' "$carried")"
  else
    no "AGENTS.md size" "$managed managed words exceeds $ceiling for profile $prof — move detail to .claude/rules/"
  fi

  # The scaffold must match the declared profile, or the declaration is a lie.
  has_scaffold=0
  grep -qF '## Execution scaffold' "$A" 2>/dev/null && has_scaffold=1
  if [[ "$prof" == "unknown" ]]; then
    sk "profile declaration" "no profile marker — re-run bootstrap.sh"
  elif [[ "$prof" == "lean" && "$has_scaffold" == "1" ]]; then
    no "profile consistency" "declared lean but scaffold present"
  elif [[ "$prof" != "lean" && "$has_scaffold" == "0" ]]; then
    no "profile consistency" "declared $prof but scaffold absent"
  else
    ok "profile consistency" "$prof"
  fi

  if [[ "$prof" == "lean" ]]; then
    f="$project_path/.prometheus/model-fleet.md"
    if [[ -f "$f" ]] && grep -qiE '^\|[^|]+\|[^|]+\|[^|]+\| *yes' "$f" 2>/dev/null; then
      ok "lean is measured" "fleet records a measured model"
    else
      no "lean is measured" "lean profile with no measured fleet entry in .prometheus/model-fleet.md"
    fi
  fi
fi

# --- CLAUDE.md reaches AGENTS.md, and carries no second constitution ---
C="$project_path/CLAUDE.md"
if [[ -L "$C" ]]; then
  tgt="$(readlink "$C")"
  [[ "$tgt" == "AGENTS.md" ]] && ok "CLAUDE.md" "symlink -> AGENTS.md (no double load)" \
                              || no "CLAUDE.md" "symlink -> $tgt (expected AGENTS.md)"
elif [[ -f "$C" ]]; then
  if grep -q '^@AGENTS\.md[[:space:]]*$' "$C"; then
    ok "CLAUDE.md" "imports AGENTS.md"
  else
    no "CLAUDE.md" "no @AGENTS.md import line"
  fi
  # An import above a retained v3 body loads both constitutions at once. This
  # is the failure the AGENTS.md refusal exists to prevent, reached by the
  # other entry point — verify it explicitly rather than passing on the import.
  cids="$(grep -cE '^\*\*[A-G]-[0-9]+ ·' "$C" 2>/dev/null; :)"
  if [[ "${cids:-0}" -ge 5 ]]; then
    no "CLAUDE.md has no second constitution" "$cids v3 rule IDs still live — duplicate constitution"
  else
    ok "CLAUDE.md has no second constitution" ""
  fi
  cw="$(wc -w < "$C" | tr -d ' ')"
  aw="$(wc -w < "$A" 2>/dev/null | tr -d ' ')"
  total=$(( ${aw:-0} + cw ))
  # @import loads at launch, so AGENTS.md is counted twice in the effective load.
  eff=$(( total + ${aw:-0} ))
  if [[ "$total" -le 2000 ]]; then
    ok "combined resident" "$total words (effective ~$eff with double-load)"
  else
    no "combined resident" "$total words across both files (effective ~$eff) — CLAUDE.md carries $cw"
  fi
else
  no "CLAUDE.md" "absent"
fi

# --- hooks are executable, or they are decoration ---
H="$project_path/.claude/hooks"
if [[ -d "$H" ]]; then
  bad=0; n=0
  for f in "$H"/*.sh; do
    [[ -e "$f" ]] || continue
    n=$((n+1))
    [[ -x "$f" ]] || { bad=$((bad+1)); echo "        not executable: ${f#$project_path/}"; }
  done
  if [[ "$n" == "0" ]]; then
    sk "hooks executable" "no hooks installed"
  elif [[ "$bad" == "0" ]]; then
    ok "hooks executable" "$n hook(s)"
  else
    no "hooks executable" "$bad of $n not executable — chmod +x"
  fi
else
  sk "hooks executable" ".claude/hooks absent"
fi

# --- settings.json parses and wires the hooks ---
S="$project_path/.claude/settings.json"
if [[ ! -f "$S" ]]; then
  no "settings.json" "absent"
elif ! command -v jq >/dev/null 2>&1; then
  sk "settings.json parses" "jq absent — NOT verified"
elif jq -e . "$S" >/dev/null 2>&1; then
  ok "settings.json parses" ""
  jq -e '.hooks.PreToolUse' "$S" >/dev/null 2>&1 \
    && ok "tier-guard wired" "" \
    || no "tier-guard wired" "hook installed but not referenced in settings.json"
  b="$(jq -r '.skillListingBudgetFraction // "unset"' "$S" 2>/dev/null)"
  [[ "$b" == "unset" ]] \
    && no "skill budget set" "unset — 1% default may drop skill descriptions" \
    || ok "skill budget set" "$b"
else
  no "settings.json parses" "invalid JSON"
fi

# --- skill budget MEASURED across every scope, never assumed ---
# A repo-local count is the wrong denominator: user and plugin scopes dominate.
SB="$(dirname "${BASH_SOURCE[0]}")/skill-budget.sh"
if [[ ! -x "$SB" ]]; then
  sk "skill budget measured" "skill-budget.sh not executable — NOT verified"
elif ! command -v python3 >/dev/null 2>&1; then
  sk "skill budget measured" "python3 absent — NOT verified"
else
  sb_out="$("$SB" --path "$project_path" --json 2>/dev/null || true)"
  if [[ -z "$sb_out" ]] || ! command -v jq >/dev/null 2>&1; then
    sk "skill budget measured" "could not parse measurement — NOT verified"
  else
    sb_ratio="$(printf '%s' "$sb_out" | jq -r '.ratio // "?"')"
    sb_n="$(printf '%s' "$sb_out" | jq -r '.skills // 0')"
    sb_tok="$(printf '%s' "$sb_out" | jq -r '.est_tokens // 0')"
    sb_bud="$(printf '%s' "$sb_out" | jq -r '.budget_tokens // 0')"
    if [[ "$(printf '%s' "$sb_out" | jq -r '.over_budget')" == "true" ]]; then
      sb_repo="$(printf '%s' "$sb_out" | jq -r '.scopes[] | select(.scope=="repo") | .chars')"
      wr "skill budget measured" "${sb_n} skills, ~${sb_tok} tok vs ~${sb_bud} — ${sb_ratio}x OVER (machine-wide; repo contributes ${sb_repo} chars)"
    else
      ok "skill budget measured" "${sb_n} skills, ~${sb_tok} tok of ~${sb_bud}"
    fi
    sb_empty="$(printf '%s' "$sb_out" | jq -r '.empty_descriptions // 0')"
    [[ "$sb_empty" -gt 0 ]] && wr "skills can auto-trigger" "$sb_empty with empty descriptions (machine-wide)"
  fi
fi

# --- learning layer ---
P="$project_path/.prometheus"
missing=""
for e in session-log.md decisions.md gotchas.md postmortems knowledge; do
  [[ -e "$P/$e" ]] || missing="$missing $e"
done
[[ -z "$missing" ]] && ok ".prometheus layout" "complete" \
                    || no ".prometheus layout" "missing:$missing"

# --- position authority ---
W="$project_path/.kbd-orchestrator/current-waypoint.json"
if [[ ! -f "$W" ]]; then
  no "waypoint" "absent — tier-guard will assume phase=implement"
elif command -v jq >/dev/null 2>&1; then
  jq -e '.phase' "$W" >/dev/null 2>&1 \
    && ok "waypoint" "phase=$(jq -r .phase "$W")" \
    || no "waypoint" "no .phase field — tier-guard cannot read it"
else
  sk "waypoint parses" "jq absent — NOT verified"
fi

echo
printf 'PASS %d   FAIL %d   WARN %d   SKIP %d\n' "$pass" "$fail" "$warn" "$skip"
[[ "$skip" -gt 0 ]] && echo 'SKIP is not PASS. Those checks did not run.'
[[ "$warn" -gt 0 ]] && echo 'WARN is a real finding this repo cannot fix on its own. Do not ignore it.'
[[ "$fail" -gt 0 ]] && exit 1
exit 0
