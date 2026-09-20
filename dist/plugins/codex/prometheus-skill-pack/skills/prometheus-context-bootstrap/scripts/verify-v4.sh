#!/usr/bin/env bash
# Assert a Prometheus Rules Architecture v4 project is intact and enforcing.
# Reached from verify.sh when CLAUDE.md carries the `prometheus-rules: v4` header.
#
# PASS / FAIL / WARN / SKIP with the same meaning as verify.sh: SKIP is never PASS.
set -uo pipefail

project_path="${1:?usage: verify-v4.sh <project root>}"
SKILL_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

pass=0; fail=0; skip=0; warn=0
ok() { printf 'PASS  %-34s %s\n' "$1" "${2:-}"; pass=$((pass+1)); }
no() { printf 'FAIL  %-34s %s\n' "$1" "${2:-}"; fail=$((fail+1)); }
sk() { printf 'SKIP  %-34s %s\n' "$1" "${2:-}"; skip=$((skip+1)); }
wr() { printf 'WARN  %-34s %s\n' "$1" "${2:-}"; warn=$((warn+1)); }

echo "Verifying $project_path (layout v4)"
echo

C="$project_path/CLAUDE.md"

# --- single source and generator: drift, Layer 0 budget, A-1..A-17, paths: on every rule ---
if [[ ! -x "$project_path/rules/build.sh" || ! -d "$project_path/rules/src" ]]; then
  no "rules source" "rules/build.sh or rules/src/ missing — run bootstrap.sh --layout v4"
elif ! command -v python3 >/dev/null 2>&1; then
  sk "rules build --check" "python3 absent — NOT verified"
else
  out="$(bash "$project_path/rules/build.sh" --check 2>&1)" \
    && ok "rules build --check" "${out#rules/build: ✓ }" \
    || no "rules build --check" "$(printf '%s' "$out" | head -3 | tr '\n' ' ')"
fi

# --- Layer 0 is the real file; the other names link to it ---
if [[ -f "$C" && ! -L "$C" ]]; then ok "CLAUDE.md" "real file"; else no "CLAUDE.md" "must be a regular file in layout v4"; fi
for name in AGENTS.md GEMINI.md; do
  f="$project_path/$name"
  if [[ -L "$f" && "$(readlink "$f")" == "CLAUDE.md" && -f "$f" ]]; then ok "$name" "-> CLAUDE.md"
  else no "$name" "must be a symlink to CLAUDE.md — a copy diverges"; fi
done
if grep -q 'REPLACE THIS LINE' "$C" 2>/dev/null; then
  wr "project block" "§P still has the placeholder line — edit rules/src/constitution.md"
else
  ok "project block" "edited"
fi

# --- nested AGENTS.md must never carry the constitution (duplicate-constitution hazard) ---
dupes="$(find "$project_path" -mindepth 2 -maxdepth 4 -name AGENTS.md -not -path '*/node_modules/*' -not -path '*/.git/*' \
  -exec grep -lE '^\s*-?\s*\*\*A-[0-9]+ ·' {} + 2>/dev/null | head -5)"
[[ -z "$dupes" ]] && ok "no nested constitution" "" \
                  || no "no nested constitution" "rule IDs repeated in: $(printf '%s' "$dupes" | tr '\n' ' ')"

# --- Layer 4: hooks present, executable and wired ---
H="$project_path/.claude/hooks"
if [[ -d "$H" ]]; then
  bad=""; for f in "$H"/*.sh; do [[ -x "$f" ]] || bad="$bad $(basename "$f")"; done
  [[ -z "$bad" ]] && ok "hooks executable" "$(ls "$H"/*.sh 2>/dev/null | wc -l | tr -d ' ') hook(s)" || no "hooks executable" "not executable:$bad"
else
  sk "hooks executable" ".claude/hooks absent"
fi
S="$project_path/.claude/settings.json"
if [[ ! -f "$S" ]]; then
  no "settings.json" "absent"
elif ! command -v jq >/dev/null 2>&1; then
  sk "settings.json parses" "jq absent — NOT verified"
elif jq -e . "$S" >/dev/null 2>&1; then
  ok "settings.json parses" ""
  for pair in "PreToolUse:tier-guard.sh" "PreToolUse:build-guard.sh" "PostToolUse:file-lines-guard.sh"; do
    ev="${pair%%:*}"; sc="${pair##*:}"
    if [[ -f "$H/$sc" ]]; then
      jq -e --arg e "$ev" --arg s "$sc" '[.hooks[$e][]?.hooks[]?.command // ""] | any(test($s))' "$S" >/dev/null 2>&1 \
        && ok "$sc wired" "" || no "$sc wired" "installed but not referenced in settings.json — it enforces nothing"
    fi
  done
  blanket="$(jq -r '(.permissions.deny // [])[] | select(test("^(Edit|Write|MultiEdit|NotebookEdit)\\((\\./)?\\.kbd-orchestrator(/\\*\\*?)?/?\\)$"))' "$S" 2>/dev/null)"
  [[ -z "$blanket" ]] && ok "KBD artifacts writable" "" || no "KBD artifacts writable" "deny rule covers all of .kbd-orchestrator — remove it"
else
  no "settings.json parses" "invalid JSON"
fi

# --- A-15 commit hook: installed and actually active for this clone ---
if [[ -x "$project_path/.githooks/commit-msg" ]]; then
  if [[ -d "$project_path/.git" || -f "$project_path/.git" ]]; then
    hp="$(git -C "$project_path" config --get core.hooksPath 2>/dev/null || true)"
    [[ "$hp" == ".githooks" ]] && ok "commit-msg hook active" "" \
      || wr "commit-msg hook active" "run: git config core.hooksPath .githooks  (per-clone setting; A-15 is not enforced until then)"
  else
    sk "commit-msg hook active" "not a git repository"
  fi
else
  no "commit-msg hook" ".githooks/commit-msg missing or not executable"
fi

# --- the structure rules, as checks ---
for s in check-file-lines check-architecture; do
  f="$project_path/scripts/$s.sh"
  if [[ ! -x "$f" ]]; then no "$s" "scripts/$s.sh missing or not executable"; continue; fi
  out="$("$f" 2>&1)"; rc=$?
  case "$rc" in
    0) ok "$s" "" ;;
    1) no "$s" "$(printf '%s' "$out" | grep -m3 -vE '^\s*$' | tr '\n' ' ' | cut -c1-160)" ;;
    *) sk "$s" "could not run (exit $rc) — NOT verified" ;;
  esac
done

# --- Layer 3 memory and position ---
missing=""; for e in session-log.md decisions.md gotchas.md postmortems knowledge; do [[ -e "$project_path/.prometheus/$e" ]] || missing="$missing $e"; done
[[ -z "$missing" ]] && ok ".prometheus layout" "complete" || no ".prometheus layout" "missing:$missing"
W="$project_path/.kbd-orchestrator/current-waypoint.json"
[[ -f "$W" ]] && ok "waypoint" "present" || wr "waypoint" "absent — run /kbd-init and /kbd-new-phase; tier-guard assumes implementation until then"

# --- skill budget: measured, machine-wide, so WARN not FAIL ---
if [[ -x "$SKILL_ROOT/scripts/skill-budget.sh" ]] && command -v python3 >/dev/null 2>&1; then
  b="$("$SKILL_ROOT/scripts/skill-budget.sh" --path "$project_path" 2>/dev/null | tail -1)"; rc=$?
  [[ $rc -eq 0 ]] && ok "skill budget measured" "$b" || wr "skill budget measured" "${b:-over budget} — route by name from CLAUDE.md §F"
else
  sk "skill budget measured" "python3 or skill-budget.sh absent — NOT verified"
fi

echo
printf 'PASS %d   FAIL %d   WARN %d   SKIP %d\n' "$pass" "$fail" "$warn" "$skip"
[[ "$skip" -gt 0 ]] && echo 'SKIP is not PASS. Those checks did not run.'
[[ "$warn" -gt 0 ]] && echo 'WARN is a real finding this repo cannot fix on its own, or a one-line setup step. Do not ignore it.'
[[ "$fail" -gt 0 ]] && exit 1
exit 0
