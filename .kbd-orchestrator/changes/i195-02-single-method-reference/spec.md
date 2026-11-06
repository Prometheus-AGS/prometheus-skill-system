# One identical selected-method core in all stage skills

Phase: issue-195-closeout · Change: i195-02-single-method-reference · Depends on: i195-01-policy-conflict-ack

## Why

The `## Selected-method recovery and authority` section is hand-copied into 11 stage skills (two byte variants today, differing by one trailing blank line). Issue #195 acceptance 5 needs source, generated and installed surfaces to keep the same semantics; copies drift.

## What changes

- Replace the section in the 11 stage skills (kbd-assess, kbd-child-exit, kbd-execute, kbd-handoff, kbd-init, kbd-new-child, kbd-next-child, kbd-pause, kbd-plan, kbd-resume, kbd-spec) with one identical short core (below) that keeps the load-bearing rules inline and links `references/method-context.md` for the rest. A bare link is rejected: a model may not open it.
- C-01 reconciliation (constraints.md:8-19): run `npm run build:distribution` and `npm run generate:skills-index` twice, record the SHA-256 of every tracked output after each run in `.kbd-orchestrator/phases/issue-195-closeout/evidence/reconciliation.json` and require them byte-identical; then run `npm run check:distribution`, `npm run check:skills-index` and `npm run validate:harness-adapters`. No distribution certification is claimed before this.
- Add `scripts/check-selected-method-core.mjs` (wired as `npm run check:selected-method`) that fails when any stage skill's section differs byte-for-byte from the canonical core or is missing.
- Deviation from goal B's literal wording ("one reference file"): the reference file stays single, but each stage skill keeps an identical short inline core because a bare link may not be opened; the #195 mapping must report goal B as satisfied in this adapted form, not literally.
- `dist/**` is tracked (4,482 files per `git ls-files dist` at 6bb66d5c). Regenerate and commit tracked outputs in the same change: `npm run build:distribution` (dist/plugins/claude/**, dist/plugins/codex/**, .claude-plugin/marketplace.json, .agents/plugins/marketplace.json) and `npm run generate:skills-index` (SKILLS.md).

Canonical core:

```markdown
## Selected-method recovery and authority

Before dependent work, run `shared/method-context.mjs --project <project-root> --phase <canonical-phase-id> --write` from the resolved orchestrator root (or read the phase-local `method-context.md` when Node is unavailable) and act on its `status`:

- `RESOLVED_REFERENCES` — continue with the recovered method, contract and stage authority; preserve the same references in role inputs, dispatch and handoff read order.
- `BLOCKED` — a method decision, contract, calibration or supersession is missing or changed; resolve it explicitly. Prior commands, memory and tool inventories are not fresh authority.
- `CONFLICT` — governing host policy conflicts with the selected method. Stop before dependent work and show the conflict; continue only after the operator records a `KBD-POLICY-ACK` decision for that file's current hash. Never rewrite operator-owned policy.

For selected VLBV, define acceptance before generation and finish the whole agreed value phase before any generated-code verification; tasks, hours, commits, reviewer requests and compiler problems create no boundary. Preserve original evidence, consumed budgets and unresolved effects across resume and supersession; reconcile an uncertain effect before any retry. Full rules: `references/method-context.md`.
```

## Scope

- `skills/process/kbd-process-orchestrator/skills/kbd-assess/SKILL.md`
- `skills/process/kbd-process-orchestrator/skills/kbd-child-exit/SKILL.md`
- `skills/process/kbd-process-orchestrator/skills/kbd-execute/SKILL.md`
- `skills/process/kbd-process-orchestrator/skills/kbd-handoff/SKILL.md`
- `skills/process/kbd-process-orchestrator/skills/kbd-init/SKILL.md`
- `skills/process/kbd-process-orchestrator/skills/kbd-new-child/SKILL.md`
- `skills/process/kbd-process-orchestrator/skills/kbd-next-child/SKILL.md`
- `skills/process/kbd-process-orchestrator/skills/kbd-pause/SKILL.md`
- `skills/process/kbd-process-orchestrator/skills/kbd-plan/SKILL.md`
- `skills/process/kbd-process-orchestrator/skills/kbd-resume/SKILL.md`
- `skills/process/kbd-process-orchestrator/skills/kbd-spec/SKILL.md`
- `scripts/check-selected-method-core.mjs`
- `package.json`
- `dist/plugins/claude/**`
- `dist/plugins/codex/**`
- `.claude-plugin/marketplace.json`
- `.agents/plugins/marketplace.json`
- `SKILLS.md`

## Constraints

- Recalled lesson: "Use rebase-regenerate.sh for Generated-Only Rebase Conflicts" — resolve generated-only dist/ conflicts with scripts/rebase-regenerate.sh.
- Recalled lesson: "A CRITICAL shipped despite all per-change gates passing … the installer never applied the Codex memory setting" — surface checks observe the installer-dispatched payload, never source text alone.
- Recalled lesson: "`kbd-apply mark-done` updates the task flag but does not synchronize the canonical KBD ledger" — close tasks only with kbd-apply begin-task/end-task.
- Generated outputs: i195-02 task 2 is the named C-01 reconciliation owner (twice-run, byte-identical hashes, drift validators); i195-01 records that owner in its persisted execution evidence (execution.md and the end-task note) and claims no distribution certification.
- Review before archive: i195-03 task 3 runs the independent cumulative-diff review whose receipt covers i195-01 and i195-02; neither archives before that receipt (or a recorded owner waiver).
- Implementation first (CLAUDE.md, highest precedence): no tests, builds or checks before every planned production change in the phase is complete; acceptance is gathered only at the final boundary owned by i195-03.
- Exclusions: #184 exit semantics, #185 trusted tool admission, the operator-authority trust model, a generic shell fence, and any edit to CLAUDE.md/AGENTS.md policy text.
