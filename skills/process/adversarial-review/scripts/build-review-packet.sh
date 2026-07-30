#!/usr/bin/env bash
# build-review-packet.sh — deterministic assembly of the review packet the
# judge receives. The packet is the ONLY context the judge ever sees:
# it contains the work product and its success criteria — never the
# producing session's chat history. Isolation is structural.
#
# Usage:
#   build-review-packet.sh --mode diff     --phase <phase> --target <change-id> [--out <path>]
#   build-review-packet.sh --mode artifact --phase <phase> --target assess|analyze|plan [--out <path>]
#
# Emits packet JSON to --out (default: stdout).
# Exit codes: 0 ok · 1 usage · 2 missing inputs
#
# bash 3.2 compatible (no mapfile, no declare -A). No LLM calls (class=small).
set -uo pipefail

MODE="" PHASE="" TARGET="" OUT=""
while [ $# -gt 0 ]; do
  case "$1" in
    --mode)   MODE="${2:-}"; shift 2 ;;
    --phase)  PHASE="${2:-}"; shift 2 ;;
    --target) TARGET="${2:-}"; shift 2 ;;
    --out)    OUT="${2:-}"; shift 2 ;;
    *) echo "usage: $0 --mode diff|artifact --phase <phase> --target <id|stage> [--out <path>]" >&2; exit 1 ;;
  esac
done
case "$MODE" in diff|artifact) ;; *) echo "[packet] ERROR: --mode must be diff or artifact" >&2; exit 1 ;; esac
[ -n "$PHASE" ] && [ -n "$TARGET" ] || { echo "[packet] ERROR: --phase and --target are required" >&2; exit 1; }

find_kbd_root() {
  local d="$PWD"
  while [ "$d" != "/" ]; do
    [ -d "$d/.kbd-orchestrator" ] && { printf '%s' "$d/.kbd-orchestrator"; return 0; }
    d="$(dirname "$d")"
  done
  return 1
}
KBD_ROOT="$(find_kbd_root 2>/dev/null || true)"
[ -n "$KBD_ROOT" ] || { echo "[packet] ERROR: .kbd-orchestrator not found above $PWD" >&2; exit 2; }
PHASE_DIR="$KBD_ROOT/phases/$PHASE"
[ -d "$PHASE_DIR" ] || { echo "[packet] ERROR: phase dir not found: $PHASE_DIR" >&2; exit 2; }

echo "[MODEL_ROUTING] phase=adv-review-packet class=small" >&2

command -v python3 >/dev/null 2>&1 || { echo "[packet] ERROR: python3 required" >&2; exit 2; }

WORK="$(mktemp -d "${TMPDIR:-/tmp}/adv-review-packet.XXXXXX")"
# shellcheck disable=SC2064
trap "rm -rf '$WORK'" EXIT

# --- producer model -----------------------------------------------------------
# Best-effort record of which model produced the work under review, for the
# judge!=producer collision check. Sources, in order: progress.json,
# KBD_PRODUCER_MODEL, ANTHROPIC_MODEL, "unknown".
PRODUCER="$(python3 - "$PHASE_DIR/progress.json" <<'PY' 2>/dev/null || true
import json, sys
try:
    d = json.load(open(sys.argv[1]))
    print(d.get("producer_model") or "")
except Exception:
    pass
PY
)"
[ -n "$PRODUCER" ] || PRODUCER="${KBD_PRODUCER_MODEL:-${ANTHROPIC_MODEL:-}}"
# Harness-provided identifiers, before giving up. CLAUDE_MODEL / CLAUDECODE_MODEL
# are set by some Claude Code builds; CLAUDE_CODE_MODEL by others.
[ -n "$PRODUCER" ] || PRODUCER="${CLAUDE_MODEL:-${CLAUDE_CODE_MODEL:-${CLAUDECODE_MODEL:-}}}"

# "unknown" is not a harmless default: the judge's collision check compares
# candidate != producer, so an unknown producer makes it pass TRIVIALLY. Every one
# of the 8 historical reviews carried producer_model="unknown", which is why
# judge!=producer was never actually enforced despite the check being present.
# Warn loudly so a trivially-passing check is visible at packet-build time.
if [ -z "$PRODUCER" ]; then
  PRODUCER="unknown"
  echo "[packet] WARN: PRODUCER_UNKNOWN — cannot determine which model produced this" >&2
  echo "[packet]       work, so the judge!=producer guarantee cannot be enforced." >&2
  echo "[packet]       Set KBD_PRODUCER_MODEL (e.g. export KBD_PRODUCER_MODEL=claude-opus-5)" >&2
  echo "[packet]       to restore the cross-model check." >&2
fi

# --- shared context -----------------------------------------------------------
CONSTRAINTS_FILE="$KBD_ROOT/constraints.md"
[ -f "$CONSTRAINTS_FILE" ] && cp "$CONSTRAINTS_FILE" "$WORK/constraints.md"

# File tree: top 2 levels, pruning bulk dirs. Deterministic (sorted).
REPO_ROOT="$(dirname "$KBD_ROOT")"
( cd "$REPO_ROOT" && find . -maxdepth 2 \
    -not -path '*/node_modules*' -not -path '*/.git*' -not -path '*/target*' \
    -not -path '*/dist*' -not -path '*/.refiner*' 2>/dev/null | sort ) > "$WORK/file_tree.txt" || true

# --- mode-specific content ----------------------------------------------------
if [ "$MODE" = "diff" ]; then
  CHANGE_DIR=""
  for cand in "$KBD_ROOT/changes/$TARGET" "$REPO_ROOT/openspec/changes/$TARGET"; do
    [ -d "$cand" ] && { CHANGE_DIR="$cand"; break; }
  done

  # Acceptance criteria: tasks.md / spec.md / verification.md from the change dir.
  if [ -n "$CHANGE_DIR" ]; then
    for f in tasks.md spec.md proposal.md verification.md; do
      [ -f "$CHANGE_DIR/$f" ] && cat "$CHANGE_DIR/$f" >> "$WORK/acceptance_criteria.md"
    done
  fi
  [ -s "$WORK/acceptance_criteria.md" ] || \
    echo "[packet] WARN: no acceptance criteria found for change $TARGET" >&2

  # Diff: scope to the change's file list when recorded, else uncommitted work.
  FILES_LIST="$WORK/changed_files.txt"
  if [ -n "$CHANGE_DIR" ] && [ -f "$CHANGE_DIR/files.txt" ]; then
    cp "$CHANGE_DIR/files.txt" "$FILES_LIST"
  fi
  if [ -s "$FILES_LIST" ]; then
    ( cd "$REPO_ROOT" && git diff HEAD -- $(cat "$FILES_LIST") 2>/dev/null ) > "$WORK/diff.patch" || true
  else
    ( cd "$REPO_ROOT" && git diff HEAD 2>/dev/null ) > "$WORK/diff.patch" || true
  fi
  [ -s "$WORK/diff.patch" ] || \
    ( cd "$REPO_ROOT" && git show --patch HEAD 2>/dev/null ) > "$WORK/diff.patch" || true
  [ -s "$WORK/diff.patch" ] || { echo "[packet] ERROR: no diff content resolvable for $TARGET" >&2; exit 2; }
else
  # artifact mode: TARGET selects the stage artifact set.
  case "$TARGET" in
    assess)  ARTS="assessment.md" ;;
    analyze) ARTS="analysis.md library-candidates.json" ;;
    plan)    ARTS="plan.md" ;;
    *) echo "[packet] ERROR: artifact --target must be assess|analyze|plan" >&2; exit 1 ;;
  esac
  FOUND=0
  for a in $ARTS; do
    if [ -f "$PHASE_DIR/$a" ]; then
      { echo "===== $a ====="; cat "$PHASE_DIR/$a"; echo; } >> "$WORK/artifact.md"
      FOUND=1
    fi
  done
  [ "$FOUND" -eq 1 ] || { echo "[packet] ERROR: no artifacts found for stage $TARGET in $PHASE_DIR" >&2; exit 2; }

  [ -f "$PHASE_DIR/goals.md" ] && cp "$PHASE_DIR/goals.md" "$WORK/goals.md"

  # Prior-stage handoff summaries (JSON files under phases/<phase>/handoffs/).
  if [ -d "$PHASE_DIR/handoffs" ]; then
    for h in "$PHASE_DIR"/handoffs/*.json; do
      [ -f "$h" ] || continue
      { echo "===== $(basename "$h") ====="; cat "$h"; echo; } >> "$WORK/handoffs.md"
    done
  fi
fi

# --- assemble packet ----------------------------------------------------------
PACKET="$(MODE="$MODE" PHASE="$PHASE" TARGET="$TARGET" PRODUCER="$PRODUCER" WORK="$WORK" \
python3 <<'PY'
import json, os

work = os.environ["WORK"]

def slurp(name):
    p = os.path.join(work, name)
    if os.path.exists(p):
        return open(p, encoding="utf-8", errors="replace").read()
    return None

packet = {
    "packet_version": 1,
    "mode": os.environ["MODE"],
    "phase": os.environ["PHASE"],
    "target": os.environ["TARGET"],
    "producer_model": os.environ["PRODUCER"],
    "constraints": slurp("constraints.md"),
    "file_tree": slurp("file_tree.txt"),
}
if packet["mode"] == "diff":
    packet["diff"] = slurp("diff.patch")
    packet["acceptance_criteria"] = slurp("acceptance_criteria.md")
else:
    packet["artifact"] = slurp("artifact.md")
    packet["goals"] = slurp("goals.md")
    packet["prior_handoffs"] = slurp("handoffs.md")
print(json.dumps(packet, indent=2))
PY
)"

if [ -n "$OUT" ]; then
  mkdir -p "$(dirname "$OUT")"
  printf '%s\n' "$PACKET" > "$OUT"
  echo "[packet] wrote $OUT" >&2
else
  printf '%s\n' "$PACKET"
fi
