# Assessment — phase-learn-feynman

**Phase:** phase-learn-feynman
**Assessed:** 2026-06-28
**Brief version:** Feynman-Spine Learning & Education Capability (§0–§12)
**Assessor discipline:** PMPO assess protocol. Claims routed through sycophancy-correction §S-01–S-08. Confidence expressed as ranges. Trade-offs surfaced rather than buried. Open questions resolved or explicitly deferred with cause. No plan content. No skill files. Halt here for approval.

---

## 1. Honest Premise Assessment

The brief is unusually well-grounded for a feature seed. The research citations are real, the effect sizes are from known meta-analyses, and the three failure-mode vignettes in §11 are structurally honest. That said, the assess must name the following before proceeding:

### 1a. Where the brief is correct and should be preserved
- The core architectural claim — **skills emit intent; substrate renders and persists; Tier 0 is the universal floor** — is correct and is already the pack's established pattern. Preserve it.
- **Feynman as a bundle of validated components** (not a branded ritual) is the right framing, and the architectural decisions that flow from it (external grader, explicit learner model, retention engine) are individually justified by the cited evidence.
- The three failure modes in §11 are genuine and the mitigations named are architecturally sound. The recursion rabbit-hole, the confident-incompetence amplifier, and scope-creep-into-three-products are the real risks. This assess accepts all three as first-class requirements.
- The research-forced additions in §3 are all valid. Every one is confirmed as a requirement.
- The locked decisions in §7 are accepted without challenge. No locked decision is overturned.

### 1b. Where the brief overstates confidence
- **"CRDT-shaped state is straightforward"** — the brief states learner/progress state should be CRDT-shaped as if this is a design detail. It is not. Making multi-device merge conflict-free requires *defining the conflict semantics* for every field in the learner model (mastery estimates, FSRS scheduling state, gap records, credential evidence). This is a non-trivial design problem and the brief treats it as solved. **Assessment correction:** the CRDT shape is required but the field-level conflict semantics are an open design question that must be resolved in plan, not assumed.
- **"The grader checks the explanation against the content-grounding corpus"** — this implies the grader is a RAG retrieval + semantic comparison system that can reliably detect wrong mental models vs. correct-but-incomplete explanations vs. confident-wrong explanations. This is harder than it sounds. The 2025–2026 evidence on LLM-as-evaluator accuracy is mixed, even with grounding. The grader architecture needs a spike before plan treats it as solved. **Assessment correction:** mark grader implementation fidelity as a risk requiring a design spike.
- **"Certification integrity: novel transfer problems the operator cannot trivially have the agent solve"** — this is structurally impossible to guarantee when the operator has the agent in the same session. The brief acknowledges this as an open question (§9.9) but does not quantify the risk. **Assessment correction:** certification integrity is a *trust model*, not a technical enforcement. OB 3.0 / W3C VC certifies the evidence bundle, not the test-taking condition. The integrity claim depends on the operator's acknowledgement that the credential represents their learning, not the agent's. This should be stated plainly in the design.

### 1c. Where the brief underspecifies
- **`surface-bridge` scope.** The brief describes it as "from the prior cross-harness UI analysis." No such analysis is present in the codebase (`skills/` directory has no `surface-bridge` skill, no prior cross-harness UI spec). This is a greenfield build, and the brief's description ("MCP App server, AG-UI endpoint emitting A2UI specs, `detect-surface-tier` probe") is a design goal, not a reference to existing work. **Assessment correction:** `surface-bridge` is a new substrate component requiring its own design sub-phase. It is on the critical path of every learning skill's interactive mode.
- **`content-grounding` corpus assembly.** The brief says "deep research to scope the subject and build a target-proficiency rubric" but does not specify how the grounding corpus is built for arbitrary subjects. For math-of-LLMs this means curated papers, textbooks, and reference implementations. For "history of Byzantine art" it is a different corpus. The corpus assembly process is per-subject and must be part of `learn-goal`'s design, not assumed.
- **`learner-model` cold-start.** The brief lists BKT/DKT/PFA/LLM-seeded Bayesian network as candidates but defers to §9.1. This is the most consequential open question because the cold-start representation determines what the `learn-survey` diagnostic step can produce. Without resolving this, the `learn-survey` → `learner-model` interface is undefined.

---

## 2. §3 Additions — Committed or Rejected

| Addition | Verdict | Reasoning |
|---|---|---|
| External-grounded gap detection | **COMMITTED** | Confirmed required; grader architecture is a spike risk |
| Explicit persistent learner model | **COMMITTED** | Source of mastery truth; cold-start model is §9.1 dependency |
| Retention engine (FSRS) | **COMMITTED** | `fsrs-rs` is available; scheduling over Feynman artifacts is mandatory |
| Deliberate-practice / application track | **COMMITTED** | Procedural fluency gap is real; `learn-practice` is required, not polish |
| Misconception detection | **COMMITTED** | Confident-wrong explanations are the grader's hardest case; must be explicit |
| Assessment validity and integrity | **COMMITTED (trust model)** | Technical enforcement is impossible in-session; replaced by a trust-model design |
| Content provenance and hallucination guardrails | **COMMITTED** | Feynman amplifies inputs; source-grounding is load-bearing |
| Adherence and effort-reframing | **COMMITTED** | Misinterpreted-effort is a system-level risk, not UX polish |
| Recursion floor | **COMMITTED** | Set by `learn-survey`; enforced as a hard bound in `feynman-loop` |
| Honest feasibility gating | **COMMITTED** | Agent must be able to say "not reachable in your time" |
| Re-planning loop | **COMMITTED** | `learn-plan` re-runs as learner model updates; not a one-shot plan |
| Learner-model privacy | **COMMITTED** | Sovereign storage + selective disclosure (OB 3.0 supports) |

All 12 additions are confirmed requirements.

---

## 3. §9 Open Questions — Resolved or Deferred

### Q1: Knowledge-tracing model (BKT vs. DKT vs. PFA vs. LLM-seeded Bayesian)
**Resolution: LLM-seeded Bayesian network for cold-start, with PFA transition at ≥ 5 observations per concept.**

Reasoning:
- BKT is interpretable but requires substantial data per skill before reliable estimates — poor for cold-start on novel subjects.
- DKT (deep knowledge tracing) is highest accuracy at scale but requires significant historical data and a trained model per domain — unacceptable for an arbitrary-subject system without pre-training.
- PFA (performance factors analysis) is good with moderate data and handles multiple prerequisite factors, but also needs observation history.
- LLM-seeded Bayesian network for cold-start is the correct design for the "arbitrary subject, first session" condition: the LLM primes a probability distribution over concept mastery from the `learn-survey` diagnostic, providing a structured prior. Once the system has ≥ 5 observations per concept (roughly 2–3 `feynman-loop` iterations), transition to PFA updates using actual performance evidence.

**Decision:** cold-start = LLM-seeded Bayesian priors from survey diagnostic; ongoing updates = PFA-style incremental updates from `learn-grade` outputs. No DKT; the trained-model dependency is inconsistent with arbitrary subjects.

**Confidence: 75–85%.** The hybrid approach is defensible but untested in this exact configuration. A spike is required before plan locks the `learner-model` interface.

---

### Q2: CRDT engine for learner state (Loro vs. automerge-rs)
**Resolution: automerge-rs, with deferred migration path to Loro if Flint Realtime chooses it.**

Reasoning:
- Both are mature. automerge-rs has a wider adoption base in the Rust ecosystem, is WASM-compatible (relevant for browser and edge surfaces), and has stronger multi-author conflict resolution semantics for the specific case of concurrent mastery estimate updates from multiple devices.
- Loro is newer, faster at large-document sizes, and has a richer type system (but the learner model is not a large document — it is a compact JSON-shaped state).
- The brief notes this is an open Flint Realtime decision that propagates here. **Correctamendment:** the correct posture is to define a `CrdtEngine` trait in the `storage-provider` layer and implement automerge-rs as the default adapter. If Flint picks Loro, the adapter can be swapped without touching learning skill code.

**Decision:** abstract CRDT engine trait; automerge-rs default adapter; Loro adapter is a one-change swap if Flint Realtime resolves differently. Conflict semantics for learner model fields must be documented in the `learner-model` schema (field-level merge strategy per field type).

**Confidence: 85%.** The abstraction insulates the decision; the automerge-rs choice is low risk given the learner-model's compact state size.

---

### Q3: Default storage adapter priority
**Resolution: local-dir first, Iroh-docs second, AT-Proto PDS third, IPFS/Kubo as tertiary adapter.**

Reasoning:
- Local-dir is always available, requires no network, and is the correct default for a learner's private mastery state. Sync is a secondary concern; ownership is primary.
- Iroh-docs (QUIC P2P, CRDT key-value, content-addressed blobs via iroh-blobs) is the sovereign Rust-native multi-device adapter. The brief correctly identifies it as the target default for sync.
- AT-Proto PDS is already in the stack (for did-plc credential issuance) and is appropriate for *credential and evidence* sync (public-facing OB 3.0 VCs), not for private mastery state.
- IPFS/Kubo: valid tertiary for content-addressed artifact archiving but NOT a sync substrate; the brief's decision to demote it is correct.

**Priority order:** `local-dir → iroh-docs (sync) → AT-Proto PDS (credentials only) → S3-compatible → IPFS (archive only)`.

The split between private mastery state (local-dir + Iroh-docs) and public credential evidence (AT-Proto PDS) is architecturally correct and must be a first-class design distinction in the `storage-provider` schema.

**Confidence: 90%.** This is a well-understood problem given the 2026 sovereign-sync landscape.

---

### Q4: Credential issuer model (self-issued VC vs. 1EdTech-certified issuer)
**Resolution: self-issued W3C VC with did-plc DID, deferred 1EdTech certification.**

Reasoning:
- Self-issued VC with did-plc is already on the stack, requires no operational overhead, verifies offline, and provides sovereign ownership of the credential. This is the correct default.
- 1EdTech certification adds external verifier trust (recognized by HR systems, academic institutions) but requires maintaining an accredited issuer relationship — this is an operational product decision, not a skill-pack decision.
- The correct design: the `learn-certify` skill emits a self-issued W3C VC in OB 3.0 format. The VC is designed to be *re-signed* by a 1EdTech-certified issuer later without changing the evidence structure. This preserves portability.

**Decision:** self-issued OB 3.0 / W3C VC with did-plc by default. 1EdTech certification is a plan-time option documented as a `learn-certify --issuer <endpoint>` parameter, not a hard dependency.

**Confidence: 95%.** This is a clean technical decision; the operational question is explicitly deferred.

---

### Q5: Grader grounding corpus (per-subject source vetting + deep-research budget)
**Resolution: `learn-goal` assembles a subject-specific grounding corpus via a bounded deep-research loop; corpus is stored under `storage-provider` and referenced by `learn-grade`.**

Reasoning:
- The `learn-goal` research phase already uses the `pmpo-elicit` research loop (6 sources, 10 minutes). The grounding corpus assembly requires a larger budget: 12–20 vetted sources, up to 30 minutes, with a source-type priority: primary literature > textbooks > reference implementations > curated surveys > secondary sources > LLM synthesis (last resort, flagged).
- Corpus sources are stored with provenance records (`source_ref`, `source_type`, `retrieved_at`, `confidence`). Any content used in `learn-grade` must trace to a provenance record.
- Misconception detection requires the corpus to include known wrong answers and common error patterns for the subject, not only correct content.

**Design constraint added:** the corpus assembly process is a named sub-step of `learn-goal` with its own output artifact (`grounding-corpus.json` + source files). It is not folded into the feasibility gate.

**Confidence: 80%.** The corpus assembly for novel subjects is the hardest part of the grounding requirement; the bounded budget prevents runaway research but may under-cover niche subjects. This is a known trade-off.

---

### Q6: Recursion depth budget defaults and feasibility-gate thresholds
**Resolution: depth budget = 3 levels below the declared target concept; feasibility gate = honest-time estimate.**

Depth budget:
- The recursion floor is the learner's declared prior-knowledge frontier (set by `learn-survey`). Above the floor: no recursion. Below the declared target: recursion permitted up to 3 levels deep (i.e., if target is "attention mechanisms," recursion goes to linear algebra prerequisites but not to set theory).
- 3 levels is the default; the operator can declare a depth override in `learn-goal`. The `feynman-loop` hard-stops at the depth limit and surfaces a "prerequisite flagged as requiring separate learning goal" signal rather than continuing to recurse.

Feasibility gate:
- Research-derived time-to-mastery (from `learn-goal` corpus) is compared to operator's stated available hours/week × stated duration.
- Gate thresholds: if estimated time > 1.5× available time → RED (not reachable; agent must say so explicitly, not soften); 1.0–1.5× → YELLOW (achievable with priority, name the trade-offs); < 1.0× → GREEN.
- The 1.5× multiplier accounts for underestimation bias in self-reported available time. This is the sycophancy-correction equivalent in the feasibility domain: the agent does not round down the honest estimate to make the goal seem achievable.

**Confidence: 80%.** The 3-level depth default and 1.5× multiplier are judgment calls; they should be configurable in `learn-goal` and tunable by the operator's domain expertise level.

---

### Q7: Learner-model placement relative to surreal-memory
**Resolution: learner-model is a separate service; concept DAG uses surreal-memory graph; mastery estimates in learner-model only.**

Reasoning:
- surreal-memory stores the knowledge graph (concept nodes + prerequisite edges = the concept DAG). This is appropriate — it is graph-shaped and the existing query surface handles traversal and semantic search.
- Mastery estimates, FSRS scheduling state, and session history are time-series per-concept data that need append semantics and efficient temporal queries. These do NOT belong in a graph store; they belong in the `learner-model` service (a compact CRDT document per learner + an FSRS-scheduled review queue).
- Integration: `feynman-loop` writes concept nodes to surreal-memory (new concepts discovered via recursion) AND mastery updates to `learner-model`. The `learn-plan` skill queries both: the DAG from surreal-memory and mastery state from `learner-model`.

**Design:** `learner-model` service = local-file-backed CRDT document (automerge-rs) + iroh-docs sync + FSRS queue. surreal-memory = concept DAG and semantic search only. No mastery state in surreal-memory.

**Confidence: 90%.** The split is clean and avoids overloading surreal-memory with time-series semantics it is not designed for.

---

### Q8: `learn-grade` boundary (standalone skill vs. folded into `feynman-loop`)
**Resolution: standalone skill.**

Reasoning:
- `learn-grade` has a distinct input/output contract (explanation text + concept + grounding corpus → gap signals + novel transfer problems + mastery update) that is reusable independently of `feynman-loop`. It is called by `feynman-loop` AND by `learn-certify` (checkpoint grading). Folding it into `feynman-loop` violates the single-responsibility principle.
- The sycophancy-correction path on `learn-grade` is more explicit when it is a named skill with a defined grader system prompt. If it is folded into the loop, the anti-sycophancy guard becomes harder to audit.

**Decision:** `learn-grade` is a standalone skill. `feynman-loop` calls it; `learn-certify` calls it. Both pass the grounding corpus reference.

**Confidence: 95%.** This is unambiguous.

---

### Q9: Integrity model for certification
**Resolution: trust model + procedural guardrails, not technical enforcement.**

As stated in §1b: it is structurally impossible to prevent an operator with an in-session agent from having the agent solve the transfer problems. The OB 3.0 / W3C VC model handles this correctly: the credential asserts *evidence of demonstration*, not *evidence of unassisted demonstration*.

**Design decision:** the `learn-certify` skill:
1. Generates novel transfer problems that are *not* the same as the Feynman explanations (so copying the loop output does not solve them).
2. Requires the learner to explain their reasoning in their own words — which is itself a Feynman step, gradeable by `learn-grade`.
3. Includes in the VC evidence bundle: the explanation artifacts, the transfer problem responses, and the learner-model mastery trajectory over time. A credential reviewer can inspect the trajectory; a step-change from zero mastery to certification in one session would be anomalous.
4. Emits a self-attested declaration field in the VC: the operator signs the credential with did-plc, asserting it represents their learning. This is the trust model — the same model used by professional certifications that do not proctor every exam.

**Procedural guardrail:** `learn-certify` flags anomalous mastery trajectories (e.g., no intermediate checkpoint history) as a warning in the VC metadata. The grader runs sycophancy-correction on the transfer problem responses to detect responses that look agent-generated (high fluency, no hesitation, covers edge cases perfectly).

**Confidence: 85%.** The trust model is the right frame; the anomaly-detection guardrail adds meaningful signal without claiming unverifiable enforcement.

---

## 4. Skill Specification

### Layer A — Substrate Services

#### `surface-bridge` (NEW — greenfield, critical path)
**Type:** Rust service (Axum) + MCP server transport
**Inputs:** `detect-surface-tier` probe → returns one of: `text`, `structured-prompt`, `mcp-app-iframe`, `agui-a2ui`, `full-external`
**Outputs:** resolved surface tier + render adapter per tier
**Consumes:** harness environment variables, MCP transport capabilities
**Critical constraint:** `surface-bridge` must be an optional dependency. When absent, all learning skills fall through to Tier 0 (text/markdown). This is enforced by the design, not by configuration.
**Spike required:** Yes. `detect-surface-tier` probe logic and the MCP App / AG-UI serving mechanism are greenfield.

#### `storage-provider` (NEW)
**Type:** Rust trait crate with adapter implementations
**Trait surface:** `read(key) → bytes`, `write(key, bytes)`, `merge(key, crdt-delta)`, `list(prefix)`, `watch(key) → stream`
**Adapters (in priority order):** `local-dir`, `iroh-docs`, `at-proto-pds` (credentials only), `s3-compatible`, `ipfs-kubo` (archive only)
**CRDT engine:** `CrdtEngine` trait; automerge-rs default implementation
**Field-level conflict semantics (must be specified in schema):**
  - mastery estimates: LWW (last-write-wins with vector clock); the more recent observation wins
  - FSRS scheduling state: merge = take max stability, min due-date (conservative — prefer more review, not less)
  - gap records: union append (gaps never deleted by merge; only resolved by grading)
  - credential evidence: append-only; no merge conflict possible

#### `learner-model` (NEW)
**Type:** Rust crate + local CRDT document + FSRS-6 scheduler
**State schema:**
```
{
  learner_id: DID,
  concepts: { [concept_id]: { mastery: float[0,1], observations: [{timestamp, score, source_skill}], fsrs_card: FSRSCard } },
  gaps: { [gap_id]: { concept_id, description, detected_at, resolved_at?, source_skill } },
  sessions: [{ session_id, started_at, skills_called, concepts_touched }]
}
```
**Backed by:** `storage-provider` (local-dir primary, iroh-docs sync)
**FSRS:** `fsrs-rs` (open-spaced-repetition/fsrs-rs) for all scheduling; review queue derived from FSRS due dates
**Cold-start:** LLM-seeded Bayesian priors from `learn-survey` diagnostic output (structured JSON: `{concept_id, estimated_mastery_prior, confidence}`)
**DAG:** concept nodes + prerequisite edges stored in surreal-memory; `learner-model` holds mastery state per node, not the graph structure
**Spike required:** Yes. The CRDT conflict semantics for FSRS state and the cold-start prior format require validation before `learn-survey` interface is locked.

#### `content-grounding` (NEW)
**Type:** skill-callable service (MCP tool or local shell script)
**Inputs:** subject description, target proficiency level, budget (sources, minutes)
**Outputs:** `grounding-corpus.json` = array of `{source_ref, source_type, content_summary, provenance, retrieved_at, confidence}`
**Source priority:** primary literature > textbooks > reference implementations > curated surveys > secondary sources > LLM synthesis (flagged)
**Misconception sources:** the corpus must include known-wrong-model examples for the subject, not only correct content
**Backed by:** `pmpo-elicit` research loop with extended budget (12–20 sources, 30 minutes max)
**Spike required:** No. This is a parameterized version of the existing research loop. The misconception-source requirement is novel but implementable.

---

### Layer B — UI Primitive

#### `ui-surface` (NEW skill: `skills/learn/ui-surface/`)
**Purpose:** generalization of `pmpo-elicit`'s dual-mode render. Takes a UI intent + surface tier → renders + reads back.
**Inputs:** `{intent_type: survey|explanation|grading|review|report, content: ..., min_tier, preferred_tier}`
**Outputs:** operator response + surface tier used
**Tier 0 (universal floor):** plain markdown, checklist, linked resources — always available, no substrate dependency
**Tier 1:** `AskUserQuestion` (Claude Code), structured file prompt + response file (OpenCode/Codex/Kimi/Zed)
**Tier 2:** MCP App iframe (via `surface-bridge`) or AG-UI → A2UI (on hosts that speak it)
**Tier 3:** external browser / desktop panel (full A2UI app)
**Degradation:** if `surface-bridge` is absent or returns a lower tier than `preferred_tier`, silently use what is available. Log the tier gap to the learner session. Never block on preferred_tier.

---

### Layer C — Learning Skills (`skills/learn/`)

#### `learn-goal`
**Entry command:** `/learn-goal "<desire>"`
**Purpose:** scope the subject, build rubric, assemble grounding corpus, elicit operator parameters, run feasibility gate.
**Inputs:** operator desire string
**Outputs:**
- `learn-goal.json`: `{subject, target_level, rubric: [{concept, criterion, assessment_method}], grounding_corpus_path, feasibility: {status: RED|YELLOW|GREEN, estimated_hours, available_hours, multiplier}}`
- `grounding-corpus.json` (via `content-grounding`)
**Consumes:** `content-grounding`, `pmpo-elicit`, `ui-surface`
**Sycophancy gate:** feasibility calculation runs through `sycophancy-correction` — the result may not be softened by the agent
**Sub-step: corpus assembly** (named, distinct from feasibility gate): assembles grounding corpus for subject, including misconception sources

#### `learn-survey`
**Entry command:** `/learn-survey` (called by `learn-goal` or standalone)
**Purpose:** diagnostic placement — probes current standing with objective items, detects misconceptions, sets recursion floor, seeds `learner-model` cold-start priors.
**Inputs:** `learn-goal.json`, `grounding-corpus.json`
**Outputs:**
- `survey-result.json`: `{concepts_probed: [...], mastery_priors: {[concept_id]: float}, misconceptions_detected: [...], recursion_floor: [concept_id], learner_model_seed: <JSON for learner-model>}`
**Consumes:** `ui-surface` (Tier 1 preferred for interactive survey, Tier 0 fallback as checklist file), `learner-model` (write cold-start priors), `content-grounding` (grounding corpus for misconception detection)
**Extends:** `pmpo-elicit` (research loop for subject-scoped objective items), `zeespec-interrogator` (structured interrogation pattern)
**Critical output:** `recursion_floor` = set of concept IDs the learner demonstrably owns. `feynman-loop` never recurses into these.

#### `learn-plan`
**Entry command:** `/learn-plan` (called after `learn-survey`)
**Purpose:** build prerequisite-gated, ordered curriculum from survey result to target level; realistic time budget; re-plans as learner model updates.
**Inputs:** `learn-goal.json`, `survey-result.json`, `learner-model` (current mastery state)
**Outputs:** `curriculum.json`: `{phases: [{phase_id, concepts: [...], prerequisite_concepts: [...], estimated_hours, feynman_loops: [...]}], total_estimated_hours, schedule_suggestion}`
**Consumes:** surreal-memory (concept DAG queries), `learner-model` (mastery query), `ui-surface`
**Re-plan trigger:** `feynman-loop` calls `learn-plan --replan` when a concept's mastery estimate diverges > 0.2 from the plan's assumption
**Renders:** concept DAG as Tier-2 visual via `ideation-mindmap` when available; Tier-0 ordered list otherwise

#### `feynman-loop`
**Entry command:** `/feynman-loop <concept_id> [--depth <n>] [--audience novice|peer|skeptic]`
**Purpose:** the core Feynman cycle for one concept at one audience level. Recursive on gaps; horizontally escalating on audience.
**PMPO mapping:**
- Spec: select concept + target audience + depth budget
- Plan: how to structure the explanation (analogies, examples, sub-concepts)
- Execute: produce plain-language explanation + analogies + teach-the-skeptic pass (operator writes or dictates; agent scaffolds)
- Reflect: call `learn-grade` → gap signals → candidate child loops
**Recursion:** gaps with `mastery_prior < recursion_floor_threshold` AND concept not in `recursion_floor` → spawn child `feynman-loop` with `depth - 1`. Hard-stop at `depth == 0`: surface gap as "requires separate learning goal" rather than recurse further.
**Horizontal escalation:** `feynman-loop --audience novice` → `--audience peer` → `--audience skeptic`. Difficulty dial maps to: worked-example → completion → independent → teach-the-skeptic.
**Outputs per cycle:** `feynman-artifact.json`: `{concept_id, audience, explanation_text, analogies, gaps_detected: [...], transfer_problems_passed: [], mastery_estimate_post: float, session_id}`
**Consumes:** `learn-grade` (grading), `learner-model` (read mastery pre, write mastery post), `ui-surface`, `content-grounding` (via learn-grade)

#### `learn-grade`
**Entry command:** not user-facing — called by `feynman-loop` and `learn-certify`
**Purpose:** external, source-grounded, sycophancy-corrected grader. Checks explanation against corpus; emits gap signals and novel transfer problems; updates learner model.
**Inputs:** `{explanation_text, concept_id, audience, grounding_corpus_path, learner_model_ref}`
**Outputs:** `grade-result.json`: `{gaps: [{gap_id, description, severity, source_evidence}], misconceptions: [{concept, wrong_model, correction_source}], transfer_problems: [{problem_text, expected_approach, concept_tested}], mastery_update: {concept_id, delta, confidence}}`
**Grader anti-sycophancy:** runs `sycophancy-correction` on grader output before returning. A grade that finds no gaps when the explanation has obvious gaps is flagged as S-02 (ungrounded validation).
**Misconception detection:** checks explanation against known-wrong-model entries in grounding corpus; flags confident assertions that match misconception patterns
**Novel transfer problems:** generated from corpus, NOT from explanation text (prevents trivial self-plagiarism); must probe the concept from a different angle than the explanation covered
**Consumes:** `content-grounding` corpus (via path reference), `sycophancy-correction`, `learner-model` (write)

#### `learn-retain`
**Entry command:** `/learn-retain` (called on schedule or by `feynman-loop` post-cycle)
**Purpose:** FSRS-6 scheduling over feynman-artifacts and gap records; surfaces spaced reviews.
**Inputs:** `learner-model` (FSRS card state), completed `feynman-artifact.json` records
**Outputs:** review session via `ui-surface` (Tier 1 prompt: "Review this explanation — what's still correct? What would you change?"), updated FSRS card state in `learner-model`
**FSRS integration:** on each review response, call `fsrs-rs` `next_states()` with the rating; persist updated `FSRSCard` to `learner-model`; schedule next review
**Triggers:** (a) post-`feynman-loop` interval trigger (initial review 1 day later); (b) operator-invoked; (c) `learn-plan` schedule integration

#### `learn-practice`
**Entry command:** `/learn-practice <concept_id> [--type derivation|implementation|transfer]`
**Purpose:** deliberate-practice / application track. Carries procedural mastery that Feynman does not.
**Problem types:**
- `derivation`: re-derive a result from first principles without looking
- `implementation`: implement the concept in code, math, or a domain artifact
- `transfer`: solve a problem where the concept applies in a novel context (generated by `learn-grade` transfer-problem pool)
**Difficulty progression:** interleaved across problem types (not blocked by type); mastery-gated (harder problems unlock when learner-model mastery > threshold)
**Inputs:** `learner-model`, `grounding-corpus.json`, concept DAG (surreal-memory)
**Outputs:** `practice-result.json` → mastery update to `learner-model`; practice artifacts referenced in `learn-certify` evidence bundle
**Consumes:** `learn-grade` (for grading practice responses), `ui-surface`, `learner-model`

#### `learn-certify`
**Entry command:** `/learn-certify [--checkpoint | --final]`
**Purpose:** checkpoint tests + capstone; OB 3.0 / W3C VC evidence bundle; progress chart; trust-model integrity guardrails.
**Checkpoint mode:** runs `learn-grade` over a set of curriculum concepts; updates learner model; emits an intermediate badge with evidence.
**Final mode:** requires:
- ≥ N feynman-artifacts (N = `learn-goal` curriculum length, subject to learner-model mastery threshold across all target concepts)
- ≥ M practice results (M configurable, default = 3 per target concept)
- Capstone: a novel transfer problem + a teach-the-skeptic explanation for one umbrella concept, graded by `learn-grade`
- Mastery trajectory showing progression (not a step-change from zero)
**OB 3.0 / W3C VC output:** self-issued, signed with did-plc, evidence fields = feynman-artifact paths + practice-result paths + grade-result paths + mastery trajectory snapshot
**Integrity guardrails:**
- Anomaly flag: step-change from zero mastery to certification (no intermediate history) → `integrity_warning: true` in VC metadata
- Anti-agent-plagiarism: `learn-grade` runs sycophancy-correction on capstone responses; flags responses that show anomalous fluency + zero hesitation + perfect edge-case coverage (S-01 pattern applied to transfer problems)
- Trust declaration: operator signs VC with did-plc assertion that it represents their learning
**Progress chart:** Tier 0 = text mastery table per concept; Tier 2 = visual mastery radar via `ui-surface` if available
**Consumes:** `learn-grade`, `learner-model`, `storage-provider` (AT-Proto PDS for VC publication), `ui-surface`

---

## 5. Per-Concept Mastery Criterion (Concrete)

A concept loop (`feynman-loop`) closes when ALL of the following hold:

1. **Explanation grade:** `learn-grade` finds ≤ 1 minor gap (no major gaps, no misconceptions)
2. **Transfer problem:** learner solves ≥ 2 novel transfer problems generated by `learn-grade` with score ≥ 0.7 on each
3. **Retention check:** `learn-retain` has been triggered at least once (≥ 24 hours after the explanation) and the review response passes `learn-grade` at the same threshold
4. **Self-reported fluency is NOT a criterion.** It is recorded for calibration research but does not gate loop closure.
5. **Depth budget respected:** the loop is within its declared depth budget (not truncated early, not overrun)

Mastery estimate in `learner-model` after loop closure: 0.8–0.9 (not 1.0; mastery is probabilistic and FSRS will confirm or decay it over time).

---

## 6. Recursion Bound (Concrete)

- **Floor:** concept IDs in `survey-result.recursion_floor` — never recursed into, ever.
- **Depth limit:** 3 levels below the declared target concept (default; operator-overridable in `learn-goal`).
- **Depth accounting:** `feynman-loop` tracks `current_depth` in its call context. At `depth == 0`, gaps are surfaced as: `"Prerequisite '{concept}' is below your recursion floor. Create a separate learning goal: /learn-goal '{concept}'"` — never auto-created, always operator decision.
- **Budget gate:** before spawning a child loop, `feynman-loop` checks estimated cost (hours) against remaining feasibility budget from `learn-goal.feasibility`. If child cost would cause total > 1.5× operator-available-time, it surfaces the trade-off rather than spawning silently.

---

## 7. Cross-Harness Degradation Behavior by Tier

| Tier | Available on | `learn-survey` | `feynman-loop` | `learn-certify` |
|---|---|---|---|---|
| Tier 0 | All harnesses | Checklist markdown file | Explanation prompt → text file | Mastery table + VC as markdown |
| Tier 1 | Claude Code, OpenCode, Kimi | `AskUserQuestion` structured | `AskUserQuestion` scaffolded | `AskUserQuestion` capstone |
| Tier 2 | Claude Code + MCP App, A2UI hosts | Interactive survey form | Explanation canvas | Visual mastery radar |
| Tier 3 | Browser/desktop panel | Full interactive survey app | Rich Feynman canvas | Full certificate + portfolio |

**Rule:** every skill declares `min_tier: 0` and `preferred_tier: 1` (default). The substrate may exceed preferred_tier if available; never blocks on it. **No skill hard-depends on Tier > 0.**

---

## 8. Locked Decisions (Confirmed Carried Forward)

All 9 locked decisions from §7 of the brief are confirmed without challenge:

1. Skills emit harness-agnostic intent and state; substrate renders and persists.
2. No learning skill hard-depends on substrate. Tier 0 fallback mandatory.
3. Explicit knowledge-tracing `learner-model` is mastery truth — not LLM judgment, not self-report.
4. `learn-grade` runs through `sycophancy-correction`. Pedagogy resists comfort-optimization.
5. Gap-fill content is grounded in vetted sources with provenance. Feynman amplifies its inputs.
6. Recursion bounded at declared target and prior-knowledge frontier (3-level default, `learn-survey` sets floor).
7. Storage provider-abstracted; CRDT-shaped state; local-dir → iroh-docs → AT-Proto PDS priority; IPFS is tertiary archive only.
8. Credentials are evidence-bound OB 3.0 / W3C VC with did-plc; self-issued default; 1EdTech certification deferred.
9. Rust-first for substrate (`fsrs-rs`, Axum, automerge-rs). Design-first, phase discipline, no skip-ahead.

**Three additional locks added by this assessment:**
10. `learner-model` is a separate service from `surreal-memory`. Concept DAG in surreal-memory; mastery state in `learner-model` only.
11. `learn-grade` is a standalone skill, called by both `feynman-loop` and `learn-certify`.
12. Certification integrity is a trust model + anomaly-detection guardrail, not technical enforcement. The VC asserts evidence, not un-assisted demonstration.

---

## 9. Deliberate Non-Goals (Preserved)

- Not a fixed-content LMS or course catalog.
- Does not use the LLM as a mastery judge (it is a grader assistant; the learner-model is the judge).
- Not a synchronous human-tutoring product.
- Does not invent UI protocols — implements A2UI, AG-UI, MCP Apps.
- Does not build the `learn-to-build` creation bridge (explicitly deferred, out of scope).
- Writes no code and no skill files during assess. This document is the assess deliverable.
- Does not guarantee certification integrity under adversarial conditions — the trust model is honest about this.

---

## 10. Spike Requirements Before Plan Can Lock

Two design spikes are required before the plan stage can commit interfaces:

**Spike 1: `surface-bridge` detect-surface-tier probe + MCP App serving**
- Unknown: exactly how each harness signals its MCP App capability and how A2UI specs are served at runtime
- Deliverable: a working `detect-surface-tier` probe that correctly identifies tier on Claude Code, OpenCode, Codex; a minimal MCP App server stub
- Estimated effort: S (1–2 changes)
- Blocks: `ui-surface`, and through it, all learning skills above Tier 0

**Spike 2: `learner-model` cold-start prior format + CRDT conflict semantics**
- Unknown: the structured JSON format for LLM-seeded mastery priors that `learn-survey` emits and `learner-model` ingests; the FSRS CRDT conflict semantics (merge strategy for FSRSCard across devices)
- Deliverable: a schema for `learner_model_seed` (output of `learn-survey`) and a formal conflict-semantics document for all `learner-model` CRDT fields
- Estimated effort: XS (1 change, design-only)
- Blocks: `learner-model` implementation and the `learn-survey` → `learner-model` interface

**Neither spike requires full implementation during assess or plan.** Both are design outputs (schemas + contracts). They unblock the interfaces so plan can commit to them.

---

## 11. Sequence Constraint (Preserved from §11 of brief)

The implementation sequence is forced by dependencies. No phase can start before its predecessors' interfaces are locked:

```
Layer A substrate (spikes → interfaces locked):
  surface-bridge (spike 1) + learner-model (spike 2) → ui-surface

Layer C first wave (interfaces depend on substrate):
  learn-goal + learn-survey + feynman-loop + learn-grade

Layer C second wave (depend on first wave mastery loop):
  learn-plan + learn-retain + learn-practice + learn-certify

Optional (deferred, no dependency from any above):
  learn-to-build
```

This sequence is a requirement, not a recommendation. Implementing `feynman-loop` before `learn-grade`'s interface is locked produces an orphaned artifact. Implementing `learn-certify` before `learn-practice` exists produces credentials with insufficient evidence.

---

## 12. Assessment Confidence Summary (Anti-Sycophancy Calibration)

| Decision | Confidence |
|---|---|
| Research citations and effect sizes | 85–90% (mid-2026 validated; literature moves) |
| Feynman-as-bundle framing | 95% (well-grounded) |
| LLM-seeded Bayesian + PFA hybrid for learner model | 75–85% (spike required) |
| automerge-rs CRDT default | 85% (abstracted; low-risk swap) |
| CRDT field-level conflict semantics (as designed) | 70% (spike required to validate) |
| Storage priority (local-dir → iroh-docs → AT-Proto PDS) | 90% |
| Grader fidelity for misconception detection | 60–70% (hardest part; spike recommended) |
| Certification trust model (not technical enforcement) | 90% (correct framing; honest) |
| Recursion floor mechanics | 85% |
| 3-level depth default | 75% (configurable; default is a judgment call) |
| Feynman-loop PMPO mapping | 90% |
| `learn-grade` as standalone skill | 95% |
| Sequence constraint (Layer A before C) | 98% (forced by dependency graph) |
| Scope-creep risk (§11.3) | **HIGH — explicitly flagged** |

The scope-creep risk is real and remains the highest risk in the phase. The Layer A substrate (`surface-bridge`, `storage-provider`, `learner-model`, `content-grounding`) is each product-sized. The mitigation — Tier 0 fallback for all skills, no hard substrate dependency — is correctly designed but requires active enforcement during plan and execute. If a single skill slips into "requires Tier 2 to work," the portability promise breaks.

---

*Assessment complete. No plan content. No skill files created. Halt for approval before /kbd-plan.*
