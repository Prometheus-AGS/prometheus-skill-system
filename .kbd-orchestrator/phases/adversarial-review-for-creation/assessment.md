# Assessment — adversarial-review-for-creation

**Phase:** `adversarial-review-for-creation`
**Stage:** assess
**Date:** 2026-07-30
**Preflight:** `status: ok`, gateway `http://localhost:8181/v1`, judge `kbd-judge`,
critic `kbd-critic`, 0 config defects — a real cross-model judge is available.

Scope was split at the user's direction: **Part 1** assesses this phase's seven goals
(concrete, verified, buildable now). **Part 2** assesses adversarial review as a *product*
surface, with web research, and recommends it become its own phase rather than being
absorbed here.

---

## Part 1 — Creation gating (this phase)

### The verified gap

| Surface | Gated by a cross-model judge? |
|---|---|
| `kbd-assess` | ✅ |
| `kbd-analyze` | ✅ |
| `kbd-plan` | ✅ |
| `kbd-execute` | ✅ |
| **`pmpo-skill-creator`** | ❌ **zero references** |
| **`native-agent`** | ❌ **zero references** |

Verified by `grep -rl "adversarial-review"` over each skill tree, re-run at vet time:
`skills/process/pmpo-skill-creator/` → **0 files**, `skills/process/native-agent/` → **0
files**, versus four hits under `skills/process/kbd-process-orchestrator/skills/`
(`kbd-assess`, `kbd-analyze`, `kbd-plan`, `kbd-execute`). `build-review-packet.sh` accepts
`--mode diff|artifact` only. The pack subjects its own
*plans* to an adversarial judge but ships **generated skills and agents ungated** — the
artifacts most likely to be reused, hardest to eyeball, and most costly to get wrong,
because a bad generated skill propagates into every project that installs it.

### Current state per goal

| # | Goal | State | Notes |
|---|---|---|---|
| 1 | Wire review into skill creator's Reflect | **Not started** | `reflect.md` has an 11-step checklist and a weighted ≥95% gate, but every step is model-honoured instruction. `validate-skill.sh` implements ~steps 1–4 and outputs text, not the JSON report `reflect.md` claims. |
| 2 | Wire review into native-agent generation | **Not started** | Generation ends at `cargo check` + `npm install`. Nothing judges *quality* of the emitted workspace. |
| 3 | Artifact-mode packets for the two new kinds | **Not started** | `build-review-packet.sh` supports `diff` and `artifact` modes over `assessment.md`/`analysis.md`/`plan.md`. A generated SKILL.md tree and a Cargo workspace are neither. |
| 4 | Enforce `KBD_PRODUCER_MODEL` at creator entry | **Partial** | The variable is now honoured and `PRODUCER_UNKNOWN` warns loudly (fixed 2026-07-30), but neither creator exports it. |
| 5 | Blocking on CRITICAL, 2-rejection cap | **Infrastructure exists** | `check-findings-sycophancy.sh` already implements the cap; not invoked by either creator. |
| 6 | Promote sycophancy pass to an enforced gate | **Not started** | `execute.md:229` and `reflect.md:118` mandate it; `validate-skill.sh` implements none of it. |
| 7 | End-to-end proof with deliberately flawed inputs | **Not started** | No fixture exists. This is the goal that makes the others falsifiable. |

### Assets that already exist (do not rebuild)

- `dispatch-judge.sh` — REST gateway dispatch, real `isolation_mode`, explicit HTTP status
  handling, collision fallback judge→critic. Verified working end-to-end this session.
- `check-findings-sycophancy.sh` — the anti-theatre gate with its 2-rejection cap.
- `kbd-model-resolve.sh` — role resolution + `kbd_complete`.
- `findings.schema.json` — now accepts `rest-gateway:<url>`, `producer_model`,
  `cross_model_check`; still rejects the retired literal and garbage.
- Four working precedents in `kbd-assess/analyze/plan/execute`.

**The work is wiring, not invention.** That materially lowers the risk of this phase.

### Gaps and open questions for analyze/plan

1. **What is the review packet for a generated *skill*?** A skill is a tree, not a file.
   Candidate: `SKILL.md` + frontmatter + script inventory + cross-reference map + the
   original intent. Needs a decision.
2. **What is it for a generated *agent*?** A Cargo workspace is far past a judge's context
   budget. Candidate: `agent.toml` + `system_prompt.md` + crate manifest + the generation
   spec — not the source.
3. **Blocking vs advisory — flagged by the judge as contradicting goal 2.** Goal 2 requires
   an agent workspace be "reviewed **before it is declared ready**". An earlier draft here
   recommended "advisory for agents", which conflicts with that. Corrected position:
   review is **always blocking on the readiness declaration** — generation may complete and
   the workspace may exist, but it is not reported as ready until findings are written and
   no CRITICAL remains. What is negotiable is whether a CRITICAL *deletes* the workspace
   (no) or *withholds the ready signal* (yes). Analyze must ratify this wording.
4. **Cost.** Every generation gains a frontier-class call. `pmpo-skill-creator` already
   loops up to 3× — a naive wiring triples judge cost. Review the *final* artifact only.
5. **Who reviews the reviewer?** Deliberately out of scope; note it.

### Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Judge unavailable → creators silently skip the gate | **High** | Record `harness-native`/`skipped` in output; never pass silently |
| Packet exceeds context for agent workspaces | Medium | Manifest-level packets, not source |
| Gate becomes theatre (always PASS) | Medium | Goal 7's flawed-input fixture is the control |
| Adds latency to an interactive flow | Medium | Review the final artifact only, never each loop iteration; run it concurrently with `cargo check` where possible |

---

## Part 2 — Adversarial review as a product surface

Assessed with targeted web research. **The evidence is genuinely mixed, and the headline
finding contradicts the strong form of the premise.** Reporting it plainly is the point of
a sycophancy-corrected assessment.

### What the research actually says

> **Citations.** Each source below is given with a resolvable identifier so analyze can
> re-verify before planning on it. Findings marked *proof* are formal results; the rest are
> empirical and should be treated as directional.
>
> - `openreview.net/forum?id=iUjGNJzrF1` — Choi, Zhu & Li, NeurIPS 2025 spotlight
> - `arxiv.org/html/2607.09099v1` — L-MAD
> - `nature.com/articles/s41598-026-42705-7` — Kraidia 2026
> - `mdpi.com/2227-9709/12/4/135` — human–AI collaboration review (over-reliance)
> - `sciencedirect.com/science/article/pii/S2451958826001764` — AI-overdependence
> - `blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/` — MCP Apps GA
> - `community.openai.com/t/.../1371022` — ChatGPT 5000-token tool-schema limit


**Debate alone does not improve correctness.** *Debate or Vote* (Choi, Zhu & Li, **NeurIPS
2025 spotlight**) disentangles Multi-Agent Debate into majority voting and inter-agent
debate across seven NLP benchmarks:

> Majority Voting alone accounts for most of the performance gains typically attributed to
> MAD … we prove that it induces a **martingale over agents' belief trajectories, implying
> that debate alone does not improve expected correctness** … simple ensembling methods
> remain strong and more reliable alternatives in many practical settings.

This is a proof, not a preference. It does **not** invalidate our design — our judge is a
*different model reviewing a producer's output* (an ensemble-with-asymmetry), not N agents
debating to consensus. But it directly contradicts "a team of personas debating will reach
better answers," which is the intuitive version of the persona-team idea.

The same paper points at what *does* work: *"targeted interventions, by biasing the belief
update toward correction, can meaningfully enhance debate effectiveness."* A judge with an
explicit find-problems mandate is exactly such a bias. Our existing design is on the right
side of this evidence; a naive persona round-table is not.

Corroborating: *L-MAD* (arXiv 2607.09099) finds debate structures improve on single-agent
baselines by **up to 8%** — real but modest. A Nature paper (Kraidia 2026) documents
*"persuasion driven adversarial influence"* — a failure mode where a confident wrong agent
moves the group.

**Automation bias is the dominant risk for the personal-decision use case.** The
human-AI-collaboration literature is consistent and unfavourable to naive coaching:

- Explainable AI *increased trust but risked promoting over-reliance*, producing **"False
  Confirmation" errors** — making reasoning visible "may instead provide false assurance
  that errors have been checked for and ruled out."
- Documented **skill degradation** and de-skilling under high-control configurations.
- "AI-overdependence and human cognitive decline" (ScienceDirect 2026) catalogues hazards.

For *"should I become a pro athlete / run a triathlon / take up bodybuilding"* and
especially **personal relationship problems**, this is the governing consideration. A
confident persona panel is a machine for manufacturing false confidence about
irreversible personal decisions.

### The market

Idea-validation is a **crowded, commoditised category**: ideaproof.io ("validate in 120
seconds", "50+ criteria", multi-model analysis, 2,345+ founders), plus roundups testing
9–12 competing tools. Generic "validate my startup idea" is not a differentiator.

**What the specific tools I checked do not have** — this is narrowed deliberately; I
surveyed ideaproof.io plus two 2026 roundups (9 and 12 tools), not the whole category, and
did not audit each competitor's roadmap:

| Their offering | Ours |
|---|---|
| One-shot score, session ends | Karpathy loop — the decision and its outcome persist |
| Confident answer | Sycophancy-corrected answer, structurally |
| Opaque single model | `cross_model_check: verified-distinct` recorded in the artifact |
| No learning | Feynman loop with a mastery criterion that rejects self-reported fluency |

On that limited sample the differentiator holds, but treat it as a hypothesis for analyze
to test against named competitors rather than a settled finding.

The defensible product is **not** "AI validates your idea."  It is **"a system that
remembers what you decided, what happened, and refuses to flatter you"** — longitudinal
accountability, not a verdict.

### We are closer than expected

Six ideation-adjacent skills already ship:

| Skill | Role |
|---|---|
| `ideation-mindmap` | divergent generation |
| `validate-idea` | convergent gating |
| `kbd-idea-critic` | **already a separate-model critic with a 4-dimension rubric** (`feasibility`, `pain_addressed`, `stack_fit`, `buildability`) |
| `zeespec-interrogator` | 60-question pre-spec interrogation with NO-GO verdicts |
| `learn-goal` / `feynman-loop` | learn-what-you-don't-know, with a hostile mastery criterion |
| `pmpo-evolver` | strategy router incl. strategic dreaming |

`kbd-idea-critic`'s docstring already states the principle: *"the idea that proposed the
idea should never also grade it."* **The persona-team product is largely an assembly and
UX problem, not a research problem.**

### Delivery constraints (verified)

- **MCP Apps shipped** as an official MCP extension (2026-01-26), *"available today both on
  web and desktop"* for Claude — interactive UI from a tool, which is what a persona panel
  needs.
- **ChatGPT enforces a hard 5000-token limit** on *all* tool schemas combined. A
  multi-persona server will breach this unless tools are few and terse. This is a real
  design constraint on cross-harness parity.
- Claude Desktop `.dxt` Desktop Extensions give one-click install.
- Known Claude Desktop limitation: many-tool servers degrade discoverability — the same
  catalog-budget problem this pack already hit with Codex.

### librefang integration is real infrastructure

Verified in `/Users/gqadonis/Projects/references/librefang`:

- `librefang-kernel/src/hooks.rs` → **`ExternalHookEvent`**
- `librefang-kernel/src/event_bus.rs` → **`Event`**
- `librefang-kernel/src/session_lifecycle.rs` → `SessionLifecycleEvent`
- `librefang-api/src/webhook_store.rs` → `WebhookEvent`
- `librefang-uar-spec` → parses the 15-section `UAR-AGENT-MD` format **without a running
  UAR**, and translates bidirectionally to librefang's native `AgentManifest`

The "with or without the UAR" requirement is already satisfied by design. Coach/reflector
events emitted as `ExternalHookEvent` is a supported path, not a wish.

### Honest verdict on the ideation product

**Worth building, in a narrowed form.**

Supported by evidence:
- A **judge with a find-problems mandate** — the intervention the NeurIPS paper says works.
- **Longitudinal memory** of decisions and outcomes — nobody in the category has it.
- **Structural anti-sycophancy** — directly counters the documented "False Confirmation"
  failure.

Not supported:
- **A debating persona round-table.** Proven not to improve expected correctness; adds cost
  and a persuasion-cascade failure mode. Use asymmetric roles (producer → critic → judge),
  which is what we already have.
- **Confident guidance on irreversible personal decisions.** Relationships and
  career-defining bets are where automation bias does real harm. Recommend the coach
  surface *structure the user's own reasoning and record it*, and explicitly decline to
  render verdicts on relationships and health.

### Recommended Next Phase

**`ideation-and-decision-tools`** — after this phase closes. Candidate goals:

1. `ideation-panel` skill composing the six existing skills into an asymmetric
   producer → critic → judge flow (**not** a debating round-table).
2. `coach` and `reflector` personas writing to the Karpathy wiki, so decisions and their
   outcomes are queryable months later.
3. Decision records: commitment, rationale, disconfirming evidence, review date.
4. `ExternalHookEvent` emission into librefang's kernel event bus, UAR-optional.
5. MCP Apps packaging for Claude Desktop; a deliberately minimal tool surface to stay under
   ChatGPT's 5000-token schema budget.
6. An explicit **refusal boundary** for relationship and health decisions, with the
   automation-bias rationale documented.
7. A falsifiable evaluation: does a user's decision quality or calibration improve? If we
   cannot measure it, we are shipping the thing the research warns about.

---

## Summary

- **Part 1 is low-risk, high-value, and mostly wiring.** Every component exists and is
  verified working; the phase connects them and proves the connection with a flawed-input
  fixture.
- **Part 2 is a real opportunity with a real trap.** The evidence supports a
  memory-backed, sycophancy-corrected, asymmetric-judge decision system. It does **not**
  support a persona debate club, and it actively warns against confident AI guidance on
  irreversible personal decisions.
- **The honest headline:** the strongest version of the user's idea is not "a team of
  personas argues about your idea." It is *"a system that will not let you forget what you
  decided, what you believed at the time, and whether it worked."*

## Open questions for analyze

1. Packet shape for a generated skill tree and an agent workspace.
2. Blocking vs advisory per creator (recommendation: skills blocking, agents advisory in v1).
3. Cost ceiling — review only the final artifact, or every loop iteration?
4. Does `validate-skill.sh` grow the sycophancy gate, or does the creator call the existing
   `check-findings-sycophancy.sh`?
5. Should Part 2 reuse `kbd-idea-critic` directly, or generalise it out of KBD?

---

## Adversarial review record

Vetted by a cross-model judge, per this phase's own subject matter.

| Field | Value |
|---|---|
| judge / producer | `kbd-judge` (gpt-5.6-sol) / `claude-opus-5` |
| isolation_mode | `rest-gateway:http://localhost:8181/v1` |
| cross_model_check | **`verified-distinct`** |
| verdict | PASS — 0 CRITICAL, 4 WARNING |

All four WARNINGs were addressed in this revision rather than carried forward:

1. **Repo claims not traceable from the packet** → grep results and mode list inlined above.
2. **Web claims unverifiable** → resolvable identifiers added for all seven sources.
3. **Market-uniqueness overstated** → narrowed to the sample actually surveyed and demoted
   to a hypothesis for analyze.
4. **"Advisory for agents" contradicted goal 2** → the judge was right; position corrected
   to blocking-on-readiness, with the negotiable part named explicitly.

Finding 4 is the one worth noting: the judge caught a real internal contradiction between
the assessment's own recommendation and the phase goal it was assessing against. That is
precisely the class of error a same-model self-review misses.
