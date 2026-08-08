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

pass=0; fail=0; skip=0
ok()   { printf 'PASS  %-34s %s\n' "$1" "${2:-}"; pass=$((pass+1)); }
no()   { printf 'FAIL  %-34s %s\n' "$1" "${2:-}"; fail=$((fail+1)); }
sk()   { printf 'SKIP  %-34s %s\n' "$1" "${2:-}"; skip=$((skip+1)); }

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
  prof="$(grep -o '<!-- profile: [a-z]* ' "$A" 2>/dev/null | head -1 | awk '{print $3}')"
  prof="${prof:-unknown}"
  case "$prof" in
    lean)  ceiling=900  ;;
    *)     ceiling=1500 ;;
  esac
  if [[ "$words" -le "$ceiling" ]]; then
    ok "AGENTS.md size" "$words words (profile $prof, ceiling $ceiling)"
  else
    no "AGENTS.md size" "$words words exceeds $ceiling for profile $prof — move detail to .claude/rules/"
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

# --- CLAUDE.md reaches AGENTS.md ---
C="$project_path/CLAUDE.md"
if [[ -L "$C" ]]; then
  tgt="$(readlink "$C")"
  [[ "$tgt" == "AGENTS.md" ]] && ok "CLAUDE.md" "symlink -> AGENTS.md" \
                              || no "CLAUDE.md" "symlink -> $tgt (expected AGENTS.md)"
elif [[ -f "$C" ]]; then
  grep -q '^@AGENTS\.md[[:space:]]*$' "$C" && ok "CLAUDE.md" "imports AGENTS.md" \
                                           || no "CLAUDE.md" "no @AGENTS.md import line"
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
    && no "skill budget" "unset — 1% default may drop skill descriptions" \
    || ok "skill budget" "$b"
else
  no "settings.json parses" "invalid JSON"
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
printf 'PASS %d   FAIL %d   SKIP %d\n' "$pass" "$fail" "$skip"
[[ "$skip" -gt 0 ]] && echo 'SKIP is not PASS. Those checks did not run.'
[[ "$fail" -gt 0 ]] && exit 1
exit 0
