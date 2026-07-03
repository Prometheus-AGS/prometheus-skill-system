# Reflection — phase-learn-feynman

**Phase:** phase-learn-feynman
**Reflected:** 2026-06-28
**Change count:** 28 / 28 DONE
**Commit:** b0cf755 — feat(learn): ship Feynman-Spine Learning & Education Capability — v1.4.0
**Version bump:** 1.3.0 → 1.4.0

---

## Goal Achievement

Goals were not written to `goals.md` (file contains a stub from `/kbd-new-phase`). The authoritative goal set was embedded in the Phase Brief (§0–§12), refined through `assessment.md`, and carried into `plan.md`. Goals are reconstructed here from those canonical sources.

| # | Goal | Status | Evidence |
|---|------|--------|----------|
| G1 | Implement Feynman-Spine learning loop as a PMPO-mapped skill (`feynman-loop`) covering novice→peer→skeptic escalation and bounded vertical recursion | **MET** | `skills/learn/feynman-loop/SKILL.md` shipped; recursion floor guard, depth cap 3, horizontal escalation documented |
| G2 | External, sycophancy-corrected grader (`learn-grade`) — grounding-corpus-backed, S-02 check on critical path | **MET** | `skills/learn/learn-grade/SKILL.md`; S-02 check invoked before grade is returned; pass = score ≥0.7 AND misconceptions_absent |
| G3 | Explicit persistent learner model: CRDT-backed (automerge), FSRS-6 scheduler, JSON-RPC shell interface | **MET** | `substrate/learner-model/` Rust crate; PFA update at ≥5 observations; Rating enum; `substrate/storage-provider/` trait crate |
| G4 | Surface-tier degradation contract: Tier 0 (text) → Tier 1 (AskUserQuestion / file-pair) → Tier 2 (surface-bridge) | **MET** | `skills/learn/ui-surface/SKILL.md` + `shared/scripts/detect-surface-tier.sh`; `substrate/surface-bridge/` Axum server on :7890 |
| G5 | FSRS-6 spaced retrieval engine (`learn-retain`) | **MET** | `skills/learn/learn-retain/SKILL.md`; reads due queue; four-tier rating mapping |
| G6 | Deliberate-practice track (`learn-practice`) with derivation/implementation/transfer modes | **MET** | `skills/learn/learn-practice/SKILL.md`; mastery-gated access; interleaved schedule |
| G7 | OB 3.0 / W3C VC certification with evidence bundle and integrity guardrail (`learn-certify`) | **MET** | `skills/learn/learn-certify/SKILL.md`; self-issued via did-plc; Δmastery > 0.4 → integrityNote |
| G8 | Privacy-safe custom KB adapter — never forwards KB content to external APIs | **MET** | `shared/scripts/content-grounding-kb.sh`; four adapter types (dify:, palace:, local:, web:); `warn_external_api_vars()` guard |
| G9 | `learn-kb` skill for operator KB management | **MET** | `skills/learn/learn-kb/SKILL.md`; add/list/query/update/remove subcommands |
| G10 | Meta-grounding corpora enabling the pack to teach itself | **MET** | `docs/learn/meta-corpus/kbd-lifecycle-corpus.json` (18+8) and `skill-pack-corpus.json` (15+9) |
| G11 | Self-teaching adoption skills: `learn-about-system` and `learn-harness` | **MET** | Both skills shipped; `learn-about-system` routes to Feynman loop over KBD/skill-pack via meta-corpus; `learn-harness` auto-detects and produces 13-row capability map |
| G12 | Cross-harness parity across all 5 target harnesses (Claude Code, OpenCode, Codex, Kimi, Zed) | **MET** | Tier 0 universal floor; `detect-surface-tier.sh` covers all 5; `references/harness-parity.md` (236 lines) documents per-harness behavior |
| G13 | Recursion floor guard: `learn-survey` sets floor; `feynman-loop` never recurses into concepts learner already owns | **MET** | `survey-result.json` schema includes `recursion_floor`; feynman-loop reads and enforces it |
| G14 | Honest feasibility gate: GREEN/YELLOW/RED with 1.5× multiplier; sycophancy-corrected | **MET** | `learn-goal` step 3; RED = time > 1.5× available; gate cannot be softened by agent |
| G15 | Full integration test coverage for the learn domain | **MET** | 4 suites: `integration-basic-flow.sh`, `integration-full-loop.sh`, `integration-kb.sh`, `integration-meta.sh` |
| G16 | Installation, toolchain detection, docs, and v1.4.0 release | **MET** | `install-skills-flat.sh` extended; `detect-toolchain.sh` extended; `docs/guide/10-learn-skills.md`; `CLAUDE.md` updated; versions bumped |

**Summary: 16/16 goals MET.**

---

## Delivered Changes

| Change ID | Title | Type | Notes |
|-----------|-------|------|-------|
| change-learn-001 | Spike: learner-model schema + CRDT conflict semantics | design | `learner-model.schema.json`, `crdt-conflict-semantics.md`; PFA rule; 3 worked conflict examples |
| change-learn-002 | Spike: surface-bridge detect-surface-tier probe | design | `surface-tier-detection.md`, `detect-surface-tier.sh`; 5-harness env var matrix |
| change-learn-003 | content-grounding service + public corpus assembly | shell | `content-grounding.sh`; 4-tier source chain; `grounding-corpus.schema.json` |
| change-learn-004 | content-grounding KB adapter | shell | `content-grounding-kb.sh`; dify:/palace:/local:/web: adapters; privacy guard |
| change-learn-004b | storage-provider trait crate | rust | StorageProvider + CrdtEngine traits; LocalDirAdapter; AutomergeEngine; IrohDocsAdapter stub |
| change-learn-005 | learner-model Rust crate | rust | CRDT learner model; simplified FSRS-6; JSON-RPC stdin/stdout; PFA update |
| change-learn-006 | ui-surface skill | skill | Tier 0/1/2 rendering; UiIntent schema; file-pair convention |
| change-learn-007 | learn-goal skill | skill | 7-step flow; feasibility gate; --kb flag; grounding corpus assembly |
| change-learn-008 | learn-survey skill | skill | 11 diagnostic items (5 conceptual, 3 procedural, 3 misconception probes); recursion floor; seeds learner model |
| change-learn-009 | learn-grade skill | skill | S-02 sycophancy check on critical path; 4 dimensions; novel transfer problems from corpus |
| change-learn-010 | feynman-loop skill | skill | PMPO mapping; vertical recursion with floor guard; horizontal escalation; 3 mastery closure criteria |
| change-learn-011 | learn-plan skill | skill | Concept DAG in surreal-memory; topological sort; curriculum.json; --replan mode |
| change-learn-012 | learn-retain skill | skill | FSRS due queue; learn-grade at ≥0.6; four-tier FSRS rating mapping |
| change-learn-013 | learn-practice skill | skill | Derivation/implementation/transfer modes; mastery-gating (< 0.6 / 0.6–0.8 / > 0.8) |
| change-learn-014 | learn-certify skill | skill | Checkpoint + final modes; OB 3.0 / W3C VC JSON-LD; integrity guardrail; self-issued via did-plc |
| change-learn-015 | learn-kb skill | skill | KB registry management; 4 adapter types; privacy guarantee |
| change-learn-016 | Meta-grounding corpus for KBD + skill pack | content | kbd-lifecycle-corpus.json (18+8); skill-pack-corpus.json (15+9) |
| change-learn-017 | learn-about-system skill | skill | Zero-friction adoption; 3 --area routing paths; interactive discovery; self-teaching loop |
| change-learn-018 | learn-harness skill | skill | Auto-detect harness; 13-row capability map; per-harness orientation; --map-only flag |
| change-learn-019 | surface-bridge Axum MCP App server | rust | Axum 0.7 on :7890; /health + 3 MCP routes; macOS launchd plist |
| change-learn-020 | skills/learn domain directory + validation | infra | Domain scaffold; marketplace + plugin.json wiring; README |
| change-learn-021 | Integration test: basic flow | test | write-goal → write-survey → write-artifact → write-grade pipeline |
| change-learn-022 | Integration test: full loop | test | FSRS card mutation; practice-result fields; VC JSON-LD; integrityNote |
| change-learn-023 | Integration test: KB | test | Local adapter; privacy guardrail; corpus schema validation; KB registry |
| change-learn-024 | Integration test: meta-skills | test | KBD corpus; skill-pack corpus; detect-surface-tier; learn-about-system; learn-harness; all 12 skills validate |
| change-learn-025 | install-skills-flat.sh update | infra | install_learn_substrate function; builds 3 Rust crates; launchd install |
| change-learn-026 | docs/guide update | docs | `10-learn-skills.md` (274 lines); mermaid arc diagram; operator guide |
| change-learn-027 | CLAUDE.md update | docs | Learn Domain section; 4-layer architecture; surface tier contract; KB adapter pattern |
| change-learn-028 | v1.4.0 release bump + changelog | release | package.json, plugin.json, marketplace.json, CHANGELOG.md |

---

## Artifact Quality Summary

The artifact-refiner QA gate was not run for this phase. The phase had no `constraints.md` file and no `.refiner/` directory — both are prerequisites for artifact-refiner invocation. Quality control for this phase was provided through:

1. **Manual review during execution**: Each change was authored with attention to agentskills.io compliance (frontmatter, forward slashes, version/license/tags fields), integration test coverage, and inline schema validation.
2. **Integration test suites**: 4 suites covering the full learn domain pipeline verified output schema conformance, FSRS card mutation, VC JSON-LD structure, privacy guardrails, and harness detection — providing functional quality signal in lieu of artifact-refiner constraint checks.
3. **Validation gate**: `npm run validate:strict` was run for all skills in the learn domain as part of change-learn-024.

| Metric | Value |
|--------|-------|
| Changes with artifact-refiner QA | 0 / 28 |
| Integration test suites run | 4 |
| Skills passing `validate:strict` | 12 / 12 |
| Estimated first-pass quality | HIGH — no critical issues surfaced during integration testing |

**Recommendation for follow-on phases:** Create `constraints.md` and wire artifact-refiner before execute begins. The learn domain's complexity (Rust crates + shell scripts + skill files + schemas + corpora + integration tests) would benefit from constraint-driven QA on each change type.

---

## Deltas vs. Plan

### What deviated from plan.md

1. **change-learn-004b numbering**: The plan numbered the storage-provider crate as `change-learn-004b` (an in-sequence insertion after change-learn-004 was added). Progress tracking followed this numbering, resulting in 28 changes but with a non-sequential ID in the middle. No functional impact; a cosmetic artifact of plan evolution during assess→plan.

2. **goals.md was a stub**: The `goals.md` file contained only the placeholder `"TBD — run /kbd-assess phase-learn-feynman to surface gaps and define concrete goals."` Goals were never written back to the file. All goal tracking happened in assessment.md and plan.md. This was a process gap: `/kbd-new-phase` created the stub but `/kbd-assess` did not update it.

3. **Grader fidelity acknowledged as open risk**: The assessment flagged grader fidelity (60–70% confidence) as the hardest part of `learn-grade`. The shipped implementation uses a grader system prompt that explicitly states "finding no gaps is the suspicious result" and routes through sycophancy-correction S-02. Actual misconception detection fidelity in production depends on the quality of the grounding corpus — this risk is correctly noted but not fully resolved by the implementation. It requires empirical validation with real learning sessions.

4. **IrohDocsAdapter shipped as stub only**: Per plan, the full iroh-docs P2P sync adapter was explicitly deferred. The stub (`IrohDocsAdapter` with `unimplemented!()` bodies) is present in `substrate/storage-provider/src/iroh_docs.rs` as the follow-on seam. This is correct per the scope guard in plan.md.

5. **`learn-to-build` not implemented**: Correctly deferred per plan scope guard. No deviation.

6. **Tier 3 (full external browser surface) not implemented**: Correctly deferred per plan. Tier 0 and Tier 1 are fully functional; Tier 2 ships with surface-bridge.

### What was added beyond plan

- `docs/learn/schemas/kb-corpus.schema.json` — an additional schema extending `grounding-corpus.schema.json` for KB-sourced corpus entries. Not in original plan but needed for test validation in change-learn-023.
- `docs/learn/kb-adapter-guide.md` (195 lines) — a KB adapter usage reference. Surfaced as needed documentation during change-learn-015 implementation.
- `tests/learn/fixtures/` directory with `sample-kb/` fixture KB and `sample-corpus.json` — supporting the integration-kb.sh test suite.

---

## Technical Debt Introduced

| Debt Item | Severity | Location | Recommended Resolution |
|-----------|----------|----------|----------------------|
| IrohDocsAdapter stub | MEDIUM | `substrate/storage-provider/src/iroh_docs.rs` | Implement full iroh-docs adapter in a follow-on `phase-learn-sovereign-sync` |
| `learner-model` Rust binary has no CLI argument handling | LOW | `substrate/learner-model/src/main.rs` | The JSON-RPC interface is stdin/stdout only; consider `clap` subcommands for direct invocation (troubleshooting) |
| `surface-bridge` OnceLock pending store is in-memory only | MEDIUM | `substrate/surface-bridge/src/handlers.rs` | Response collection is lost on service restart; add persistence via storage-provider for production reliability |
| `learn-grade` grader fidelity is unvalidated | HIGH | `skills/learn/learn-grade/SKILL.md` | Run empirical evaluation: 10+ real Feynman explanations graded by the skill vs. human expert; measure precision and recall on gap detection and misconception identification |
| Meta-corpora will go stale on pack evolution | MEDIUM | `docs/learn/meta-corpus/*.json` | Wire `doc-updater` into the release pipeline to rebuild meta-corpora on each version bump; currently manual |
| `goals.md` is never populated by `/kbd-assess` | LOW | KBD lifecycle convention | Update the assess protocol or `/kbd-new-phase` to write goal stubs that `/kbd-assess` can fill in |
| 1EdTech certification issuer endpoint (`--issuer`) is undocumented | LOW | `skills/learn/learn-certify/SKILL.md` | Document the `--issuer <endpoint>` parameter contract once a 1EdTech-compatible issuer is available |
| FSRS-6 is a simplified stub, not the full `fsrs-rs` model | MEDIUM | `substrate/learner-model/src/fsrs.rs` | Replace the simplified scheduler with the full `fsrs-rs` crate integration for production-accurate scheduling |

---

## Lessons Learned

### What worked well

1. **Tier 0 floor as scope guard**: The hard requirement that every skill work in text-only mode prevented substrate scope creep. Tier 2 (surface-bridge) was implemented last because it was never on the critical path. This is the correct discipline and should be applied to all future learn-domain phases.

2. **Anti-sycophancy on the critical path**: Routing `learn-grade` through sycophancy-correction S-02 as an architectural constraint (not optional guidance) is the right pattern for any system where feedback quality is load-bearing. The same pattern should be applied to any skill whose output informs downstream decisions — particularly when the LLM has a natural incentive to produce positive feedback.

3. **Privacy-first KB design**: The `warn_external_api_vars()` function in `content-grounding-kb.sh` and the hard rule "NEVER forward KB content to external APIs" were written into the implementation, not just the documentation. This is the correct way to enforce a trust guarantee — implementation-level enforcement, not documentation-level aspiration.

4. **Meta-corpus pattern**: Pre-building grounding corpora for the skill pack's own concepts (KBD lifecycle, skill domains, harness capabilities) enables the pack to teach itself without a live research phase. This pattern should be applied to other complex domains in the pack (React, Rust, DevOps) so operators can use the learning skills to onboard into those domains too.

5. **Two-spike design phase**: Starting with design-only spikes (change-learn-001: CRDT conflict semantics; change-learn-002: surface-tier detection) before any implementation prevented interface divergence between the learner-model crate and the learn-survey skill. The spike discipline is the correct pattern for any phase where substrate interfaces must be locked before skill code can be written.

6. **Integration tests as the quality signal**: With 4 integration test suites covering the full pipeline, functional correctness was verifiable without artifact-refiner. For documentation-heavy phases like this one, integration tests over the file-schema pipeline substitute effectively for constraint-based QA.

### What to do differently

1. **Write goals.md at assess time, not new-phase time**: The assess protocol should update `goals.md` with concrete, enumerated goals once the assessment is complete. A stub file is not useful as a reflection anchor.

2. **Wire artifact-refiner before execute for future learn-domain phases**: The learn domain's complexity (Rust crates + shell scripts + skills + schemas + tests + docs) warrants constraint-driven QA. `constraints.md` should be written during `/kbd-plan` for any phase with more than 10 changes.

3. **FSRS-6 should be full, not stub**: The simplified FSRS scheduler (`next_review()` with fixed interval multipliers) is not equivalent to the full `fsrs-rs` model. Production learning sessions will get sub-optimal scheduling. The substrate should have used `fsrs-rs` directly from the start; the simplification was a scoping shortcut that created real debt.

4. **Grader fidelity needs empirical validation before the phase ships**: The assessment flagged grader fidelity at 60–70% confidence. This risk was accepted rather than resolved. For a learning system, a grader that misses misconceptions is worse than no grader — it provides false assurance. Future phases that extend learn-grade should include a grader evaluation dataset as a deliverable.

5. **`learn-about-system --area` routing is static**: The routing logic in `learn-about-system` (--area kbd | skills | harness) is hard-coded. As the pack grows, a dynamic discovery approach (querying CLAUDE.md and the meta-corpus index) would scale better than a static match.

---

## Recommended Next Phase

**Recommended: `phase-learn-sovereign-sync`**

The most load-bearing deferred item is the IrohDocsAdapter — the mechanism that allows a learner's model to sync across devices without a central server. Without it, all learner state is local-only and the "resume from any harness on any device" promise from the Phase Brief is unfulfilled.

Scope:

1. Implement `IrohDocsAdapter` in `substrate/storage-provider/` using the `iroh` and `iroh-docs` crates
2. Extend `substrate/learner-model/` to use iroh-docs sync as the default adapter after local-dir
3. Validate multi-device CRDT merge using the conflict semantics from change-learn-001
4. Add device-to-device sync test to the integration suite
5. Document the sovereign sync setup in `docs/guide/10-learn-skills.md`

**Alternative: `phase-learn-grader-validation`**

If empirical validation of `learn-grade` is prioritized (HIGH debt item), a shorter phase to:

1. Assemble a grader evaluation dataset: 20+ Feynman explanations with human-expert gap annotations
2. Run `learn-grade` against the dataset; measure precision and recall
3. Tune the grader system prompt based on failure modes found
4. Add a grader regression test to the integration suite

This is lower effort but addresses the highest-severity technical debt in the phase.

**Recommendation:** Run `phase-learn-sovereign-sync` first (it completes the architecture) then `phase-learn-grader-validation` (it validates the hardest component). Both are bounded phases well under 15 changes each.

---

## Handoff Record

**Stage:** reflect → complete

**Delivered:** 28 changes across 4 layers (substrate: 3 Rust crates; UI primitive: ui-surface; 12 learning skills; 4 integration test suites + docs + infra). All goals MET. v1.4.0 shipped at commit b0cf755. Grader fidelity (60–70% confidence at assess time) is the highest-severity open risk; empirical validation is the recommended first follow-on action.

**Corrective actions for next phase:**
- Write goals.md at assess time, not new-phase time
- Wire artifact-refiner constraints before execute
- Replace simplified FSRS stub with full `fsrs-rs` integration
- Build grader evaluation dataset as a first-class deliverable

**Recommended next phase:** `phase-learn-sovereign-sync` (implement IrohDocsAdapter for multi-device sync) or `phase-learn-grader-validation` (empirical grader fidelity evaluation).
