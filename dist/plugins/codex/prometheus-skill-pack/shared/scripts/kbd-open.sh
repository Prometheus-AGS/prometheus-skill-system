#!/usr/bin/env bash
# kbd-open — session-start context primer, run by the sessionstart-kbd-open hook.
#
# Prints a markdown snapshot that the harness adds to the session:
#   1. the active KBD phase, its position reminder and its goals
#   2. bounded local knowledge for that phase (`pk context`, project + shared +
#      global scopes, no LLM call)
#   3. items waiting on a human decision: promotion candidates, new-skill
#      candidates, skill-update candidates, and knowledge gaps seen repeatedly
#   4. today's learning log, the latest daily pulse, and FSRS cards due
#
# Every section is silent when its source is absent, so the hook stays quiet on
# a machine with none of the optional pieces installed. The snapshot also goes to
# ~/.prometheus/last-open-snapshot.txt. Bash 3.2 compatible; always exits 0.
#
# Output must not begin with `{`: Codex parses hook stdout that opens with `{`
# as a structured response and fails the hook.

set -uo pipefail

PROMETHEUS_HOME="${HOME}/.prometheus"
LOG_FILE="${PROMETHEUS_HOME}/logs/kbd-open.log"
SNAPSHOT="${PROMETHEUS_HOME}/last-open-snapshot.txt"
LEARNING_LOG="${PROMETHEUS_HOME}/learning-log"
SKILL_UPDATES_DIR="${PROMETHEUS_HOME}/skill-updates"
PROMOTION_DIR="${PROMETHEUS_HOME}/promotion-candidates/pending"
SKILL_CANDIDATES_DIR="${PROMETHEUS_HOME}/skill-candidates/pending"
GAPS_FILE="${PROMETHEUS_HOME}/knowledge-gaps/gaps.jsonl"
PULSE_DIR="${PROMETHEUS_HOME}/pulse"
PK_CONTEXT_MAX_BYTES=3000
CANDIDATE_LIMIT=5

log() {
  mkdir -p "$(dirname "$LOG_FILE")" 2>/dev/null || return 0
  printf '[%s] [kbd-open] %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$*" >>"$LOG_FILE" 2>/dev/null || true
}

if [ -x "${PROMETHEUS_HOME}/bin/pk" ]; then
  PATH="${PROMETHEUS_HOME}/bin:${PATH}"
fi

have_python() { command -v python3 >/dev/null 2>&1; }

# json_field <file> <key>... — print the first non-empty top-level string field.
json_field() {
  have_python || return 0
  python3 - "$@" <<'PY' 2>/dev/null
import json, sys
try:
    with open(sys.argv[1]) as handle:
        data = json.load(handle)
except Exception:
    sys.exit(0)
for key in sys.argv[2:]:
    value = data.get(key) if isinstance(data, dict) else None
    if isinstance(value, str) and value.strip():
        print(value.strip())
        break
PY
}

find_kbd_root() {
  dir="$PWD"
  while [ "$dir" != "/" ]; do
    if [ -d "$dir/.kbd-orchestrator" ]; then
      printf '%s' "$dir"
      return 0
    fi
    dir="$(dirname "$dir")"
  done
  return 1
}

# list_candidates <dir> — one line per pending JSON candidate, newest first.
list_candidates() {
  have_python || return 0
  python3 - "$1" "$CANDIDATE_LIMIT" <<'PY' 2>/dev/null
import json, os, sys
directory, limit = sys.argv[1], int(sys.argv[2])
try:
    names = [n for n in os.listdir(directory) if n.endswith('.json')]
except OSError:
    sys.exit(0)
names.sort(key=lambda n: os.path.getmtime(os.path.join(directory, n)), reverse=True)
print(f"COUNT {len(names)}")
for name in names[:limit]:
    try:
        with open(os.path.join(directory, name)) as handle:
            data = json.load(handle)
    except Exception:
        continue
    text = next((str(data[k]).strip() for k in ('lesson', 'title', 'summary', 'name', 'description')
                 if isinstance(data, dict) and data.get(k)), '(no summary)')
    evidence = data.get('evidence') if isinstance(data, dict) else None
    count = len(evidence) if isinstance(evidence, list) else 0
    scope = data.get('proposedScope') or data.get('mode') or ''
    extra = ', '.join(part for part in (f"{count} evidence" if count else '', scope) if part)
    print(f"- `{name[:-5][:12]}` {text[:140]}" + (f" ({extra})" if extra else ''))
PY
}

# repeated_gaps — still-open knowledge-gap topics seen in at least two sessions,
# read through the gap library so the gaps.jsonl record format has one parser.
repeated_gaps() {
  have_python || return 0
  gap_lib="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd)/lib/prompt_gap.py"
  [ -f "$gap_lib" ] || return 0
  python3 "$gap_lib" list --min-seen 2 --limit 5 2>/dev/null
}

KBD_ROOT="$(find_kbd_root 2>/dev/null || true)"
KBD_PHASE=""
KBD_PROJECT=""
KBD_GOALS=""
if [ -n "$KBD_ROOT" ]; then
  KBD_PHASE="$(json_field "$KBD_ROOT/.kbd-orchestrator/current-waypoint.json" active_phase phase)"
  KBD_PROJECT="$(json_field "$KBD_ROOT/.kbd-orchestrator/project.json" name project)"
  if [ -n "$KBD_PHASE" ] && [ -f "$KBD_ROOT/.kbd-orchestrator/phases/$KBD_PHASE/goals.md" ]; then
    KBD_GOALS="$(head -50 "$KBD_ROOT/.kbd-orchestrator/phases/$KBD_PHASE/goals.md" 2>/dev/null || true)"
  fi
fi

{
  printf '# kbd-open session snapshot\n'
  printf '_generated: %s_\n\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"

  if [ -n "$KBD_PHASE" ]; then
    printf '## Active KBD phase\n'
    printf -- '- **project**: %s\n' "${KBD_PROJECT:-unspecified}"
    printf -- '- **phase**: %s\n' "$KBD_PHASE"
    printf -- '- **kbd_root**: %s\n\n' "$KBD_ROOT"
  fi

  REMINDER=""
  if [ -n "$KBD_ROOT" ] && [ -f "$KBD_ROOT/.kbd-orchestrator/position-reminder.txt" ]; then
    REMINDER="$KBD_ROOT/.kbd-orchestrator/position-reminder.txt"
  elif [ -f "${HOME}/.claude/hooks/position-reminder.txt" ]; then
    REMINDER="${HOME}/.claude/hooks/position-reminder.txt"
  fi
  if [ -n "$REMINDER" ]; then
    printf '## Position reminder\n```\n%s\n```\n\n' "$(head -30 "$REMINDER")"
  fi

  if [ -n "$KBD_GOALS" ]; then
    printf '## Phase goals\n%s\n\n' "$KBD_GOALS"
  fi

  # Local knowledge only: `pk context` reads committed snapshots and never calls
  # an LLM, unlike the `pk focus` this replaced, which needed a 30s budget.
  if command -v pk >/dev/null 2>&1; then
    QUERY="${KBD_PHASE:-${KBD_PROJECT:-$(basename "$PWD")}}"
    if [ -n "$KBD_GOALS" ]; then
      QUERY="$QUERY $(printf '%s' "$KBD_GOALS" | grep -v '^#' | head -3 | tr '\n' ' ' | cut -c1-200)"
    fi
    CONTEXT="$(pk context "$QUERY" --scope project --scope shared --scope global \
      --format hook --max-bytes "$PK_CONTEXT_MAX_BYTES" 2>/dev/null || true)"
    if [ -n "$CONTEXT" ]; then
      printf '## Knowledge for this work\n%s\n\n' "$CONTEXT"
    fi
  fi

  if [ -d "$PROMOTION_DIR" ]; then
    LISTING="$(list_candidates "$PROMOTION_DIR")"
    COUNT="$(printf '%s\n' "$LISTING" | sed -n 's/^COUNT //p')"
    if [ -n "$COUNT" ] && [ "$COUNT" -gt 0 ]; then
      printf '## Promotion candidates awaiting review (%s)\n' "$COUNT"
      printf '%s\n' "$LISTING" | grep -v '^COUNT '
      printf '\n_Accept or reject only on a human decision: `pk candidates accept|reject <id>`_\n\n'
    fi
  fi

  if [ -d "$SKILL_CANDIDATES_DIR" ]; then
    LISTING="$(list_candidates "$SKILL_CANDIDATES_DIR")"
    COUNT="$(printf '%s\n' "$LISTING" | sed -n 's/^COUNT //p')"
    if [ -n "$COUNT" ] && [ "$COUNT" -gt 0 ]; then
      printf '## New-skill candidates (%s)\n' "$COUNT"
      printf '%s\n' "$LISTING" | grep -v '^COUNT '
      printf '\n_Review with `pk candidates list --kind skill`; create with `/pmpo-skill-creator`_\n\n'
    fi
  fi

  if [ -d "$SKILL_UPDATES_DIR" ] && [ -n "$(ls -A "$SKILL_UPDATES_DIR" 2>/dev/null)" ]; then
    COUNT="$(ls -1 "$SKILL_UPDATES_DIR" 2>/dev/null | wc -l | tr -d ' ')"
    printf '## Pending skill-update candidates (%s)\n' "$COUNT"
    ls -1t "$SKILL_UPDATES_DIR" 2>/dev/null | head -"$CANDIDATE_LIMIT" | sed 's/^/- /'
    printf '\n_Review with `/pmpo-skill-creator --update <name>`_\n\n'
  fi

  if [ -f "$GAPS_FILE" ]; then
    GAPS="$(repeated_gaps)"
    if [ -n "$GAPS" ]; then
      printf '## Knowledge gaps seen repeatedly\n%s\n\n' "$GAPS"
    fi
  fi

  TODAY_LOG="${LEARNING_LOG}/$(date -u '+%Y-%m-%d').jsonl"
  if [ -f "$TODAY_LOG" ]; then
    printf "## Today's learning log (%s entries)\n" "$(wc -l <"$TODAY_LOG" | tr -d ' ')"
    if have_python; then
      tail -3 "$TODAY_LOG" | python3 -c '
import json, sys
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        entry = json.loads(line)
    except Exception:
        print(f"- (unparsed) {line[:100]}")
        continue
    print(f"- {entry.get(\"ts\", \"\")} · {entry.get(\"kbd_phase\", \"-\")} · {entry.get(\"source\", \"\")}")
' 2>/dev/null
    fi
    printf '\n'
  fi

  if [ -d "$PULSE_DIR" ]; then
    LATEST_PULSE="$(ls -1 "$PULSE_DIR"/*.md 2>/dev/null | grep -v -- '-context\.md$' | sort -r | head -1 || true)"
    if [ -n "$LATEST_PULSE" ] && [ -f "$LATEST_PULSE" ]; then
      printf '## Latest daily pulse — %s\n```\n%s\n```\n_Full file: %s_\n\n' \
        "$(basename "$LATEST_PULSE" .md)" "$(head -40 "$LATEST_PULSE")" "$LATEST_PULSE"
    fi
  fi

  if { [ -n "${LEARNER_MODEL_URL:-}" ] || [ -S /tmp/learner-model.sock ]; } && command -v curl >/dev/null 2>&1; then
    DUE="$(curl -sf --max-time 3 "${LEARNER_MODEL_URL:-http://localhost:7740}/api/v1/due" 2>/dev/null || true)"
    if [ -n "$DUE" ] && [ "$DUE" != "[]" ]; then
      printf '## FSRS cards due (review with `/learn-retain`)\n```json\n%s\n```\n\n' "$DUE"
    fi
  fi

  printf -- '---\n_end of snapshot_\n'
} | {
  if mkdir -p "$PROMETHEUS_HOME" 2>/dev/null; then tee "$SNAPSHOT" 2>/dev/null || cat; else cat; fi
}

log "snapshot written${KBD_PHASE:+ for phase $KBD_PHASE}"
exit 0
