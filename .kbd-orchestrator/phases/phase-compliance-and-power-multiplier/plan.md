# Phase Plan — phase-compliance-and-power-multiplier

> **Source assessment**: [`assessment.md`](assessment.md) (437 lines, §8 has the priority-ordered punch list)
> **Backend**: native KBD (no `openspec/` present, no `.evolver/`)
> **Date**: 2026-04-28
> **Targeted next phase**: `phase-librefang-wasm-onramp` (P0 batch only) — out of scope here

## Strategy

The assessment identifies 27 gaps across 6 priority bands (P0–P3, plus Backlog). Rather
than try to land all 27 in one phase, this plan groups them into **eight ordered
changes** that are each independently shippable, testable, and small enough to execute
in 1–3 days by one engineer.

The ordering is dictated by three rules:

1. **Quick-wins first** — compliance/correctness fixes that take hours unblock
   trust in everything that follows (`change-001`).
2. **Bootstrap before product** — without a working build environment, the WASM
   onramp can't be tested (`change-002`).
3. **Vertical slice over horizontal coverage** — once bootstrap lands, build the WASM
   path end-to-end (`change-003 → change-005`) before broadening to opencode/ideation.

Changes 6–8 are post-P0 polish that can run in parallel with verification (the
scheduled remote agent on 2026-05-05) once the headline path is proven.

## Ordered Change List

| # | Change ID | Title | Gaps Addressed | Priority | Effort | Recommended Agent |
|---|-----------|-------|----------------|----------|--------|-------------------|
| 1 | `change-001-compliance-quickfixes` | Tighten validator + remove dangling refs + populate empty plugin dirs | A1, A3, B1 | P1 | XS (≤2h) | `code-simplifier` |
| 2 | `change-002-toolchain-bootstrap` | Auto-build submodule binaries + add `wasm32-wasip2` target + `npm run doctor` | F1, F2, F3, F4 | P0 | S (≤1d) | `devops-engineer` |
| 3 | `change-003-librefang-wasm-skill` | New `skills/rust/librefang-wasm-skill/` with WASM-ABI templates | G1 | P0 | M (1–3d) | `rust-skills:rust-skill-creator` |
| 4 | `change-004-native-agent-wasm-target` | `--target librefang-wasm` flag in native-agent + WASM crate emission | G2, E1, E3 | P0 + P2 | M (1–3d) | `code-architect` |
| 5 | `change-005-package-and-upload` | `forge package-librefang` + `/upload-to-bossfang` + `/start-business-build` orchestrator | G3, H4, G4, G5 | P0 + P2 | M (1–3d) | `architect` |
| 6 | `change-006-karpathy-loop-hooks` | `UserPromptSubmit` → `pk focus` + `Stop` → `forge reflect` + per-skill license fields | C1, C2, A4 | P1 | S (≤1d) | `harness-optimizer` |
| 7 | `change-007-opencode-real-plugin` | Real `.opencode/plugin.ts` Plugin export + `@opencode-ai/sdk` + auto-`opencode.json` | D1, D2, D3 | P1 | S (≤1d) | `typescript-reviewer` |
| 8 | `change-008-rustbpe-skill` | `skills/rust/karpathy-tokenizer/` teaching `rustbpe` usage | E2 | P2 | S (≤1d) | `rust-skills:rust-skill-creator` |

Out of scope for this phase (future):

- **change-009 (Backlog)** — `ideation-mindmap` skill (Gap H1) — defers to
  `phase-ideation-onramp` after the WASM path is proven.
- **Plugin-native `commands/` migration** (Gap B2) — defers to a dedicated
  marketplace-modernization phase since it touches every slash-command.
- **Private/public skill-hub story** (Gap H2) — long-term work; needs design.

## Critical Path

```
change-001 (XS, parallel)
    │
    ▼
change-002 (S) ──── unblocks WASM build for all that follows
    │
    ▼
change-003 (M) ──── the new skill that produces WASM-ABI-compliant skills
    │
    ▼
change-004 (M) ──── native-agent generator emits WASM crate
    │
    ▼
change-005 (M) ──── packaging + upload + headline orchestrator
    │
    ▼ (P0 batch complete — scheduled remote verification on 2026-05-05 fires here)
    │
    ├── change-006 (S, parallel) ──── close Karpathy loop
    ├── change-007 (S, parallel) ──── real opencode plugin
    └── change-008 (S, parallel) ──── rustbpe skill
```

Total wall-clock with one engineer: ~3 weeks. With two engineers parallelizing
changes 6–8 against 1–5: ~2 weeks.

## Recommended Agent per Change

The "Recommended Agent" column above is the *primary* agent. For implementation we
expect to also lean on:

- `tdd-guide` — every change with new code (3–8) should write tests first.
- `rust-reviewer` — for changes 2, 3, 4, 5, 8.
- `typescript-reviewer` — for change 7.
- `code-reviewer` — final review on every change before commit.
- `security-reviewer` — required on change-005 (it shells out to a user-supplied URL,
  classic SSRF surface) and change-002 (it executes `cargo build` and writes to
  `~/.local/bin`).

## Per-Change Files

Each change has a directory under `.kbd-orchestrator/changes/<change-id>/` with:

- `change.md` — proposal: context, scope, deliverables, acceptance criteria, tasks
- `tasks.md` — ordered task list with `[ ] / [/] / [x]` markers (created by execute phase)

Initial `change.md` files have been emitted for all 8 changes. The `tasks.md` files
are intentionally NOT created at plan time — they are produced by `/kbd-execute` so
the executor can decompose against the current state of the codebase.

## Verification & Phase Exit Criteria

This phase exits when:

1. All P0 gaps (G1, G2, G3, H4, F1, F2) are closed — verified by the scheduled remote
   agent on 2026-05-05 (routine `trig_01MK1jtQZj3z1mQ7joETevuJ`).
2. `npm run validate` is green.
3. End-to-end smoke test from assessment §9 produces a valid
   `<agent-name>.lf-skill.zip` containing a `skill.toml` with `runtime.type = "wasm"`.
4. The remote-agent-opened GitHub issue recommends "ready for
   `phase-librefang-wasm-onramp` implementation" (not "P0 incomplete").

P1/P2 gaps (changes 6–8) can land in this phase OR roll into the next; they are not
exit blockers.

## Notes for /kbd-execute

- The execute backend should be **`hybrid`** — changes 1, 2, 6, 7 are best done in
  Claude Code; change 3 is the Rust-skill author's wheelhouse (`rust-skills:rust-skill-creator`);
  changes 4, 5, 8 benefit from the architect agent for design + Rust agents for impl.
- Each change should produce a single conventional-commit PR. The `[evolver_item_id]`
  field in `change.md` is intentionally null — this phase is not driven by an
  evolver cycle.
- Hooks are already wired (`SubagentStop` → `state-checkpoint.sh`) so executor agents
  with `assessor`/`planner`/`executor`/`reflector` matchers will auto-checkpoint.
