# Reflection — pmpo-evolver

**Phase:** pmpo-evolver
**Reflected:** 2026-06-28
**Changes:** 10 / 10 complete
**Backend:** OpenSpec

---

## Goal Achievement

| Goal | Status | Evidence |
|------|--------|----------|
| G1 — Ship `skills/process/pmpo-evolver/SKILL.md` (strategy router) | **MET** | `SKILL.md` created (191 lines); defines `/pmpo-evolver` and `/pmpo-evolver-status` entry commands; five perspectives documented with routing table; liter-llm model routing directives throughout |
| G2 — Define evolver schema (`pmpo-evolver.schema.json`) | **MET** | `references/schemas/pmpo-evolver.schema.json` created with all required fields: perspective, perspective_cursor, competitor_tracking, learning_signals[], idea_origin, evolver_lessons[], model_routing |
| G3 — Wire evolver into pmpo-outer-loop (perspective handoff) | **MET** | `loop-tick.sh` updated with `--perspective <value>` flag; `loop-definition.schema.json` has root `perspective` field; pmpo-outer-loop SKILL.md has perspective routing paragraph |
| G4 — Inner-loop bridge: evolver items → KBD phases | **MET** | `evolver-seed-phase.sh` creates goals.md + progress.json + evolver-bridge.json from archive manifest; updates current-waypoint.json |
| G5 — State persistence (learning signals, resumable cursor) | **MET** | `pmpo-evolver.schema.json` defines learning_signals[] with full field set; `evolution-state.schema.json` extended with learning_signals[] and perspective; feedback-digest.sh writes per-tick archives |
| G6 — Platform parity | **MET** | `skills/process/pmpo-evolver/` auto-discovered by `install-skills-flat.sh`; all scripts use bash+python3; no platform-specific behavior; context-management.md documents harness-native fallback for liter-llm |

**Overall: 6/6 goals MET. All 5 success criteria satisfied.**

---

## Delta — Planned vs. Delivered

### Planned exactly as specified
- `pmpo-evolver.schema.json` — all fields per plan (perspective, perspective_cursor, competitor_tracking, learning_signals, idea_origin, evolver_lessons, model_routing)
- `evolution-state.schema.json` extensions — additive only, backward-compatible
- `loop-definition.schema.json` extended with 6 new feedback_source types + staleness_ttl_minutes + root perspective field
- `SKILL.md` (191 lines, under 500 limit) + `model-routing.md` (16-row class assignment table)
- `competitive-analysis.md` + `competitor-registry-init.sh` + `changelog-fetch.sh`
- `learning-signals.md` + `commit-history-analyze.sh` + `feedback-digest.sh`
- `validate-idea/SKILL.md` (three-gate Darwin pipeline) + `idea-gate-1.sh`
- `domain-taxonomy.md` (6 domain clusters) + `carry-forward-aggregate.sh`
- `strategic-dreaming.md` + `post-cycle-dream.sh`
- `loop-tick.sh` perspective extraction + pmpo-outer-loop SKILL.md perspective paragraph
- `evolver-seed-phase.sh` + `context-management.md` + `liter-llm-bridge/references/model-discovery.md`

### Adaptations from plan
- **`commit-history-analyze.sh` implementation method:** Plan called for passing git log to liter-llm `complete(model=small)`. The `small` class LLM is only needed for ambiguous cases; conventional commit patterns (regex) are sufficient for classification. Implemented with python3 regex classification, which is deterministic, free, and faster. LLM fallback can be added in a future cycle. Model routing directive preserved: `[MODEL_ROUTING] phase=evolver-signal-commits class=small`.
- **`loop-definition.schema.json` root `additionalProperties: false` constraint:** The schema had a root `additionalProperties: false` that would have blocked new fields. Resolution: added `perspective` explicitly to the properties block before the constraint. `feedback_sources` items did not have `additionalProperties: false` so new type-specific fields were added directly without conflict.
- **`feedback-digest.sh` Python execution method:** Initial plan embedded git log directly into a Python heredoc. Fixed by piping via stdin. This pattern is now documented in the script itself.
- **`validate-idea/SKILL.md` gate model directives:** Gate 1 uses `small` as planned. Gate 2 uses `medium` as planned. Gate 3 uses `frontier` as planned. Archive format matches the plan exactly.
- **`carry-forward-aggregate.sh` smoke test:** Ran against 22 existing reflection.md files and found 4 carry-forward items — all from pmpo-elicit. Confirms the script works against real repo history.

### Gaps from assessment that were deprioritized
- **G-06: Domain taxonomy — standards-body mapping:** Plan called for mapping to standards bodies (IETF, W3C, NIST, ISO). Delivered as domain cluster mapping with detection queries and polling TTLs. Standards-body enumeration within each cluster is operator-specific and would require research per project; documented as the operator's responsibility via the cluster structure.
- **No `mcp-tool` feedback source type in loop-tick.sh:** The schema extension added `mcp-tool` as a valid type in loop-definition.schema.json. The `loop-tick.sh` implementation does not yet handle `mcp-tool` at runtime (it falls to the `unknown type WARN` branch). Added a `feedback-digest.sh` handler stub for non-implemented types. Full MCP tool dispatch in loop-tick.sh is a carry-forward.
- **Idea-to-spec bridge with `zeespec-interrogator`:** Plan mentioned zeespec-interrogator integration. Delivered as a standalone SPEC.md format in validate-idea. zeespec-interrogator integration is a carry-forward (would require verifying zeespec-interrogator still exists and its current interface).
- **Platform-specific `references/platforms/` directory:** Assessment noted this as LOW priority for G6. Not implemented — `context-management.md` documents harness-native fallback pattern instead, which covers platform differences adequately.

---

## Artifact Quality Summary

No artifact-refiner QA logs present for this phase (`.refiner/artifacts/change-evolver-NNN/` directories absent). QA performed manually via `npm run validate:strict` at each change.

| Metric | Value |
|--------|-------|
| Changes total | 10 |
| Changes passing `npm run validate:strict` | 10 / 10 |
| Skills created with validate:strict | 2 (pmpo-evolver, validate-idea) |
| Validation errors | 0 |
| Validation warnings | 0 (pre-existing `kbd-process-orchestrator` 548-line warning unrelated) |
| Scripts made executable | 8 (all new scripts) |
| Schema files validated as valid JSON | 3 (pmpo-evolver.schema.json, evolution-state.schema.json, loop-definition.schema.json) |

---

## Technical Debt Introduced

1. **`loop-tick.sh` does not dispatch `mcp-tool` feedback sources at runtime.** Schema supports the type; the runtime handler is a stub (`WARN: unknown feedback source type`). Operator can still define `mcp-tool` sources in loop.json — they will not be evaluated until this is implemented.

2. **`commit-history-analyze.sh` hotspot detection is empty.** The script correctly classifies commits by type but returns `hotspots: []` always. Hotspot detection (files most frequently changed in fix commits) requires `git log --name-only` parsing — a more complex script. Output is valid JSON; just the hotspot array is empty.

3. **`post-cycle-dream.sh` graceful degradation on missing liter-llm.** When liter-llm is absent, the script exits 0 with `lessons_added: 0`. This means a cycle without liter-llm produces no strategic lessons. Acceptable fallback, but operators running without liter-llm will need to run dreaming manually or use the host model inline.

4. **`validate-idea` sub-skill Gate 2 and Gate 3 are prose-only.** The SKILL.md defines the protocol for Gate 2 (domain research) and Gate 3 (spec + human gate) but does not have corresponding shell scripts (unlike Gate 1 which has `idea-gate-1.sh`). These gates are intended for the host model to execute inline following the skill instructions. A future cycle could extract them into `idea-gate-2.sh` and `idea-gate-3.sh`.

---

## Lessons

1. **Schema `additionalProperties: false` at the wrong scope blocks evolution.** The root-level `additionalProperties: false` in `loop-definition.schema.json` would have silently broken any extension if not caught. Lesson: check for this constraint when extending any existing schema before writing new fields.

2. **Subprocess isolation for data-heavy scripts was the right architecture.** Every script that could produce large outputs (git log, gh issues, changelog content) reads only what it needs and outputs compact JSON. The evolver session never holds raw source content. This pattern should be the default for all future evolver scripts.

3. **`carry-forward-aggregate.sh` revealed real repo state.** Running it against 22 existing reflections found that most phases had no `## Carry-Forwards` section. Only pmpo-elicit had 4. This signals a gap in the KBD Reflect discipline — phases should produce Carry-Forwards sections consistently. The script is a diagnostic tool as much as an aggregator.

4. **Darwin Gödel Machine's three-gate pattern is directly implementable.** The staged evaluation (fast/cheap Gate 1 → medium research Gate 2 → deep spec Gate 3) maps perfectly to `small|medium|frontier` liter-llm class assignments. The pattern is not just conceptual — it determines the cost model directly.

5. **Strategic dreaming requires a distinct prompt from PMPO Reflect.** The distinction (product direction vs. execution quality) is clear in theory but easy to collapse in practice. The `post-cycle-dream.sh` prompt explicitly prevents this by instructing the model to focus on "what did we learn about where the product should go next" — not "did we execute well."

---

## Carry-Forwards

- **`mcp-tool` feedback source runtime dispatch in `loop-tick.sh`** — Schema defined; handler not implemented. Requires: MCP tool name + arguments dispatch pattern in bash. Estimated: 1 change, S effort.
- **`commit-history-analyze.sh` hotspot detection** — `hotspots: []` always. Requires: `git log --name-only` parsing + frequency count. Estimated: 1 change, XS effort.
- **Gate 2 and Gate 3 scripts for `validate-idea`** — Currently prose-only in SKILL.md. Extracting to `idea-gate-2.sh` and `idea-gate-3.sh` would make them callable from non-Claude-Code platforms. Estimated: 1 change, S effort.
- **`zeespec-interrogator` integration in validate-idea Gate 3** — Validate whether zeespec-interrogator is still available and its current interface. If so, pipe the Gate 3 SPEC.md through it for automated spec quality scoring. Estimated: 1 change, M effort.
- **Carry-Forwards discipline in KBD Reflect** — Most phases lack `## Carry-Forwards` sections. The KBD Reflect skill prompt should explicitly require this section with at least one entry (or a deliberate "none" marker). Estimated: 1 prose change to kbd-reflect SKILL.md, XS effort.
- **`feedback-digest.sh` handlers for `sentiment-feed`, `telemetry-url`, `competitor-scan`, `changelog`** — Currently only `commit-history` and `gh-issues` have runtime handlers. Four types have stubs. Estimated: 4 handlers, 1 change, M effort.

---

## Recommended Next Phase

**`/kbd-reflect pmpo-evolver`** is now complete.

**Recommended next:** `/pmpo-evolver prometheus-skill-pack --perspective competitive` — run the first live evolver cycle against this repository, using the competitive perspective to validate the entire system end-to-end. This would exercise competitor-registry-init.sh, changelog-fetch.sh, feedback-digest.sh, and the strategy routing logic all in one real cycle.

**Alternative:** Address the highest-priority carry-forward — `mcp-tool` runtime dispatch + the four missing `feedback-digest.sh` handlers — as a short follow-up phase before running the first live cycle.

---

## Evolver Bridge

No evolver-bridge.json exists for this phase (pmpo-evolver was itself an evolver meta-phase, not driven by a higher-level evolution cycle). No execution results to write back.
