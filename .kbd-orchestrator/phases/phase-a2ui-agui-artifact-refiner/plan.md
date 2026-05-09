# Phase Plan — phase-a2ui-agui-artifact-refiner

**Phase goal**: deliver finished A2UI domain support and a scoped AG-UI domain extension to `GQAdonis/artifact-refiner-skill`, then bump the submodule pointer in `prometheus-skill-pack`. No coupling to TheBoss; no new submodule for cherry-studio.

**Source assessment**: `.kbd-orchestrator/phases/phase-a2ui-agui-artifact-refiner/assessment.md` (mirrored from `~/.claude/plans/a2ui-agui-artifact-refiner.md`).

**Change backend**: split.
- Changes 1 and 2 happen inside the submodule (`skills/imported/artifact-refiner/`) which uses **OpenSpec** (`openspec/changes/`). They are emitted as OpenSpec proposals there.
- Change 3 happens in this repo and uses **native-kbd** (`.kbd-orchestrator/changes/`).

**Evolver bridge**: none. This phase is not driven by an iterative-evolver cycle.

---

## Ordered change list

### change-001 — finish-a2ui-domain `[OpenSpec, upstream artifact-refiner]`

**Status**: pending
**Recommended agent**: `architect` for schema design, then `tdd-guide` + `code-reviewer` for the render script and example
**Backend**: OpenSpec inside `skills/imported/artifact-refiner/openspec/changes/finish-a2ui-domain/`
**Repo**: `GQAdonis/artifact-refiner-skill`

Tasks:

- [ ] Author `references/schemas/a2ui-component.schema.json` codifying the structural rules currently described prose-only in `references/domain/a2ui.md` (component hierarchy, bindings, no circular refs, naming conventions).
- [ ] Author `references/schemas/a2ui-manifest.schema.json` extending `refinement-state.schema.json` with A2UI-specific fields (`component_count`, `runtime_target`, `preview_artifact_path`).
- [ ] Implement `scripts/render-a2ui-preview.sh` + `scripts/lib/` (Node renderer via `npx`, screenshot via `playwright`) — fulfills the preview promise on `references/domain/a2ui.md:28`.
- [ ] Add `examples/a2ui-component-refinement/` with `artifact_manifest.json`, starter spec, and rendered preview output. Mirror layout of `examples/ui-preview-refinement/`.
- [ ] Wire A2UI detection into `prompts/specify.md` (Content Type Detection) and `prompts/meta-controller.md` (Content Type Routing).
- [ ] Wire A2UI execution path into `prompts/execute.md` so the executor calls `render-a2ui-preview.sh` and validates against the new schema.
- [ ] Update `SKILL.md` "Supported Artifact Domains" entry for A2UI to reference the new schema and preview path.
- [ ] Verify: `bash scripts/state-init.sh demo-a2ui ui-component direct:a2ui` initializes; refiner runs end-to-end; intentional circular reference fails the constraint check with a useful message.
- [ ] Cut upstream tag (e.g., `v1.2.0`).

Exit criterion: A2UI refinement runs end-to-end on a demo spec and produces a normalized spec, validation report, HTML preview, and screenshot.

### change-002 — agui-spike-and-domain `[OpenSpec, upstream artifact-refiner]`

**Status**: pending — gated on change-001
**Recommended agent**: `planner` for the spike, then `architect` + `tdd-guide` if the spike confirms fit
**Backend**: OpenSpec inside `skills/imported/artifact-refiner/openspec/changes/agui-spike-and-domain/`
**Repo**: `GQAdonis/artifact-refiner-skill`

Spike tasks (commit before any further AG-UI work):

- [ ] Author `references/domain/ag-ui-spike.md` covering: AG-UI event types, run lifecycle, message envelope, tool-call protocol (per `ag-ui.com` / CopilotKit). State explicitly which artifact is being refined (current hypothesis: the agent-side AG-UI spec — exposed tools, event-emission rules, UI-fragment bindings — *not* runtime event traces).
- [ ] Decision: does PMPO fit? Document answer in the same file. **If no, stop here.** Negative result is acceptable.

Domain tasks (only if spike says yes):

- [ ] Author `references/domain/ag-ui.md` mirroring the shape of `a2ui.md`.
- [ ] Author `references/schemas/ag-ui-spec.schema.json`.
- [ ] Add `examples/ag-ui-spec-refinement/` with a refining-an-AG-UI-spec-for-a-Rust-native-agent example.
- [ ] Routing entries in `prompts/meta-controller.md` and `prompts/specify.md`.
- [ ] Update `SKILL.md` triggers and supported domains.
- [ ] Verify: same flow as change-001, applied to an AG-UI spec example.
- [ ] Cut upstream tag (e.g., `v1.3.0`).

Exit criterion: either a working AG-UI domain refinement OR a documented "PMPO does not fit" decision with reasoning. Both are valid.

### change-003 — bump-artifact-refiner-pointer `[native-kbd, this repo]`

**Status**: pending — gated on change-001 (and change-002 if it ships)
**Recommended agent**: `code-reviewer` for the pointer-bump diff
**Backend**: native-kbd inside `.kbd-orchestrator/changes/change-003-bump-artifact-refiner-pointer/`
**Repo**: this repo (`prometheus-skill-pack`)

Tasks:

- [ ] `cd skills/imported/artifact-refiner && git fetch origin && git checkout <new-tag>`
- [ ] `cd ../../.. && git add skills/imported/artifact-refiner`
- [ ] If `marketplace/marketplace.json` lists per-skill tags or domain entries, refresh them.
- [ ] Run `npm run validate` (lenient mode covers imported skills).
- [ ] Run `npm run build` and confirm `.claude-plugin/` symlinks rebuild cleanly.
- [ ] Smoke test in Claude Code: confirm new A2UI detection path triggers; if AG-UI shipped, confirm AG-UI path triggers.
- [ ] Commit: `chore(submodules): bump artifact-refiner to <tag> (A2UI + AG-UI domains)`.

Exit criterion: skill pack ships pointer to upstream tag, all validation passes, end-to-end skill invocation works in Claude Code.

---

## Out of scope (explicitly)

- Phase 4 / TheBoss adoption — separate ticket in `Know-Me-Tools/the-boss`. Documented in the assessment for reference only.
- `ui-component-pipeline` orchestrator skill — pipelining is a calling-agent concern.
- Any submodule for cherry-studio.
- Any change that makes the refiner emit live AG-UI events (it refines specs, not streams).

---

## Risks and gates

| Risk | Mitigation |
|---|---|
| AG-UI spike concludes PMPO doesn't fit | Acceptable outcome. change-002 ships only the spike doc; change-003 still proceeds with A2UI-only bump. |
| Upstream A2UI canonical schema differs from our codified one | Before authoring the schema, search for an upstream A2UI JSON Schema; use it if it exists, otherwise codify and note the divergence in the spec doc. |
| Render script picks up a heavy dependency | Constrain to `npx`/`uvx` package runners; no committed `node_modules`. Existing `playwright` use in skill pack tooling is the precedent. |
| Submodule pointer bump breaks downstream consumers | Lenient `npm run validate` is the gate; strict mode does not run on `skills/imported/`. |

## Verification trail (per phase)

1. After change-001: tag exists upstream, examples dir is populated, screenshot file present.
2. After change-002: either AG-UI example renders, or `ag-ui-spike.md` documents the negative decision.
3. After change-003: `git submodule status skills/imported/artifact-refiner` shows the new SHA, `npm run validate` passes, `/refine-a2ui demo` works in Claude Code.
