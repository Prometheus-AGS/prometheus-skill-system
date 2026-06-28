# change-goal-010: Kimi Code Evaluator Skill

**Phase:** goal-loop-support
**Status:** pending
**Sub-phase:** B (integration)
**Depends on:** change-goal-001, change-goal-003

## Problem

Kimi Code's `/goal next` is a queue primitive, not a condition-based loop. There is no built-in mechanism to evaluate "is my stopping condition met?" after each turn — the builder model would self-grade without a separate evaluator skill.

## Solution

Build `kbd-goal-check` as a SKILL.md that Kimi auto-discovers and invokes after each turn during a goal loop. It reads the stopping condition from `goal.json`, checks evidence in STATE.md / runs check commands, and returns PASS (with evidence) or CONTINUE (with next action hint).

## Files

- `skills/process/kbd-goal-check/SKILL.md` (CREATE)
- `skills/process/kbd-goal/references/platforms/kimi.md` (CREATE)

## Tasks

- [ ] Write `skills/process/kbd-goal-check/SKILL.md` with valid frontmatter
- [ ] Skill body: read `goal.json → phases[active_phase].stopping_condition`; read STATE.md; evaluate; return PASS (evidence quote) or CONTINUE (next TASKS.md action)
- [ ] On PASS: instruct agent to mark Kimi goal complete and queue next phase with `/goal next <next-phase-condition>`
- [ ] Validate: `npm run validate:strict skills/process/kbd-goal-check`
- [ ] Write `kimi.md` platform reference documenting `/goal next` queue pattern + evaluator skill integration
- [ ] Add install entry for `~/.kimi-code/skills/` in `install-skills-flat.sh`
- [ ] Update `kbd-goal/SKILL.md` Kimi section
