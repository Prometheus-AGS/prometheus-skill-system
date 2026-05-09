# Execution Plan — change-001-finish-a2ui-domain

**Phase**: phase-a2ui-agui-artifact-refiner
**Backend**: OpenSpec, upstream `GQAdonis/artifact-refiner-skill`
**Submodule path**: `skills/imported/artifact-refiner/`
**Submodule current SHA**: `55e8625` (`vite`)
**Upstream `origin/main` SHA**: `a8d3383` (newer — fast-forward needed before branching)
**Recommended agent rotation**: `architect` (schema design) → `tdd-guide` (renderer + example) → `code-reviewer` (final pass)
**Dispatch tool**: claude-code (manual sequencing — not auto-dispatched to Roo/Cursor)

---

## What changed in scope after reading upstream state

The original assessment over-counted gaps. Upstream change `add-browser-rendering-for-htmx-react-artifacts` already shipped:

- `scripts/render-preview.mjs` (general renderer used by both UI and A2UI)
- `scripts/compile-tsx-preview.mjs`
- `references/schemas/artifact-manifest.schema.json` extended with `preview` block
- `references/domain/a2ui.md` updated with preview requirements
- `prompts/execute.md`, `prompts/plan.md`, `agents/pmpo-executor.md`, `agents/artifact-validator.md` updated for preview flow
- `scripts/post-execute-check.sh` and `scripts/validate-manifest.sh` updated

**Therefore change-001 does NOT add a new render script.** That would duplicate `render-preview.mjs`. The actual remaining gaps are narrower than the assessment originally framed.

## Real remaining gaps (verified by reading the upstream tree)

1. **`references/schemas/content-type.schema.json`** — `content_type` enum has no `direct:a2ui` value. Current values stop at `direct:code` and `meta:*`. A2UI cannot be cleanly typed today.
2. **`references/schemas/a2ui-component.schema.json`** — does not exist. The structural rules in `references/domain/a2ui.md` (component hierarchy, no circular refs, no undefined bindings, naming conventions) are prose-only. Constraint validation has no schema to point at.
3. **`prompts/specify.md` Content Type Detection** — heuristics distinguish `meta:*` vs `direct:*` but do not recognize A2UI markers (e.g., `a2ui` keyword, `.a2ui.json` extension, `version: "a2ui/v1"` field).
4. **`prompts/meta-controller.md` Content Type Routing** — needs explicit `direct:a2ui` → `references/domain/a2ui.md` adapter row.
5. **`examples/a2ui-component-refinement/`** — does not exist. `ui-preview-refinement` exists but is React/HTMX-flavored, not an A2UI spec demonstration.
6. **`SKILL.md` "Supported Artifact Domains"** — A2UI line should reference the new schema and example once they exist.

These are the only real edits. No new script. No new bash plumbing.

## OpenSpec proposal layout (to be authored upstream)

Path inside the submodule's repo: `openspec/changes/finish-a2ui-domain/`

Files:
- `proposal.md` — Why / What Changes / Capabilities / Impact, mirroring the structure of `add-browser-rendering-for-htmx-react-artifacts/proposal.md`.
- `tasks.md` — ordered checklist matching the six gaps above.
- `design.md` — short design note explaining the A2UI component schema choice (codified vs imported from upstream A2UI spec if one exists).
- `specs/` — one capability spec per new capability:
  - `a2ui-content-type/` — registers `direct:a2ui` in `content-type.schema.json`.
  - `a2ui-component-schema/` — adds the JSON Schema and constraint validation hook.
  - `a2ui-detection-routing/` — adds detection heuristics in `specify.md` and the routing entry in `meta-controller.md`.
  - `a2ui-example/` — `examples/a2ui-component-refinement/` end-to-end.

## Pre-flight gate (verified)

Submodule is detached at `55e8625` ("vite"). `git merge-base --is-ancestor 55e8625 origin/main` returns true and `git rev-list --left-right --count 55e8625...origin/main` reports `0    1`. Translation: `55e8625` is reachable from `origin/main`, exactly one commit behind. Fast-forward to `origin/main` is **safe and lossless** — no local-only work to preserve.

Sequence:
1. In the submodule: `git checkout -b finish-a2ui-domain origin/main`
2. Author OpenSpec proposal + code on that branch.
3. Push branch + open PR upstream.
4. After upstream merge + tag: change-003 bumps the pointer here.

The pointer this repo records is **not** changed by step 1 — submodule branch creation is local until pushed. The pointer only moves when change-003 runs.

## Tasks (in order)

### Step A — Reconcile submodule with upstream (gated on user confirmation)
- [ ] Verify whether `55e8625` ("vite") is reachable from `origin/main`. If yes, fast-forward is safe. If no, decide: keep the local commit and rebase onto upstream, or discard.
- [ ] After confirmation: `git checkout -b finish-a2ui-domain origin/main` inside the submodule.

### Step B — Author the OpenSpec proposal (upstream)
- [ ] Create `openspec/changes/finish-a2ui-domain/proposal.md` covering all six gaps.
- [ ] Create `openspec/changes/finish-a2ui-domain/tasks.md` with the checkpoint list below.
- [ ] Create `openspec/changes/finish-a2ui-domain/design.md` documenting the schema-source decision (do we own the A2UI schema or import an upstream canonical one?).
- [ ] Create one `specs/<capability>/` directory per new capability with a `spec.md` per OpenSpec convention.

### Step C — Implement (upstream, on `finish-a2ui-domain` branch)
- [ ] Add `direct:a2ui` to `references/schemas/content-type.schema.json` enum.
- [ ] Author `references/schemas/a2ui-component.schema.json` codifying the constraints from `references/domain/a2ui.md` (component hierarchy, no circular refs, required fields, naming).
- [ ] Update `prompts/specify.md` — add A2UI detection heuristics under Content Type Detection.
- [ ] Update `prompts/meta-controller.md` — add `direct:a2ui` row to the Content Type Routing table mapping to `references/domain/a2ui.md`.
- [ ] Update `prompts/execute.md` — register A2UI normalization step before the existing `render-preview.mjs` invocation.
- [ ] Author `examples/a2ui-component-refinement/` with `artifact_manifest.json`, starter `.a2ui.json` spec, dist output, README. Mirror `examples/ui-preview-refinement/` shape.
- [ ] Update root `SKILL.md` Supported Artifact Domains row for A2UI.
- [ ] Verify: `bash scripts/state-init.sh demo-a2ui ui-component direct:a2ui` initializes; refiner runs end-to-end; intentional circular reference fails the constraint check.

### Step D — Submit upstream
- [ ] Commit on `finish-a2ui-domain` branch with conventional-commit messages.
- [ ] Push branch to `origin`.
- [ ] Open PR against `GQAdonis/artifact-refiner-skill:main`.
- [ ] After review/merge upstream, request a tag (`v1.2.0` or whatever the maintainer chooses).

### Step E — Update phase progress here
- [ ] Once upstream PR is open: set `change-001-finish-a2ui-domain.status = "in_review"` in `progress.json`.
- [ ] Once merged + tagged: set status to `done` and unblock change-003.

## Per-change artifact-refiner QA gate

Skipped for the OpenSpec **proposal** itself (docs-only, fewer than 3 source files). Required after Step C implementation lands and before Step D.

When triggered: `/refine-validate change-001-finish-a2ui-domain` with the constraints in `.kbd-orchestrator/constraints.md` (creates that file as part of QA setup if absent — out of scope for this change).

## Out of scope for change-001

- Any AG-UI work — that's change-002.
- Any submodule pointer bump in this repo — that's change-003.
- Any TheBoss / cherry-studio integration — separate ticket.
- New render scripts — `render-preview.mjs` already covers it.
- Refactoring the existing preview pipeline.

## What I will NOT do without user confirmation

- Touch the submodule's pinned SHA in this repo (Step A item 2).
- Discard or rebase the local `55e8625` commit ("vite") in the submodule.
- Push any branch upstream.
- Open any pull request.
