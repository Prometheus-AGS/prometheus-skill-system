#!/usr/bin/env bash
# Detect drift between the installed skill generation and the skill-system repo.
#
# WHY THIS EXISTS
# ---------------
# On 2026-08-12 an agent authored an entire KBD phase without ever moving
# canonical state, because the installed copy of `kbd-assess/SKILL.md` predated
# the repo by 25 days and did not contain the step that says:
#
#     "Enter/complete the assessment stage with a typed `prometheus kbd stage`
#      command; never edit progress.json"
#
# The instruction existed in git and had never reached the machine. Nothing
# compared the two, so the drift was invisible until a second harness read a
# 4-day-stale position file and stalled.
#
# This check makes that class of failure loud at session start, where it costs
# one command to fix, instead of silent until it corrupts a phase.
#
# EXIT CODES
#   0  fresh, or cannot determine (never block a session on an unknown)
#   1  stale — installed generation is behind the repo
#   2  duplicate skill names on disk (shadowing; ambiguous resolution)
#
# Emits human-readable findings on stdout. `--json` emits a machine-readable
# object for `prometheus doctor` to consume.

set -uo pipefail

PLUGIN_ROOT="${PROMETHEUS_PLUGIN_ROOT:-$HOME/.prometheus/plugins/prometheus-skill-pack}"
CURRENT="$PLUGIN_ROOT/current"
STAMP="$CURRENT/.source-commit"
CLAUDE_SKILLS="${CLAUDE_SKILLS_DIR:-$HOME/.claude/skills}"
JSON=0
[ "${1:-}" = "--json" ] && JSON=1

status="fresh"
detail=""
installed_sha=""
repo_sha=""
dupes=""

# ---------------------------------------------------------------- installed
if [ ! -d "$CURRENT" ]; then
  status="unknown"
  detail="no installed generation at $CURRENT"
else
  [ -f "$STAMP" ] && installed_sha="$(tr -d '[:space:]' < "$STAMP" 2>/dev/null)"
fi

# --------------------------------------------------------------------- repo
# The repo path is not guessed: it is recorded at install time, or supplied.
REPO="${PROMETHEUS_SKILL_SYSTEM_REPO:-}"
if [ -z "$REPO" ] && [ -f "$CURRENT/.source-repo" ]; then
  REPO="$(tr -d '[:space:]' < "$CURRENT/.source-repo" 2>/dev/null)"
fi

if [ -n "$REPO" ] && [ -d "$REPO/.git" ]; then
  repo_sha="$(git -C "$REPO" rev-parse HEAD 2>/dev/null || true)"
fi

# ------------------------------------------------------------------ compare
if [ "$status" = "fresh" ]; then
  if [ -z "$installed_sha" ]; then
    status="unknown"
    detail="installed generation carries no .source-commit stamp; reinstall to add one"
  elif [ -z "$repo_sha" ]; then
    status="unknown"
    detail="skill-system repo not locatable (set PROMETHEUS_SKILL_SYSTEM_REPO)"
  elif [ "$installed_sha" != "$repo_sha" ]; then
    status="stale"
    detail="installed ${installed_sha:0:8} != repo ${repo_sha:0:8}"
  fi
fi

# --------------------------------------------------- duplicate skill names
# A second copy of a skill on disk is worse than a stale one: which loads
# depends on scan order, so the same session can behave two ways. This is
# exactly what a July-dated `~/.claude/skills/prometheus/` tree did while a
# correct flat generation sat beside it.
if [ -d "$CLAUDE_SKILLS" ]; then
  dupes="$(
    find "$CLAUDE_SKILLS" -name SKILL.md -maxdepth 6 2>/dev/null \
      | xargs grep -h -m1 '^name:' 2>/dev/null \
      | sed 's/^name:[[:space:]]*//' | tr -d '"' \
      | sort | uniq -d | head -20
  )"
fi

# ------------------------------------------------------------------ report
if [ "$JSON" -eq 1 ]; then
  printf '{"check":"skills-freshness","status":%s,"installed":%s,"repo":%s,"detail":%s,"duplicates":%s}\n' \
    "\"$status\"" "\"$installed_sha\"" "\"$repo_sha\"" "\"$detail\"" \
    "$(printf '%s' "$dupes" | tr '\n' ',' | sed 's/,$//' | awk '{printf "\"%s\"", $0}')"
else
  case "$status" in
    stale)
      echo "[skills-freshness] STALE — $detail"
      echo "  fix: (cd ${REPO:-<skill-system-repo>} && git pull --ff-only && node scripts/install.js --scope user)"
      ;;
    unknown) echo "[skills-freshness] indeterminate — $detail" ;;
    fresh)   echo "[skills-freshness] ok — installed matches repo (${installed_sha:0:8})" ;;
  esac
  if [ -n "$dupes" ]; then
    echo "[skills-freshness] DUPLICATE skill names on disk — resolution is scan-order dependent:"
    printf '  %s\n' $dupes
  fi
fi

[ -n "$dupes" ] && exit 2
[ "$status" = "stale" ] && exit 1
exit 0
