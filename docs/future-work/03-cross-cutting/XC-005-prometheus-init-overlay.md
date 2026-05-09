---
id: XC-005
title: prometheus init project-scoped overlay
status: planned
priority: P1
estimated_effort: 2-3d
agent_role: skill-pack-maintainer
depends_on: [SP-008]
unblocks: []
related: [XC-001, XC-003]
created_from_conversation_turn: 3-4
---

# XC-005 — `prometheus init` project-scoped overlay

## Problem

When a new project adopts the prometheus stack, there are several files that should exist locally:

- `.prometheus/` — runtime data dir (per SP-008, with `knowledge/` subdir).
- `.kbd-orchestrator/` — phase tracking.
- `.gitignore` entries for `.prometheus/`, `.kbd-orchestrator/local/`.
- `BUG_FIX_LEDGER.md` (per XC-001).
- `bdd-video.config.json` (if BDD-004 productized, optional).
- `.prometheus/scratchpad-template.md` (per XC-003).
- Slash commands (the project's local overrides).
- A starter CLAUDE.md drawing rules from canonical (per SP-001).

Today this scaffolding is hand-rolled. Each new adoption produces drift.

## Evidence

Reason from SP-008's per-project KB scoping requirement: as soon as that lands, every project needs the local `.prometheus/` dir and its conventions. Without an init command, each adopter misses some piece.

## Why it matters

- **Adoption friction.** A new project should adopt the stack with one command, not five-to-fifty manual setup steps.
- **Drift prevention.** Consistent scaffolding means consistent on-disk artifact layout, which means doctor (XC-004) can make assumptions.
- **Reversibility.** A single command makes it easy to opt back out (`prometheus uninit`) for cleanup or experimentation.

## Proposed fix

A `prometheus init` command that:

1. **Detects the project root** (per SP-008's resolution).
2. **Scaffolds the overlay** by copying templates from the skill-pack:
   - Creates `.prometheus/` and subdirectories.
   - Creates `.kbd-orchestrator/` if not present.
   - Adds entries to `.gitignore`.
   - Drops `BUG_FIX_LEDGER.md` template if not present.
   - Drops `bdd-video.config.json` template if BDD project (detected by presence of `tests/features/`).
   - Drops `scratchpad-template.md`.
   - Sets up local `CLAUDE.md` with `# This project follows prometheus-skill-pack conventions; see <link>` plus project-specific rules.
3. **Reports** what was created vs already-existing vs skipped.
4. **Idempotent** — running twice is safe; existing files aren't overwritten without `--force`.

A complementary `prometheus uninit` removes the overlay (with confirmation prompts).

## Trade-offs and risks

- **Risk: templates drift from skill-pack canonical.** Mitigation: templates live in `prometheus-skill-pack/templates/project-init/` and are the single source. The init command pulls from there.
- **Risk: project's existing files conflict with templates.** Mitigation: idempotent semantics — existing files are preserved, missing ones added. `--diff` mode shows what would change without applying.
- **Cost: maintaining the templates.** Bounded; templates rarely change once a project type is well-defined.

## Acceptance criteria

- [ ] `prometheus init` runs successfully in a clean project root.
- [ ] Idempotent: running twice produces no error and no duplicate entries.
- [ ] `--diff` mode shows pending changes without applying.
- [ ] `--force` overwrites existing files (with explicit confirmation).
- [ ] BDD project detection produces extra files; non-BDD projects don't get them.
- [ ] `prometheus uninit` exists and removes scaffolded files (after confirmation).
- [ ] Documentation: how to use, when to re-init.

## Implementation steps

1. Define the templates directory in skill-pack.
2. Author each template file.
3. Implement the project-type detector (BDD project? Rust workspace? Next.js?).
4. Implement the scaffold writer with idempotent semantics.
5. Implement uninit.
6. Test on a fresh project.

## Dependencies

SP-008 (per-project KB scoping must exist for the `.prometheus/knowledge/` template to make sense).

## Open questions

- Should init also register the project with surreal-memory (e.g. create a project entity)? Reasonable; gated on SP-019 (LibrarianEvent persistence).
- Should there be project-type templates (Rust, Next.js, mixed)? Yes — start with Next.js (SSR baseline) and add as needed.
