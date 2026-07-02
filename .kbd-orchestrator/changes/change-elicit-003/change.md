---
id: change-elicit-003
title: Install pmpo-elicit to all platforms + SKILL.md platform-mode section
phase: pmpo-elicit
gaps: [G-05]
goals: [G5]
priority: HIGH
effort: S
agent: claude-code
status: done
scope:
  - scripts/install-skills-flat.sh
  - skills/process/pmpo-elicit/SKILL.md
---

# change-elicit-003 — Platform install + SKILL.md platform section

## Context

`pmpo-elicit` is registered in `.claude-plugin/plugin.json` (Claude Code via symlink) but
is absent from `install-skills-flat.sh`'s skill copy list. All five non-Claude-Code
platforms (Kimi, Codex, OpenCode, Cursor, Zed) never receive the skill directory,
making the file-based async contract impossible to use.

## Scope

### `scripts/install-skills-flat.sh` (MODIFY)

Find the block where `kbd-goal` and `kbd-goal-check` are added to the process skill
copy list (the list used by `copy_skill_to_platform` or equivalent). Add `pmpo-elicit`
to the same list:

```bash
PROCESS_SKILLS=(
  "kbd-goal"
  "kbd-goal-check"
  "pmpo-elicit"    # ← ADD THIS
  # ... other process skills
)
```

If no explicit PROCESS_SKILLS array exists, find the block that copies `kbd-goal` to
each platform and add an identical block for `pmpo-elicit` immediately after.

### `skills/process/pmpo-elicit/SKILL.md` (MODIFY)

Add a `## Platform Mode` section after the existing `## Modes` section:

```markdown
## Platform Mode

On **Claude Code**, option 3 ("research it for me") runs in-session and the
question UI uses `AskUserQuestion` synchronously — no checkpoint file needed.

On **all other platforms** (Codex, OpenCode, Kimi, Zed), elicitation uses the
file-based async contract:

1. The caller invokes `scripts/pmpo-elicit-checkpoint.sh <elicit-dir> "<question>" <criticality> <caller> [hints...]`.
2. The script writes `request.json`, `checkpoint.json`, and `request-prompt.txt`
   under `<elicit-dir>/`, then exits 2 (BLOCKED).
3. The operator reads `request-prompt.txt` (human-readable) and writes `result.json`
   with their answer.
4. The caller invokes `scripts/pmpo-elicit-resume.sh <elicit-dir>` to read the
   result and continue.

See `references/checkpoint-contract.md` for the full caller protocol and
`references/escalation-points.md` for the platform routing table.
```

## Tasks

- [x] 1. Add `pmpo-elicit` to the process skills install list in `install-skills-flat.sh`
- [x] 2. Add `## Platform Mode` section to `SKILL.md`
- [x] 3. `npm run validate:strict skills/process/pmpo-elicit` passes clean
