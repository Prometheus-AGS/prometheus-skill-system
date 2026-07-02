---
id: change-slli-010-pmpo-skill-creator-update-mode
title: pmpo-skill-creator --update mode
phase: self-learning-loop-integration
gaps: [SKILL-UPDATE-1]
priority: 10 of 10
agent: claude-code
depends_on: [change-slli-004]
status: done
scope:
  - skills/process/pmpo-skill-creator/SKILL.md
  - shared/scripts/propose-skill-update.sh
---

# change-slli-010-pmpo-skill-creator-update-mode — pmpo-skill-creator --update mode

## Summary

Add `--update` mode to `pmpo-skill-creator` so existing skills can be improved in-place from accumulated learning patterns. This closes the Hermes-parity "in-place skill refinement" gap. Requires explicit user approval before any skill file is modified.

## Files Modified

### `skills/process/pmpo-skill-creator/SKILL.md`

Add command variant:

```
/pmpo-skill-creator --update <skill-name>
```

**Flow:**
1. Read `~/.claude/skills/<skill-name>/SKILL.md` (current skill content)
2. Search `~/.prometheus/learning-log/*.jsonl` for entries matching the skill's domain
3. Search surreal-memory for patterns related to `<skill-name>`
4. Identify targeted additions: new examples, corrected instructions, updated references
5. Generate a unified diff (`unified_diff`) of proposed changes
6. Write proposed diff to `~/.prometheus/skill-updates/<skill-name>-<date>.diff`
7. Present diff to user — do NOT apply automatically
8. Call `/pmpo-elicit` if available; otherwise prompt inline: "Apply this update? (y/N)"
9. Only on explicit `y`: apply diff and re-validate skill with `npm run validate:strict`

## Files Created

### `shared/scripts/propose-skill-update.sh`

Called by `evaluate-session.sh` (change-slli-004) when learning patterns match an existing skill name:

```bash
#!/usr/bin/env bash
# Proposes a skill update based on learning log patterns
# Does NOT apply — writes diff for human review

SKILL_NAME="$1"
SKILL_PATH="${HOME}/.claude/skills/${SKILL_NAME}/SKILL.md"
[[ -f "$SKILL_PATH" ]] || exit 0

UPDATES_DIR="${HOME}/.prometheus/skill-updates"
mkdir -p "$UPDATES_DIR"

DIFF_FILE="$UPDATES_DIR/${SKILL_NAME}-$(date +%Y-%m-%d).diff"

# Only create if learning log has entries for this skill
LEARNING_HITS=$(grep -l "$SKILL_NAME" "${HOME}/.prometheus/learning-log/"*.jsonl 2>/dev/null | wc -l)
[[ "$LEARNING_HITS" -eq 0 ]] && exit 0

echo "Skill update candidate: $SKILL_NAME (${LEARNING_HITS} learning entries)" \
  >> "${HOME}/.prometheus/skill-updates/pending.log"

# Diff will be written by pmpo-skill-creator --update invocation
echo "Run: /pmpo-skill-creator --update $SKILL_NAME to review and apply" \
  >> "$DIFF_FILE"
```

## Acceptance Criteria

- `/pmpo-skill-creator --update kbd-plan` produces a diff file at `~/.prometheus/skill-updates/`
- Diff is NOT applied without explicit `y` response from user
- After `y`: updated skill passes `npm run validate:strict`
- After `n` or no response: skill file is unchanged
- `propose-skill-update.sh` exits 0 when no learning entries match the skill

## Tasks

- [x] 1. `/pmpo-skill-creator --update kbd-plan` produces a diff file at `~/.prometheus/skill-updates/`
- [x] 2. Diff is NOT applied without explicit `y` response from user
- [x] 3. After `y`: updated skill passes `npm run validate:strict`
- [x] 4. After `n` or no response: skill file is unchanged
- [x] 5. `propose-skill-update.sh` exits 0 when no learning entries match the skill
