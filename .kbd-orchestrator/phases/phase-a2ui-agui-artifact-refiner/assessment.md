# Artifact-Refiner A2UI / AG-UI Assessment & Plan

## Context

The user asked whether to (a) submodule the artifact-refiner here, (b) add A2UI and AG-UI support, and (c) wire it into TheBoss (cherry-studio fork at `Know-Me-Tools/the-boss`) as a testing pathway, with sycophancy correction applied.

After honest review, two of the three premises were already partly handled or out of scope, and one is real, finishable work. Locked decisions (confirmed with user):

1. **Where it lands**: A2UI/AG-UI domain work goes in the **upstream `GQAdonis/artifact-refiner-skill` repo**. This skill pack only bumps the submodule pointer.
2. **TheBoss role**: **Out of scope here.** Cherry-studio currently has zero A2UI/AG-UI integration code (verified via grep). Adding it is real Electron/React work that belongs in the `the-boss` repo, not in this skill pack.
3. **Phasing**: **A2UI first, AG-UI second.** A2UI is half-built (`references/domain/a2ui.md` exists, listed as a trigger keyword, has an example scaffold). AG-UI is a new gap and event-stream-shaped — needs a spike to confirm the PMPO orchestration model fits.

---

## Honest assessment of the original premise

### What was already true
- **The submodule already exists.** `.gitmodules` has `skills/imported/artifact-refiner` → `git@github.com:GQAdonis/artifact-refiner-skill.git` pinned at `55e8625`. No new submodule work needed.
- **A2UI is already a declared domain.** `SKILL.md` lists `a2ui` as a trigger and "A2UI specifications" as a supported artifact domain. `references/domain/a2ui.md` exists (98 lines). `examples/ui-preview-refinement/` is the scaffolding.

### What was missing or wrong in the framing
- **AG-UI is not present anywhere** in the artifact-refiner. AG-UI (the agent↔frontend event-stream protocol from CopilotKit) is *not the same shape as A2UI* (a UI component spec). The artifact-refiner is well-suited to A2UI; AG-UI fit is plausible but unconfirmed.
- **Cherry-studio has no A2UI/AG-UI code today.** Every grep hit was an i18n string fragment or unrelated `*.skill.test.ts` file. "Test A2UI in TheBoss" is "first add A2UI to TheBoss" — a different project.
- **The pipeline as described conflates four artifacts**: React/HTMX prototype, A2UI spec, AG-UI event stream, native Rust agent. Each is a different protocol with different validation. Cramming them into one skill is a scope error; the artifact-refiner refines one artifact at a time, and a calling agent composes the pipeline.

### Sycophancy check
The original request used assertive framing ("we should add this as yet another submodule") that pre-decided the answer. Real check: the submodule already exists, A2UI is half-built, AG-UI is the only genuine gap, and TheBoss coupling is a smell. Pushed back on those three points; user confirmed the recommended directions.

---

## Plan

### Phase 1 — Finish A2UI (in `GQAdonis/artifact-refiner-skill` upstream)

Touch the submodule's own repo, then bump the pointer here.

Concrete additions inside the artifact-refiner repo:

- `references/schemas/a2ui-component.schema.json` — JSON Schema for an A2UI component spec. Use the canonical A2UI schema if one exists upstream; otherwise codify the structure currently described prose-only in `references/domain/a2ui.md` (component hierarchy, bindings, no circular references, no undefined bindings, naming conventions).
- `references/schemas/a2ui-manifest.schema.json` — extend `refinement-state.schema.json` with A2UI-specific fields (component_count, runtime_target, preview_artifact_path).
- `scripts/render-a2ui-preview.sh` — implements the preview promise on `references/domain/a2ui.md:28` ("Render browser preview with HTMX runtime policy controls"). Takes a normalized A2UI spec, produces an HTML preview using a small Node renderer in `scripts/lib/`, captures a screenshot via `playwright` already used elsewhere in the skill pack tooling. Self-contained: uses `npx`/`uvx` so the skill stays portable.
- `examples/a2ui-component-refinement/` — concrete example with `artifact_manifest.json`, a starter A2UI spec, and the rendered preview output. Mirror the layout of `examples/ui-preview-refinement/`.
- `prompts/specify.md` and `prompts/meta-controller.md` — add A2UI detection heuristics + content-type routing entry. Today A2UI is in the trigger list but content-type detection isn't fully wired.
- `prompts/execute.md` — register the A2UI domain adapter so the executor knows to call `render-a2ui-preview.sh` and validate against the schema.

Validation hooks (no new infrastructure — reuse existing):
- `hooks/hooks.json` already has `PostToolUse` and `SubagentStop` entries; the new `validate-constraints.sh` invocation just gets the A2UI schema fed through it.

### Phase 2 — AG-UI spike, then domain (in upstream)

**Spike before commitment** (one short doc, ~1–2 days of work):
- `references/domain/ag-ui-spike.md` — document AG-UI's actual shape (event types, run lifecycle, message envelope, tool-call protocol from `ag-ui.com`/CopilotKit). Decide: does the artifact-refiner refine *the event-stream config* (tool definitions, prompts, UI handler bindings) or *individual events*? My current read: it refines the **agent-side AG-UI spec** (which tools are exposed, which events are emitted under what conditions, which UI fragments correspond to which event types). It does *not* refine runtime event traces.

If the spike confirms fit, then add (mirroring A2UI):
- `references/domain/ag-ui.md` — domain adapter doc, same shape as `a2ui.md`.
- `references/schemas/ag-ui-spec.schema.json` — schema for the agent-side AG-UI definition.
- `examples/ag-ui-spec-refinement/` — example refining an AG-UI spec for a Rust native agent.
- Routing entries in `prompts/meta-controller.md` and `prompts/specify.md`.

If the spike says PMPO doesn't fit (e.g., AG-UI specs are too dynamic / generated-not-refined), **do not force it**. Document the negative result in the spike doc and stop. That's a real outcome, not a failure.

### Phase 3 — Pointer bump in this skill pack

Once phases 1 and 2 land tags upstream:

```
cd skills/imported/artifact-refiner
git fetch origin
git checkout v1.2.0    # whatever upstream tag is cut
cd ../../..
git add skills/imported/artifact-refiner
git commit -m "chore(submodules): bump artifact-refiner to v1.2.0 (A2UI + AG-UI domains)"
```

Plus one small change to `marketplace/marketplace.json` if the skill's tag list changes.

**That is the only change to this skill pack.** No new skill, no new submodule, no new hook. The work happens upstream and arrives via a pointer bump.

### Phase 4 (separate ticket, separate repo) — TheBoss adoption

In `Know-Me-Tools/the-boss`, not here:
- Add an A2UI renderer to the artifacts panel (renderer reads the schema produced by Phase 1).
- Optional: add an AG-UI client to subscribe to native-agent event streams (after Phase 2).
- This is a real Electron/React PR in TheBoss, gated by Phase 1/2 outputs being available as schema files.

This phase is documented here for completeness but is **not part of this assessment's deliverable.**

---

## Critical files

Inside `GQAdonis/artifact-refiner-skill` (upstream — not edits to this repo):
- New: `references/schemas/a2ui-component.schema.json`
- New: `references/schemas/a2ui-manifest.schema.json`
- New: `references/schemas/ag-ui-spec.schema.json` (Phase 2)
- New: `references/domain/ag-ui-spike.md`, then `ag-ui.md` (Phase 2)
- New: `scripts/render-a2ui-preview.sh` and `scripts/lib/`
- New: `examples/a2ui-component-refinement/`, `examples/ag-ui-spec-refinement/`
- Modified: `prompts/specify.md`, `prompts/execute.md`, `prompts/meta-controller.md`
- Modified: `SKILL.md` (mention AG-UI in trigger list and supported domains after Phase 2)

In this skill pack (only after upstream tags cut):
- Modified: `skills/imported/artifact-refiner` submodule pointer
- Modified: `marketplace/marketplace.json` (tag list, if changed)

Reused unchanged: existing `hooks/hooks.json`, `state-checkpoint.sh`, `validate-constraints.sh`, `validate-manifest.sh`.

## Verification

For Phase 1 (A2UI):
1. `cd skills/imported/artifact-refiner && bash scripts/state-init.sh demo-a2ui ui-component direct:a2ui`
2. Drop a sample A2UI spec into the artifact directory.
3. Run the refiner end-to-end; confirm `render-a2ui-preview.sh` produces an HTML file and screenshot.
4. Validate the spec against `a2ui-component.schema.json`; intentionally introduce a circular reference and confirm the constraint check fails with a useful error.
5. From the skill-pack root: `npm run validate:skill skills/imported/artifact-refiner` passes.

For Phase 2 (AG-UI spike):
1. Read the spike doc, confirm or reject PMPO fit.
2. If accepted: same flow as Phase 1 but for an AG-UI spec example.

For Phase 3 (pointer bump):
1. `git submodule status skills/imported/artifact-refiner` shows the new SHA.
2. `npm run validate` passes (lenient mode for imported skills).
3. `npm run build` rebuilds `.claude-plugin/` symlinks cleanly.
4. In Claude Code: `/refine-a2ui some-component` triggers the new domain path.

## Locked decisions (confirmed with user)

- **Land in upstream repo only**, not as new sibling skills here.
- **TheBoss work is a separate ticket** in the `the-boss` repo; skill pack is not coupled to it.
- **A2UI first, AG-UI second**, with a spike before AG-UI commitment.
- **No new submodule** for cherry-studio. The artifact-refiner submodule already exists at `GQAdonis/artifact-refiner-skill.git` (pinned `55e8625`); the only change to this repo is a future pointer bump.

## What this plan deliberately does *not* do

- Does not add a `ui-component-pipeline` skill that orchestrates React→A2UI→AG-UI. Pipelining is a calling-agent concern; the refiner is a primitive.
- Does not make the refiner emit AG-UI events at runtime. AG-UI is a runtime protocol; the refiner refines the *spec*, not the live stream.
- Does not couple any preview rendering to TheBoss specifically. Preview output is a plain HTML file + screenshot, openable anywhere.
- Does not bump the submodule pointer or land any code in this repo until upstream Phase 1 is tagged. This is a scoping document, not an implementation order.
