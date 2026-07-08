---
id: change-drs-008-install-smoke-test
title: Install deep-research skill to platforms + run smoke test
phase: phase-deep-research-skill
priority: P1
effort: S
wave: 4
agent: general-purpose
status: pending
gap_id: G-05
verdict: BUILD
depends_on: change-drs-007-docs-updates
scope:
  - ~/.claude/skills/research/deep-research/ (installed copy)
  - ~/.claude/skills/research/deep-research/skills/ (sub-skills)
---

# change-drs-008 — Install + Smoke Test

## Context

Install the skill to the local Claude Code user scope and verify it loads
correctly. Smoke test confirms the SKILL.md is parseable and the skill name
appears in Claude Code's skill registry.

## Install Commands

```bash
# Install to user scope (Claude Code primary platform)
npm run install:user

# Verify installed
ls ~/.claude/skills/research/deep-research/
ls ~/.claude/skills/research/deep-research/skills/
```

## Smoke Test

The smoke test verifies:
1. SKILL.md is present and has valid frontmatter
2. All 10 sub-skill SKILL.md files are present
3. Scripts are executable in the installed copy
4. No broken symlinks

```bash
# Check parent skill
test -f ~/.claude/skills/research/deep-research/SKILL.md && echo "PASS: parent SKILL.md"

# Check sub-skills (all 10)
for i in 01 02 03 04 05 06 07 08 09 10; do
  dir=$(ls -d ~/.claude/skills/research/deep-research/skills/stage-${i}-* 2>/dev/null | head -1)
  if [ -f "${dir}/SKILL.md" ]; then
    echo "PASS: stage-${i} SKILL.md"
  else
    echo "FAIL: stage-${i} SKILL.md missing"
  fi
done

# Check scripts executable
for s in run-research export-package verify-sources build-graph detect-contradictions; do
  f=~/.claude/skills/research/deep-research/scripts/${s}.sh
  [ -x "$f" ] && echo "PASS: ${s}.sh executable" || echo "FAIL: ${s}.sh not executable"
done
```

## Acceptance Criteria

- [ ] `npm run install:user` exits 0
- [ ] `~/.claude/skills/research/deep-research/SKILL.md` exists
- [ ] All 10 stage sub-skill SKILL.md files exist in installed location
- [ ] All 5 scripts are executable in installed location
- [ ] Smoke test script outputs 0 FAIL lines
