# Plan — ideation-and-decision-tools

> **Date:** 2026-07-30 · **Backend:** OpenSpec · **Changes:** 9
> **Inputs:** `assessment.md`, `analysis.md`, `library-candidates.json`
> (20 candidates, 7 build-required)

## Prerequisite blockers — verified clear before planning

`goals.md` names two blockers that gate every change below. Both were cleared earlier
in this session and **re-verified at plan time**:

| Blocker | State | Evidence |
|---|---|---|
| Commit the previous phase (150 paths, new submodule) | **CLEARED** | `c88a05f` + `ffc0bea` on `main`, pushed; `origin/main` behind 0 / ahead 0 |
| `scripts/update-skill-pack.sh --force` — stale plugin caches | **CLEARED** | `check-model-config.sh` → **0 DRIFT** (was 6); `kbd_require_producer_model` present in both the `.claude` and `.codex` 1.6.0 caches |

This is load-bearing for changes 001, 002 and 005: they depend on the producer≠judge
guard being live **in installed copies**. Before the cache refresh it was not, so a
creator running from a plugin cache would have used a resolver without the fail-closed
guard — and the `verified-distinct` stamp would have been unearned.

## Ordering rationale

The analysis recommendation is **goals first**. An earlier draft ordered WIT
reconciliation and mobile fabric work ahead of the ideation goals; a cross-model
judge blocked it, correctly — that ordering would have produced a transport/plugin
phase with the ideation capability as a trailing hope.

So changes 001–007 satisfy the seven phase goals against substrate that already
exists and is proven. Changes 008–009 record the fabric decisions the user asked for
without building them here; the analysis is explicit that **none of the fabric work
is a prerequisite for shipping the ideation capability**.

**Dependency spine:** 001 (packet mode) → 002 (diversity) → 003 (countermeasures)
→ 004 (persistence) → 005 (coach) → 006 (harness) → 007 (fixtures prove all of it).
007 must be last: it is the evidence that the rest works, and evidence written before
the thing it tests is not evidence.

---

## change-idt-001 — `--mode decision` review packet
**Goal:** 2 · **Library:** `cand-014` (adapt) · **Agent:** general-purpose

Add a `decision` mode to `build-review-packet.sh` so an *idea* can be judged by the
same machinery that judges skills and agents. Reuses the judge, findings schema,
retry loop, and sycophancy screen unchanged — the smallest change that makes goal 2
true.

Packet carries: the idea statement, stated assumptions, the decision being made,
what would falsify it, and prior related decisions from the `pk` wiki. Manifest-level
and capped, following the truncation-recording contract already shipped.

Also needs `assets/reviewer-mandate-decision.md`, and `dispatch-judge.sh` accepts the
new mode (the gate already errors on a missing mandate, so a mode without one fails
loudly).

**Enforcement, not demonstration.** Goal 2 says *every* decision artifact carries
`cross_model_check: verified-distinct`. Proving one artifact can is not the same
property. This change therefore adds a **schema requirement plus a validator**: a
decision artifact missing `cross_model_check`, or carrying `same-model-collision` or
`unverified-producer-unknown`, **fails validation** rather than being written and
quietly trusted. That mirrors the fail-closed producer guard already shipped.

**Acceptance:**
- `--mode decision` builds a packet; `dispatch-judge.sh --mode decision` resolves its
  mandate and stamps `cross_model_check`.
- The decision-artifact schema **requires** `cross_model_check`.
- A validator **rejects** any decision artifact whose `cross_model_check` is absent
  or is not `verified-distinct`; rejection is asserted by a test, not by inspection.
- A live run produces `verified-distinct` end to end.

---

## change-idt-002 — enforced-diversity ideation flow
**Goal:** 1 · **Library:** `cand-016`, `cand-017`, `cand-015` (adapt/adopt) · **Agent:** general-purpose

Adapt the two ideation skills that already exist rather than writing new ones:
`ideation-mindmap` (6-branch concept map) and `validate-idea` (three staged gates
with an Archive of Stepping Stones). Route both through `kbd-idea-critic`, which
already encodes *"the idea that proposed the idea should never also grade it"*.

**Diversity is enforced structurally, not prompted.** Independent generation → pool →
judge. Chen et al. (2026) found multi-agent LLM ideation synchronises *despite*
architectural attempts to diversify, and Mullen et al. (1991) found interacting groups
underperform nominal groups with the gap growing by size. Persona round-tables are
contraindicated twice; this change must not add one.

Revise the `kbd-idea-critic` rubric to weight **executability over novelty** — the
Ideation-Execution Gap (arXiv 2506.20803) showed LLM idea rankings *flip* after
execution, so pre-execution novelty is the wrong measure.

**Acceptance:**
- Generation produces **at least 3** candidate sets before any pooling.
- **Independence is mechanical, not prompted:** each set is produced in a separate
  dispatch that receives no other set as input. Asserted by a test that inspects the
  dispatch inputs, not by reading the prompt text.
- The critic never scores its own output (producer != judge, asserted).
- The rubric weights executability above novelty.
- **No round-table step exists** — asserted by the absence of any dispatch whose input
  includes another candidate set before pooling.

---

## change-idt-003 — automation-bias countermeasures
**Goal:** 3 · **Build-required** · **Agent:** general-purpose

The only fully-new work in the goal set, and the differentiator no surveyed tool has.

**Commit-before-reveal:** the user records their own judgement *before* the system
shows its analysis. Microsoft Research (2025) found confidence in AI among the
strongest predictors of whether users engage in critical thinking at all; explainable
output *increased* trust while promoting over-reliance ("False Confirmation" errors).

Also: calibrated confidence rather than a bare score, and mandatory surfacing of
disconfirming evidence. For irreversible personal decisions — the relationship and
career questions named in the seed — this is the governing constraint.

**Acceptance (observable, not judgement calls):**
- The flow **refuses to emit analysis** until a user judgement is on record —
  asserted by a test that invokes it without one and expects a refusal, not analysis.
- Output contains a **machine-checkable confidence field** and a non-empty
  `what_would_change_this` field; both are schema-required, and absence fails
  validation. *(Not "states confidence honestly" — that is unfalsifiable.)*
- Output contains at least one **disconfirming** item, schema-required.
- A fixture proves the gate cannot be bypassed: invoking with the reveal step forced
  still yields a refusal.

---

## change-idt-004 — decision persistence + outcome revisit
**Goal:** 4 · **Library:** `cand-019` (adapt) · **Agent:** general-purpose

Extend the `pk` wiki rather than building storage. OKF v0.1 requires only a non-empty
`type`, so a `decision` entry type is additive. Add a revisit query answering *"you
decided X six months ago — what happened?"*

This is the largest single gap in the surveyed market: **zero of 21 tools** track
outcomes over time. Given the Ideation-Execution Gap, it is also the only thing that
makes a score meaningful.

**Note the ingest-quality risk:** this phase alone auto-generated 10 near-duplicate
`*completion*` wiki entries. A decision type that inherits that behaviour produces
noise, not memory.

**Outcomes are the point, not decisions.** Goal 4 requires decisions *and their
outcomes*. Writing a decision entry alone is only half the loop, and it is the half
every surveyed competitor already has. The entry therefore carries an explicit
**outcome status** — `pending` on write — plus an **outcome-update flow** that fills
in what actually happened and when. Without that, the revisit query returns what was
decided but never what it was worth, which is exactly the feedback the
Ideation-Execution Gap says is missing.

**Acceptance:**
- A decision writes one wiki entry with the decision, its assumptions, its falsifier,
  and `outcome_status: pending`.
- An outcome-update flow records the actual result against an existing decision and
  moves it out of `pending`.
- The revisit query returns **both** the original decision **and** its recorded
  outcome for a decision that has one, and clearly marks those still `pending`.
- Re-running the same decision does not create duplicate entries.

---

## change-idt-005 — coach role
**Goal:** 5 · **Build-required** · **Agent:** general-purpose

**Half of goal 5 is already wired:** `hooks/hooks.json:170` registers a `reflector`
SubagentStop matcher routing reflection output through sycophancy-correction at
`strict` with a 2-rejection cap. Only the **coach** is missing — `agents/` has
`kbd-idea-critic`, `kbd-goal-evaluator`, `kbd-spec-reviewer`, `kbd-task-verifier`, and
no coach.

Build the coach as a separate agent that **cannot grade its own output**, following
the producer≠judge rule this pack enforces. The coach advances the user's plan; the
reflector evaluates progress; neither does both.

**Acceptance:** a coach agent exists in `agents/`; coach output is evaluated by the
reflector, never by the coach; the existing reflector hook is reused unmodified.

---

## change-idt-006 — harness delivery via `ui-surface`
**Goal:** 6 · **Library:** `cand-018` (adopt) · **Agent:** general-purpose

Skills emit `UiIntent`; `ui-surface` resolves the tier. Tier logic stays in one place.

**Tier 0 alone does not satisfy this goal.** Tier 0 text is a universal floor, but
"works on Codex or Kimi" means reaching **Tier 1**, which outside Claude Code is a
file-pair handshake: write `~/.prometheus/learn/ui/__ui_intent__.json`, poll
`__ui_response__.json` every 2 s with a 30 s timeout (`ui-surface/SKILL.md:96–104`).
That mechanism has never been exercised by an ideation flow and only works if the
harness polls.

**Acceptance (observable pass conditions):**
- On Claude Code, the flow emits a `UiIntent` and `detect-surface-tier.sh` reports
  `tier1`; the user-facing prompt appears.
- On **one** named non-Claude harness, running the flow **writes**
  `~/.prometheus/learn/ui/__ui_intent__.json` and **consumes** a
  `__ui_response__.json` placed there, completing the round trip within the 30 s
  timeout. Pass = the flow continues using that response. **Run it; do not assert it.**
- If the harness does not poll, record that as a **stated limit** in the change and
  fall back to Tier 0 — do not claim delivery.
- Tier 0 degradation is exercised by forcing `tier0_text` and confirming the flow
  still completes.

---

## change-idt-007 — idea fixtures prove the gate discriminates
**Goal:** 7 · **Build-required** · **Agent:** general-purpose

Committed fixtures: a **weak idea** must be BLOCKed and a **sound idea** must PASS,
both `verified-distinct`. Same domain and identical stated intent across the pair, so
the only variable is quality — the design constraint that made the skill/agent
fixtures meaningful last phase.

An **inversion fails the suite**. Bounded judge calls, on-demand and release-gate
only, following `run-fixture-suite.sh`.

Must also assert the goal-3 gate: commit-before-reveal cannot be bypassed.

**Acceptance:** 4 fixtures sort correctly against a live judge; an inversion exits
non-zero; zero assertions is a hard failure; judge calls stay under the ceiling.

---

## change-idt-008 — feature-gate `iroh-docs`
**Goal:** enables added scope · **Library:** `cand-003` (adapt) · **Agent:** general-purpose

`substrate/storage-provider/Cargo.toml:19` pins `iroh-docs` **unconditionally** with
`features = ["fs-store"]`. `iroh-docs` is not wasm-compatible, so the crate cannot
build for `wasm32` at all today.

One-line change; unblocks any future browser target. Native behaviour unchanged.
Also pin **iroh ≥1.0.2** — 1.0.2 fixed a critical relay DoS where one malformed
datagram from any client crashed an entire relay.

**Acceptance:** `cargo build` unchanged natively; `cargo check --target wasm32-unknown-unknown`
progresses past the `iroh-docs` failure; iroh floor is ≥1.0.2.

---

## change-idt-009 — record the fabric decisions
**Goal:** added scope · **Library:** `cand-001`, `cand-006`, `cand-007`, `cand-012` · **Agent:** general-purpose

Write the decisions to `docs/decisions/`, following the record shipped last phase.
**Design and record only** — the user agreed no cross-repo code this phase.

1. **iroh fabric-wide** for desktop/server/mobile. **Browser is relay-only by
   architectural necessity** — no UDP in the sandbox. Reject `iroh-webrtc-transport`
   (33 downloads, pinned pre-1.0 `^0.98.2`, repo 404s, absent from n0's own
   `TRANSPORTS.md`).
2. **Unify the two WIT worlds** — `uar:skill@0.1.0` and `knowme:plugin@0.1.0` — into
   `prometheus:component/*`. **This must be settled before any skill is ported**, or
   every skill is ported twice and parity is false by construction.
3. **`frf-transport-iroh`** as a `FederationBridge` adapter, beside
   `frf-bridge-matrix`/`frf-bridge-atproto`. Additive to FRF, no architectural change.
4. **Version invariants** the fabric-integration skill will verify: Loro minor
   aligned (FRF 1.13.1 ↔ this pack 1.13), wasmtime major aligned (46 in UAR *and*
   KnowMe), iroh ≥1.0.2, WIT world version pinned.

**Acceptance:**
- A decision record exists per item in `docs/decisions/` with alternatives and
  rationale.
- **Verifiable in this repo:** the decision record contains a `knowme_sync` block
  naming the external file, the commit SHA of know-me-system at the time of edit, and
  the SHA-256 of the guide after editing. A reviewer scoped to this repository checks
  that block; it does not need access to the other repository. The external edit is
  recorded evidence, not an unverifiable acceptance criterion.
- **No code lands** in flint-realtime-fabric, universal-agent-runtime, or
  know-me-system. Documentation only, per the scope agreed at analyze.

---

## Deferred to `mobile-skill-portability`

Recorded here so the split is deliberate, not drift:

- **`prometheus:component/*` WIT family** (build-required #4) — authoring the world
- **Mobile FFI bindings** (build-required #6) — this pack has no cdylib/staticlib and
  no uniffi; `frf-ffi` (uniffi 0.31.2) is the pattern to copy
- **`fabric-integration` skill** (build-required #3) — depends on the WIT decision
- Wiring `knowme_plugin_host` into `gen_ui_ffi`; on-device wasmtime proof;
  App Store **4.7.2** resolution; relay hardening (the MCP pool is fail-open today)

## Risks

| Risk | Mitigation |
|---|---|
| 007 needs a live judge; gateway may be down | `preflight-models.sh` first; suite reports "no gateway" rather than passing vacuously |
| 006's non-Claude Tier 1 is unexercised | Verify by running, not asserting. If the harness does not poll, record it as a limit rather than claiming delivery |
| 004 inherits `pk` ingest noise | De-duplication is an acceptance criterion, not a follow-up |
| Scope creep back toward fabric work | 008/009 are bounded: one line and one document. Anything larger belongs to the next phase |

---

## Unresolved review findings

Two adversarial rounds against a cross-model judge (`verified-distinct` both times).

**Round 1 — three CRITICALs, all valid, all fixed:**

| Finding | Fix |
|---|---|
| Prerequisite blockers omitted | Added the verified-state table at the top; both re-checked at plan time |
| change-004 persisted decisions but not **outcomes** | Added `outcome_status`, an outcome-update flow, and acceptance that a revisit returns both. This was the actual differentiator — writing decisions alone is the half every competitor has |
| change-001 **demonstrated** rather than **enforced** `verified-distinct` | Added a schema requirement plus a validator that rejects artifacts missing it or carrying a collision |

The second and third are the same mistake in two places: acceptance criteria that
prove one case can succeed rather than that the property always holds. Worth naming
because it is easy to repeat.

**Round 2 — one CRITICAL and four WARNINGs, all addressed:**

| Finding | Fix |
|---|---|
| change-009 acceptance sat outside this repository | Replaced with a `knowme_sync` block (external path, that repo's SHA, guide SHA-256) recorded **in this repo**, so a scoped reviewer can check it |
| change-003 "states confidence honestly" was unfalsifiable | Replaced with schema-required `confidence` and `what_would_change_this` fields, plus a required disconfirming item |
| change-006 rendering criteria ambiguous | Replaced with an observable round trip: writes `__ui_intent__.json`, consumes `__ui_response__.json` within 30 s, flow continues |
| change-002 `N` and "independence" undefined | Fixed in round 1's pass: **≥3** sets, independence asserted by inspecting dispatch inputs |
| change-008 named a non-existent agent | Changed to `general-purpose` |

**Standing limitation, not a finding against this plan:** `build-review-packet.sh`
builds `file_tree` with `find -maxdepth 2`, so files under `skills/<domain>/<skill>/…`
and anything in another repository are invisible to an artifact-mode packet. Several
WARNINGs reduce to that. Recorded as technical debt on the review tooling in
`assessment.md` §9 and carried here.
