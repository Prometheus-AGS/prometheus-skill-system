EXECUTION: phase-okf-llm-wiki-adoption
Project: prometheus-skill-system (+ sibling prometheus-knowledge-rs for changes 003-006)
Date: 2026-07-01
Selected backend: native-tool (native-kbd, driven via /kbd-apply)
Dispatched to: SELF (Claude Code CLI)
Backend rationale: CORRECTED 2026-07-01 — `openspec/` and the `openspec` CLI
DO exist at project root (91 pre-existing changes); the original claim of "no
openspec" was a gap in my own assessment. However, `kbd-apply`'s auto-detect
resolves the backend once per repo (not per change), and picking `openspec`
here would route `verify` through `os_verify` → `openspec validate`, which
fails even on an existing done change (`change-elicit-001`) because none of
the 91 existing changes have a real `specs/` delta directory — they are
PMPO-shaped `proposal.md`/`tasks.md`, not true OpenSpec deltas with
`## ADDED/MODIFIED Requirements` + `#### Scenario:` blocks. That verify path
has apparently never been exercised successfully in this repo (flagged
separately, out of this phase's scope). `nk_verify` (native-kbd: per-task
verify commands + structural check) is the correct fit for this phase's
infra/vendoring/cross-repo work. `.kbd-orchestrator/project.json` now pins
`specBackend: native-kbd` explicitly (with rationale recorded in that file)
so auto-detection cannot silently flip this phase to `openspec` mid-execution.
Legacy `change.md` present under each `.kbd-orchestrator/changes/change-okf-*/`
is lazy-migrated to `tasks.json` on first `kbd-apply list`.
Backend entrypoint: `kbd-apply.sh` (or `/kbd-apply`) per change, task-by-task —
never bare `/opsx:apply` — this repo's own rule against skipping the KBD task
loop, and doubly true here since `/opsx:apply` would ignore the specBackend pin.
OpenSpec available: YES (repo-wide) — NOT SELECTED for this phase (pinned to native-kbd; see rationale above)
Source plan: .kbd-orchestrator/phases/phase-okf-llm-wiki-adoption/plan.md

MODEL NOTE: .kbd-orchestrator/project.json is absent (kbd-new-phase warned;
no model_policy to resolve concrete models from). Per the assess-phase default
fallback, all changes execute on the current session model until project.json
is seeded. Session model was switched to claude-sonnet-5 immediately before
this command. `Model class` per change is carried from plan.md verbatim for
future routing once project.json exists.

EXECUTION SCOPE

- change-okf-001-vendor-specs: Vendor OKF v0.1 + Karpathy docs; record decision in CLAUDE.md
- change-okf-002-pk-workspace-baseline: Clone prometheus-knowledge-rs sibling; build baseline; diagnose pk ingest bug
- change-okf-003-permissive-okf-parser: OKF §9 permissive parser + reserved-filename handling
- change-okf-004-okf-writer-and-id-mapping: OKF frontmatter emitter + path-based concept-ID mapping
- change-okf-005-index-log-and-body-links: index.md/log.md maintenance + body links/Citations
- change-okf-006-okf-lint: OKF conformance rules in pk lint
- change-okf-007-llm-wiki-skills: llm-wiki skill (ingest/query/lint) + wiki schema doc
- change-okf-008-integration-verification: Hooks/MCP/e2e verification

DISPATCH CONTRACTS

- change-okf-001-vendor-specs → SELF (claude-code)
  Entry: kbd-apply begin-task change-okf-001-vendor-specs <task-id>, one task at a time
  Model class: small
  Concrete model: session model (claude-sonnet-5) — model_policy unresolved
  Model rationale: docs-only, ≤3 tasks, no new abstractions
  Progress file: .kbd-orchestrator/phases/phase-okf-llm-wiki-adoption/progress.json
  Handoff: kbd-apply end-task marks task done, syncs progress.json + waypoint

- change-okf-002-pk-workspace-baseline → SELF (claude-code)
  Entry: kbd-apply begin-task change-okf-002-pk-workspace-baseline <task-id>
  Model class: medium
  Concrete model: session model (claude-sonnet-5)
  Model rationale: cross-repo clone + build/test baseline + bounded diagnosis, single module
  Progress file: same
  Handoff: same; diagnosis timebox (~2h) enforced by the change's own task text —
  if exceeded, mark that task's notes with "DEFERRED: see follow-up" and continue

- change-okf-003-permissive-okf-parser → SELF (claude-code)
  Entry: kbd-apply begin-task change-okf-003-permissive-okf-parser <task-id>
  Model class: frontier
  Concrete model: session model (claude-sonnet-5) — flagged: plan.md calls for
  frontier for 003-005; no project.json to enforce a MODEL MISMATCH stop, so
  proceeding on current session model per fallback, noted as a risk in FALLBACK
  CONDITIONS below
  Model rationale: parser semantics change is cross-domain (pk-store + pk-core),
  new permissive-deserialization abstraction, backward-compat constraint
  Progress file: same
  Handoff: same; depends on change-okf-002 (sibling checkout must exist first)

- change-okf-004-okf-writer-and-id-mapping → SELF (claude-code)
  Entry: kbd-apply begin-task change-okf-004-okf-writer-and-id-mapping <task-id>
  Model class: frontier
  Concrete model: session model (claude-sonnet-5)
  Model rationale: new ID-mapping abstraction, cross-file (pk-store + pk-core)
  Progress file: same
  Handoff: same; depends on change-okf-003 (parser must be permissive first)

- change-okf-005-index-log-and-body-links → SELF (claude-code)
  Entry: kbd-apply begin-task change-okf-005-index-log-and-body-links <task-id>
  Model class: frontier
  Concrete model: session model (claude-sonnet-5)
  Model rationale: prompt-engineering + store changes across pk-librarian and
  pk-store, largest task count (L effort) in this phase
  Progress file: same
  Handoff: same; depends on change-okf-004

- change-okf-006-okf-lint → SELF (claude-code)
  Entry: kbd-apply begin-task change-okf-006-okf-lint <task-id>
  Model class: medium
  Concrete model: session model (claude-sonnet-5)
  Model rationale: bounded rule-addition to existing lint pass, single module
  Progress file: same
  Handoff: same; depends on change-okf-003 fully, change-okf-005 for reserved-file checks

- change-okf-007-llm-wiki-skills → SELF (claude-code)
  Entry: kbd-apply begin-task change-okf-007-llm-wiki-skills <task-id>
  Model class: frontier (plan.md) — see model rationale
  Concrete model: session model (claude-sonnet-5)
  Model rationale: new skill authorship + schema doc is higher-judgment than
  typical medium work despite modest file count; plan.md's frontier tag is honored
  Progress file: same
  Handoff: same; drafting can start Round 2 (depends only on change-okf-001);
  final example verification blocked on change-okf-004

- change-okf-008-integration-verification → SELF (claude-code)
  Entry: kbd-apply begin-task change-okf-008-integration-verification <task-id>
  Model class: medium
  Concrete model: session model (claude-sonnet-5)
  Model rationale: verification/wiring task, no new abstractions
  Progress file: same
  Handoff: same; depends on change-okf-005, change-okf-006, change-okf-007;
  this is the phase's mastery/goal re-check point before reflect

APPROVAL GATES

- NONE for changes 001, 002, 006, 007, 008 (self-contained within owned repos)
- change-okf-003/004/005 touch prometheus-knowledge-rs, a separate repo with its
  own remote (github.com/Prometheus-AGS/prometheus-knowledge-rs) — pushing or
  opening a PR against that remote is a shared-state action per this session's
  operating rules; confirm with the user before pushing/opening a PR upstream.
  Local commits in the sibling checkout during execution are fine.

FALLBACK CONDITIONS

- If `.kbd-orchestrator/project.json` is seeded mid-phase with a model_policy
  that maps frontier to a different concrete model than the session model,
  re-dispatch remaining frontier-class changes (003, 004, 005) to that model
  before continuing — do not silently continue on session model past that point.
- If change-okf-002's diagnosis of the `pk ingest` LLM failure exceeds the
  ~2h timebox or reveals a defect outside pk-store/pk-librarian (e.g. a
  provider-routing crate), stop drilling, document the finding, and change-okf-008
  degrades from e2e verification to static format verification (per plan.md
  DEFERRED section) — this is an accepted, planned degradation, not a blocker.
- If any native-kbd change proves too opaque to track via tasks.json (e.g.
  scope balloons past the plan's task list), fall back to OpenSpec for that
  change: `mkdir openspec/` + `/opsx:new <change-id>`, migrate the task list,
  and continue via `/kbd-apply` against the openspec backend instead.

VERIFICATION REQUIREMENTS

- This repo: `npm run validate` (all skills) and `npm run validate:strict` for
  the new change-okf-007 skill specifically.
- prometheus-knowledge-rs sibling: `cargo build --workspace` and
  `cargo test --workspace` after each of changes 002-006.
- change-okf-008: end-to-end `pk ingest` → `index.md`/`log.md` updated →
  `pk focus`/`pk search` round-trip (or static fallback per FALLBACK CONDITIONS).

PROGRESS LEDGER

- [PENDING] change-okf-001-vendor-specs — claude-code
- [PENDING] change-okf-002-pk-workspace-baseline — claude-code
- [PENDING] change-okf-003-permissive-okf-parser — claude-code
- [PENDING] change-okf-004-okf-writer-and-id-mapping — claude-code
- [PENDING] change-okf-005-index-log-and-body-links — claude-code
- [PENDING] change-okf-006-okf-lint — claude-code
- [PENDING] change-okf-007-llm-wiki-skills — claude-code
- [PENDING] change-okf-008-integration-verification — claude-code

OUTPUTS

- NONE yet — dispatch contract only; artifacts land as each change executes

BLOCKERS

- NONE at dispatch time. Two known risks tracked, not yet blocking:
  1. prometheus-knowledge-rs has no local working checkout (resolved by change-okf-002, Round 1).
  2. `pk ingest` fails locally with "LLM error: failed to parse LLM response" (diagnosed by change-okf-002; may degrade change-okf-008 per FALLBACK CONDITIONS).

REFLECTION HANDOFF

- kbd-reflect should consume: final goals.md status (all 5 goals, re-checked
  against change-okf-008's verification), whether the pk ingest bug was fixed
  or deferred (and to where), whether prometheus-knowledge-rs changes were
  merged/pushed upstream or remain local-only (approval-gated per this
  execution.md), and whether the KB-empty migration-free window (noted in
  assessment.md) held through execution or a migration concern surfaced.

EXECUTION READY
