# Execution — phase-learn-feynman

**Phase:** phase-learn-feynman
**Backend:** openspec
**Dispatch contract:** Claude Code (claude-sonnet-4-6) executing via direct tool calls
**Written:** 2026-06-28
**Total changes:** 28
**First pending change:** change-learn-001

---

## Backend Selection

OpenSpec is present (`openspec/` directory at project root, 28 `change-learn-*` proposals committed). Backend = `openspec`.

All changes tracked in `openspec/changes/change-learn-*/proposal.md` and `tasks.md`. Progress mirrored in `progress.json`.

---

## Execution Groups and Parallelism

### Group 0 — Parallel start (no dependencies)
- `change-learn-001` — Spike: learner-model schema + CRDT semantics (design-only)
- `change-learn-002` — Spike: detect-surface-tier probe (design + shell)
- `change-learn-016` — Meta-grounding corpus (content-only)

### Group 1 — Layer A substrate (after spikes)
- `change-learn-003` — content-grounding service
- `change-learn-004` — KB adapter (after 003)
- `change-learn-004b` — storage-provider Rust crate (after 001)
- `change-learn-005` — learner-model Rust crate (after 001)

### Group 2 — Layer B UI primitive (after 002)
- `change-learn-006` — ui-surface skill
- `change-learn-020` — domain directory scaffolding (parallel with Group 3)

### Group 3 — Layer C first wave (after 003+004+006+001+005)
- `change-learn-007` — learn-goal skill
- `change-learn-008` — learn-survey skill
- `change-learn-009` — learn-grade skill
- `change-learn-010` — feynman-loop skill

### Group 4 — Layer C second wave (after first wave)
- `change-learn-011` — learn-plan skill
- `change-learn-012` — learn-retain skill
- `change-learn-013` — learn-practice skill
- `change-learn-014` — learn-certify skill

### Group 5 — KB management (after 003+004)
- `change-learn-015` — learn-kb skill

### Group 6 — Meta-learning / adoption (after 016+007+006)
- `change-learn-017` — learn-about-system skill
- `change-learn-018` — learn-harness skill

### Group 7 — Integration tests (after respective groups)
- `change-learn-021` — basic flow test (after Group 3)
- `change-learn-022` — full loop test (after Group 4)
- `change-learn-023` — KB test (after 015+021)
- `change-learn-024` — meta test (after 017+018)

### Group 8 — Tier 2 + release (last)
- `change-learn-019` — surface-bridge Axum server (LAST interactive enhancement)
- `change-learn-025` — install-skills-flat.sh update
- `change-learn-026` — docs/guide update
- `change-learn-027` — CLAUDE.md update
- `change-learn-028` — v1.4.0 release bump

---

## QA Policy

- Changes with ≥3 files modified: artifact-refiner QA gate before archiving
- Design-only changes (001, 002): skip QA (documentation output only)
- Content-only changes (016): skip QA
- Test changes (021-024): skip QA
- Documentation changes (026, 027): skip QA
- All skill changes (006-015, 017-018): QA gate required
- Rust crate changes (004b, 005, 019): QA gate required
- Release change (028): QA gate required

---

## Dispatch Contract

Execute begins with `change-learn-001`. Each change is applied via direct Claude Code tool calls. After each change completes:
1. Mark `status: DONE` in `progress.json`
2. Run QA gate (if applicable)
3. Archive via `openspec/changes/<id>/` status update
4. Advance `next_pending_change` in waypoint
5. Continue to next change per execution order

---

*Execution initialized. Begin with change-learn-001.*
