# Assessment — ideation-and-decision-tools

> **Date:** 2026-07-30 · **Tool:** Claude Code (claude-opus-5)
> **Scope extension:** this assessment covers the seven seeded goals **plus** two
> areas the user added at invocation: (a) running these skills on mobile inside the
> Universal Agent Runtime, with KnowMe as the reference container; (b) P2P/remote
> execution so a phone retains full function.

---

## §1 Headline findings

Three findings change what this phase should build. Each is evidence, not preference.

**F-1 — Scoring an idea measures the wrong thing.** Si, Hashimoto & Yang (2025),
*The Ideation-Execution Gap* (arXiv 2506.20803): 43 experts spent 100+ hours each
executing randomly-assigned LLM vs. human ideas. LLM ideas rated *more novel*
before execution; after execution they dropped significantly on **every** metric
(novelty, excitement, effectiveness, overall, p < 0.05) and the ranking **flipped**.
A tool that outputs a score without execution is selling a number the best available
study says is wrong. **The terminal artifact must be execution, not a verdict.**

**F-2 — Persona panels do not produce diversity.** Chen et al. (2026),
*Diversity Collapse in Multi-Agent LLM Systems* (arXiv 2604.18005): multi-agent LLM
ideation exhibits structural coupling — agents synchronise and produce redundant
ideas *despite* architectural attempts at diversification. Combined with the
Multi-Agent Debate martingale proof already recorded in the seed, the "team of
personas" framing is contraindicated twice over. Diversity must be **enforced
mechanically** (independent generation, then pool), not assumed from persona prompts.

**F-3 — Group brainstorming is empirically discredited.** Mullen, Johnson & Salas
(1991) meta-analysis, 20 studies / 800+ teams: interactive brainstorming groups are
*significantly less productive* than nominal groups in both quantity and quality, and
the loss **grows with group size** (production blocking). Encoding brainwriting
(parallel independent generation → pool) is evidence-based; encoding a round-robin
session is encoding a known failure mode.

---

## §1a Carried-forward blockers — verified state

`goals.md` names two blockers that must be clear before this phase's goals are
reachable. Both were cleared during this session; verified at assess time rather
than assumed:

| Blocker | State | Evidence |
|---|---|---|
| Commit the previous phase (150 changed paths, new submodule) | **CLEARED** | `c88a05f` (223 files, 7152 insertions, `tools/openai-proxy` gitlink) and `ffc0bea`; `origin/main` behind 0 / ahead 0 |
| `scripts/update-skill-pack.sh --force` — stale plugin caches | **CLEARED** | `check-model-config.sh` reports **0 DRIFT** findings, down from 6. `kbd_require_producer_model` present in both the `.claude` and `.codex` 1.6.0 caches |

**Consequence for goals 1, 2 and 5:** the producer≠judge guarantee *is* now live in
installed copies. Before the cache refresh it was not — any creator running from a
plugin cache would have used a `kbd-model-resolve.sh` without the fail-closed guard.
That is why the blocker gated this phase, and why the state is recorded here with
evidence rather than carried as an assumption.

**Residual:** 13 uncommitted paths remain — this phase's own artifacts
(`assessment.md`, the review packet/findings, the KnowMe guide, seeded `goals.md`).
Normal in-flight state, not a blocker. The prior phase is fully committed and pushed.

## §2 Goal-by-goal gap analysis (the 7 seeded goals)

| # | Goal | State | Gap |
|---|---|---|---|
| 1 | Judge with find-problems mandate, not a round-table | **Substrate exists** | `adversarial-review` shipped last phase and is proven to discriminate live (27 assertions, 4 fixtures, `verified-distinct`). No ideation skill invokes it yet. |
| 2 | Every decision artifact carries `cross_model_check` | **Substrate exists** | `dispatch-judge.sh` records all three values; `build-review-packet.sh` has `--mode skill\|agent`. **No `--mode idea` / `--mode decision` packet exists.** |
| 3 | Automation-bias countermeasures | **NOT STARTED** | Nothing in the pack implements commit-before-reveal, confidence calibration, or deliberate friction. This is the highest-risk gap — see §3. |
| 4 | Persist decisions + outcomes in the Karpathy wiki | **Partial** | `pk` ingest/focus hooks fire (5,260 recorded fires). No decision schema, no outcome revisit. **Nothing in the surveyed market has this** — largest differentiator. |
| 5 | Coach/reflector as roles that cannot grade themselves | **Pattern exists, unapplied** | The producer≠judge rule and `kbd_require_producer_model` fail-closed guard are live **in installed copies as of the cache refresh verified in §1a** — not before it. No coach/reflector personas exist. |
| 6 | Claude Code + one non-Claude harness via `ui-surface` | **Substrate exists** | `ui-surface` resolves 3 tiers; `detect-surface-tier.sh` is the single tier-logic site. Untested for an ideation flow. |
| 7 | Fixtures: weak idea blocked, sound idea passes | **Pattern proven** | `tests/run-fixture-suite.sh` + `test-reject-cap-override.sh` (27 + 21 assertions) are the template. No idea fixtures. |

**Net:** goals 1, 2, 5, 6, 7 are *wiring* against proven substrate. Goals 3 and 4
are genuine new construction, and they are the two that carry the differentiation.

---

## §3 Competitive position (verified survey)

Surveyed: 21 commercial tools (Preuve, DimeADozen, IdeaBrowser, IdeaProof,
ValidatorAI, Verdikt, PainMap, VenturusAI, Idealyt, FounderPal, …), the MCP
registries, VoltAgent's 1,497-skill collection, Anthropic's official skills repo,
and `github.com/topics/claude-code-skills`.

**What exists as skills:** essentially one — `brainstorming` in obra/superpowers
(Socratic questioning, no formal methodology). Plus Mentor MCP (cross-model second
opinion via DeepSeek R1) and an Apify idea-validator MCP ($0.25/call, **explicitly
stateless**).

**Verified negative results** — searched and found nothing:

- **Zero** skills encoding any named methodology: TRIZ, SCAMPER, Six Thinking Hats,
  Disney method, Crawford slip, pre-mortem. The methodology space is empty.
- **Zero of 21** commercial tools persist state across sessions. All are one-shot
  report vendors.
- **Zero** tools track idea outcomes over time ("you scored this 8.2 six months ago —
  what happened?"). Given F-1, this is exactly the missing feedback loop.
- **Zero** commercial tools do adversarial cross-model judging. IdeaProof claims a
  "4-model ensemble" with no technical detail.
- **Zero** tools carry validation evidence forward into a build. Validation tools and
  scaffold tools are disjoint populations.
- **Zero** implement over-reliance countermeasures despite the harm literature (§4).

**Closest competitor:** `idea-factory` (MIT, Claude Code template) — one-line idea →
autonomous MVP via 7 agents with a 4-reviewer isolated-worktree gate. It **builds
without validating**; Preuve **validates without building**. Neither remembers.

> **Caveat on sources:** the two comparison tables (Preuve, DimeADozen) are
> vendor-authored and rank themselves #1. Tool inventory and pricing are roughly
> reliable; rankings are marketing. The survey covered these tools and registries, not
> the whole category.

**Methodology evidence grades** (encode honestly, or not at all):

| Method | Evidence |
|---|---|
| Brainwriting / nominal groups | **Strong** — meta-analytic (F-3) |
| Pre-mortem / prospective hindsight | **Moderate** — ~30% gain, but traces to essentially one 1989 study |
| SCAMPER, Six Thinking Hats | **Weak** — K-12/undergrad classrooms, small samples |
| TRIZ | **Plausible but unproven** — mostly self-reported or advocate-authored |
| Design thinking | **Folklore** — Roth et al. (2020): "no quantitative empirical evidence" it improves project performance |
| Disney method, Crawford slip | **None found** |

---

## §4 Automation bias — the governing risk (goal 3)

The harm literature is stronger than the benefit literature, and no AI ideation or
coaching product implements countermeasures:

- Microsoft Research (2025): confidence in AI was among the strongest predictors of
  whether knowledge workers engaged in critical thinking at all — **higher trust →
  less scrutiny**.
- Greater AI dependence associated with lower critical thinking, mediated by
  cognitive fatigue; **27.7%** of students showed degraded decision-making (PubMed
  41076923).
- Explainable AI *increased trust while promoting over-reliance*, producing **"False
  Confirmation"** errors — visible reasoning "may instead provide false assurance
  that errors have been checked for and ruled out."
- **No RCT evidence** that AI coaching improves goal attainment over non-AI tracking.

Consistent qualifier across sources: *guided, reflective* use supports critical
thinking; *unstructured substitution* erodes it. The evidence-aligned pattern is
**commit-before-reveal** — force the user to record their own judgement before the
system shows its own. For irreversible personal decisions (the relationship and
career questions named in the seed), this is the governing consideration.

---

## §5 Mobile / embedded portability

### §5.1 The portability number

| Category | Count | Mobile status |
|---|---|---|
| Prompt-only skills | **107 / 145 (74%)** | Run unchanged — no process needed |
| Script-bearing skills | **38 / 145** | Need a strategy |

Dependency profile of the 38: 13 shell-only, 15 toolchain (cargo/npm/python), 6
network, 3 git, 1 service. **Toolchain-dependent skills are the hard core** — they
are inherently desktop work (building code), not candidates for mobile execution.

**The ideation-critical skills are the portable ones.** All five
`adversarial-review` scripts need only **python3 + HTTP**:

- `git` appears only in **diff mode** (`build-review-packet.sh:176–181`)
- `cargo` is already opt-in and degrades gracefully (`:362–367`)
- The `--mode skill|agent` paths built last phase need neither

That is a genuinely mobile-viable envelope, and it is the exact surface goal 2 needs.

### §5.2 KnowMe is further along than expected

> **Provenance note.** Everything in this subsection was read directly from
> `/Users/gqadonis/Projects/know-me/know-me-system`, a **separate repository** at HEAD
> `8b373c8`. It will not appear in this repo's `file_tree`, so a reviewer scoped to
> this repository cannot verify it from the packet alone. File-and-line citations are
> given so the claims are checkable in that repo. The same applies to the
> `universal-agent-runtime` references in §5.3 (HEAD `563ecc2`), where this pack is
> itself a submodule at `crates/prometheus-skill-system`.

Verified in `/Users/gqadonis/Projects/know-me/know-me-system`:

| Asset | State |
|---|---|
| `knowme_plugin_host` | **wasmtime 46, component-model, async, `runtime` feature — no `cranelift`.** Capability-gated, deny-by-default WASI, fuel + memory + epoch limits, typed `HostDispatch`, SSRF-guarded `net_fetch`, precompiled `.cwasm` cache keyed by content hash + engine version |
| `openspec/specs/plugin-sandbox-host` | 10 ratified requirements incl. **Precompiled component cache** — the requirement that makes iOS viable (no JIT) |
| `host-uar.invoke` | Already a declared host interface — **the delegation seam** for "guest asks the runtime to do something", local or remote |
| `gen_ui_mcp` | **stdio is an opt-in native-only feature** — remote MCP transports are already the mobile default. Same architectural split this pack needs |
| `docs/knowme-builder-extensibility-spec.md` | Per-target data plane verified; `.pxt` package format; unified `prometheus:component/*` WIT family; capability negotiation **at install time, before any code runs** |

**Two blocking gaps:**

1. **`knowme_plugin_host` is orphaned** — no crate depends on it. `gen_ui_ffi` (the
   mobile FFI crate) does **not** include it, so the WASM host ships nowhere today.
2. **KnowMe has no P2P** — no iroh, no WebRTC, no Loro in any crate. It reaches
   services over plain reqwest/SSE. The P2P bridge is new work there, whereas
   `sovereign-sync` in this pack already has iroh 1.0 + iroh-gossip + Loro.

The extensibility spec states the iOS constraint plainly — *"iOS cannot run Postgres:
no fork, no JIT"* — and names the **entity layer, not SQL, as the portability
boundary**. That is the same shape the skill layer needs: the *contract* is portable;
the *execution substrate* differs per tier.

### §5.3 The tier model this implies

`ui-surface` already proves the pattern for rendering (Tier 0 text → Tier 1
AskUserQuestion → Tier 2 MCP App). The execution analogue:

| Tier | Substrate | Skills served |
|---|---|---|
| **E0 — prompt-only** | Any harness, no execution | 107 skills, unchanged, everywhere |
| **E1 — WASM component** | `knowme_plugin_host` (wasmtime 46, precompiled `.cwasm`) | Deterministic script logic recompiled as components; python3-and-HTTP skills are the first candidates |
| **E2 — remote invoke** | `host-uar.invoke` → paired desktop / cloud node | Toolchain-dependent skills that cannot ever run on a phone |

E0 and E2 need no new runtime.

**Correction after research: E1 is NOT where the leverage is.** My initial read was
that the WASM host being pre-built made E1 the obvious win. The evidence says
otherwise, and E1 moved *further out*, not closer:

- **~10× interpreter slowdown.** iOS forbids JIT, so wasmtime runs Pulley
  (interpreter) or a precompiled `.cwasm` loaded read-only. There is **no
  certification of WASIp2-on-Pulley-on-iOS** — that is inference from feature-parity
  language, not a shipped result.
- **No documented WASI-preopen-on-iOS integration** for any runtime, and **no
  confirmed shipping mobile production user of wasmi**.
- **An unresolved App Store policy question.** Whether guideline **4.7.2** permits a
  WASM plugin host that exposes native APIs has **no precedent in either direction**.
  This is the single highest-risk unknown and needs Apple contact before committing.
- Published binary-size figures are all desktop; arm64 mobile sizes must be measured.

E1 remains the right *eventual* on-device answer for deterministic script logic, but
it is a research bet, not a delivery path. **E2 preserves every script at zero porting
cost** and matches three independent vendor convergences. That is the tier to build.

### §5.4 P2P / remote execution — value and risk

**Value is real and specific.** The user's framing is correct: with one internet-
connected machine executing on their behalf, a phone retains *full* function for
skills that have no mobile answer at all. That converts "38 skills don't work on
mobile" into "38 skills work anywhere you have a paired machine". For a product whose
promise is *the same runtime and skills wherever you go*, this is the difference
between a companion app and the actual product.

**The research inverts the topology recommendation. Relay is primary, not fallback.**

1. **P2P hole-punching fails specifically where phones are.** Richter et al.
   (IMC 2016, arXiv:1605.05606) measured **>90% of cellular ASes use CGNAT**, and
   **~40% of cellular CGNAT is symmetric** — major US carriers among them. When both
   peers sit behind endpoint-dependent-mapping NATs, birthday-paradox probing succeeds
   ~**0.01%** of the time. iroh's own documentation names the case: *"Where it fails
   (some corporate firewalls or cellular networks) iroh automatically falls back to
   the relay."* The widely-quoted 70–90% hole-punch figures are all **desktop/server
   populations** — the rigorous 70% ± 7.1% number (arXiv:2510.27500) is IPFS nodes,
   and that paper explicitly disclaims the CGNAT inference.
   *(Caveat: the cellular data is 2016 with no modern replication found. No 2020s
   measurement segmented by cellular-vs-WiFi from actual handsets exists publicly —
   the single biggest gap in the literature. If mobile P2P becomes strategic, that
   experiment is worth running in-house.)*

2. **The NAT binding and the battery are the same problem.** Cellular CGNAT median
   mapping timeout is **65 s**, so holding a binding needs keepalives every 15–30 s.
   RFC 8085 sets a 15 s floor and warns keepalive frequency "can become the
   determining factor that governs power consumption." Both mobile OSes resolve that
   conflict against you. Apple DTS, verbatim: *"If you need to keep your network
   connections alive indefinitely, that's not possible given the iOS multitasking
   architecture."* The mechanism is **suspension, not backgrounding** — and the Xcode
   debugger prevents suspension, so this bug looks fixed in development.

3. **Three vendors independently converged on cloud relay within four months** —
   Anthropic's Claude Code Remote Control (Feb 2026: *"outbound HTTPS requests only
   and never opens inbound ports"*), OpenAI Codex mobile (May 2026: *"a secure relay
   layer keeps trusted machines reachable"*), and Cursor. **None use P2P.** The
   reasons compose: no-inbound-ports is the enterprise selling point, cellular CGNAT
   breaks P2P exactly where phones are, backgrounding makes a phone-side P2P endpoint
   untenable, push requires a server intermediary anyway, and relay latency is
   invisible against multi-second agent turns.

**What this pack already has is the right answer.** iroh QUIC is end-to-end encrypted
between node IDs — the relay forwards ciphertext it cannot read. `sovereign-sync` is
therefore already a **blind relay**, which is the differentiator the incumbents lack:
Anthropic stores transcripts server-side and structurally excludes zero-data-retention
orgs. Keep `p2p.rs` as-is; stop describing it as "no intermediate server," and start
describing it as *a relay that cannot read your data*. Direct QUIC becomes an
opportunistic optimisation, not the architecture.

**Design for episodic sessions, not a persistent link:** durable state + fast
idempotent resume + push as doorbell + foreground service only while the user is
present.

> ⚠️ **Trap to budget for:** FCM deprioritises high-priority messages when it *"detects
> a pattern in which messages don't result in user-facing notifications"* — evaluated
> per-install over 7 days. A silent push-to-wake channel is exactly that pattern, and
> it degrades **silently, for a subset of users**. Post a genuine user-visible
> notification on wake.

2. **The MCP forwarding path is fail-open.** `mcp_client_pool.rs:120` —
   `allowed_tools` **empty → all tools allowed**. Acceptable for a loopback daemon;
   unacceptable the moment a phone can trigger execution on a desktop. Remote
   execution needs deny-by-default allow-lists, capability tokens, pairing, and replay
   protection **before** the transport, not after.

3. **No FFI bindings exist.** Neither `sovereign-client` nor `sovereign-sync` exposes
   `uniffi`/`flutter_rust_bridge`, and neither declares a `cdylib`/`staticlib` target.
   `sovereign-client` is pure reqwest/SSE and would *compile* for mobile today, but
   nothing binds it to Dart/Swift/Kotlin.

**Cheapest first step, already half-built:** `SURREAL_MEMORY_URL` already points the
memory client at a remote host. Remote *services* over HTTPS is a working pattern
today and needs no P2P at all. P2P is the upgrade for NAT-hostile networks and for
"my desktop, not a server" — not the prerequisite.

---

## §6 KnowMe as the reference container

The user's instruction: KnowMe is the reference future container, this skill set will
ship **inside** that app, and this phase must produce a guide in KnowMe's `docs/`
describing how to use these skills to build it there.

Constraints that guide must respect (all verified, all non-negotiable in KnowMe):

- **`gen_ui_core` invariant** — all networking/LLM/persistence in Rust. A skill may
  not open a socket from Dart or React.
- **Layering (Rule 16, enforced by `audit.sh`)** — Flutter: Widget → provider →
  Repository → FFI. React: Component → Hook → Store → invoke(). Stores are the *only*
  invoke() layer.
- **Capability negotiation at install time** — the installer refuses a component whose
  imports exceed the host's surface, before any code runs.
- **`.pxt` package format** — signed manifest, SBOM, SHA-256 per file, DID + cosign.

**Deliverable:** `docs/prometheus-skills-integration.md` in `know-me-system`, written
in this phase, covering the E0/E1/E2 tier model, which skills land in which tier, how
`host-uar.invoke` carries E2, and what must be true before `knowme_plugin_host` joins
`gen_ui_ffi`.

---

## §7 Open questions for analyze/plan

**Answered by research (no longer open):**

- ~~Can a P2P connection survive iOS backgrounding?~~ **No.** Apple DTS is explicit;
  the mechanism is suspension. Design for episodic sessions.
- ~~Relay-fallback rate on cellular?~~ **Relay is the common case on cellular**, not
  the exception. >90% CGNAT, ~40% symmetric. Make relay primary.

**Still open:**

| # | Question | Why it blocks |
|---|---|---|
| OQ-1 | Does App Store guideline **4.7.2** permit a WASM plugin host that exposes native APIs? | **No precedent in either direction.** Highest-risk unknown; needs Apple contact before any E1 commitment. Blocks E1 on iOS entirely. |
| OQ-2 | Real arm64 binary-size delta for wasmtime 46 in the mobile bundle | Published figures are desktop-only. Must be measured, not estimated. |
| OQ-3 | Auth/crypto design for the blind relay: capability tokens, pairing, replay protection | `mcp_client_pool.rs:120` is fail-open today. This must land **before** any transport work. |
| OQ-4 | Rewrite the 5 `adversarial-review` scripts as a WASM component, or call them via E2? | **Leaning E2** after research. The 15 toolchain skills have no E1 answer regardless; doing both is waste. |
| OQ-5 | Which methodologies to encode, given §3's evidence grades? | Encoding design thinking would ship folklore as a skill. |
| OQ-6 | Does outcome-tracking (goal 4) need a new entity type, or reuse the `pk` wiki schema? | Determines whether this touches KnowMe's entity layer and sync classes. |
| OQ-7 | **Composition risk across skills** — the research names this as the largest security work item, larger than any single sandbox | 145 skills invoking each other through a remote seam is a different threat model than one sandboxed skill. |

**Staleness flags carried forward:** App Review Guidelines change without notice —
re-verify 2.5.2 / 4.7.2 before submission. The cellular CGNAT data is 2016. WASI
Preview 3 is landing in Wasmtime 46+ and may change the component-model story
mid-build. Android Platform root certificate rotates **February 2026** if direct Key
Attestation is used.

---

## §9 Unresolved review findings

Two adversarial rounds ran against a cross-model judge (`verified-distinct` both
times). Round 1 returned 2 CRITICALs, both valid — the assessment asserted the
producer≠judge guard was "live" while `goals.md` carried an unresolved cache-drift
blocker, and never assessed the blockers at all. Both are fixed: §1a now verifies
each blocker with command output, and goal 5 is scoped to "as of the cache refresh
verified in §1a".

Round 2 returned 2 CRITICALs that are **correct observations with an incorrect
target**, so they are recorded rather than actioned:

| Finding | Disposition |
|---|---|
| "Blockers treated as cleared using evidence not present in the packet" | **Accurate about the packet, not the claim.** §1a cites `git log`, `check-model-config.sh` (0 DRIFT), and per-cache `grep` counts. Command output is not a packet field, so a judge cannot re-run it. The evidence is real and reproducible; the packet cannot carry it. |
| "Goal 6 rests on `ui-surface` / `detect-surface-tier.sh` not present in the file tree" | **Tooling defect, not a false claim.** All four cited files exist (verified by `test -f`). The packet's `file_tree` is built with `find -maxdepth 2` (`build-review-packet.sh:150`), and every skill lives at `skills/<domain>/<skill>/…` — **depth 3+**. Artifact-mode review therefore *cannot* verify any skill-file claim, ever. |

**TD carried to plan: `--mode artifact` packets are structurally blind to skill
files.** Any assess or plan artifact citing a skill path will draw an unverifiable-
claim finding regardless of correctness. Options: raise `maxdepth` for artifact mode,
add a `cited_paths` field the builder resolves and stamps with existence, or scope
artifact review to prose-only claims. This is a defect in the gate this pack shipped
last phase, found by using it — worth fixing before it trains reviewers to discount
real findings.

The three WARNINGs (external-repo claims, scope expansion) are addressed by the
provenance note in §5.2 and the explicit two-phase split in §8.

---

## §8 Scope warning

The seeded goals are 7. The user's invocation adds mobile portability, P2P remote
execution, and a KnowMe integration guide — each of which is plausibly its own phase.

**Recommendation for plan:** treat this as **two phases**, not one.

- **This phase** — ideation skills against existing substrate (goals 1–7), with the
  E0/E1/E2 tier model *designed and documented* (including the KnowMe guide) but only
  **E0 + E2-over-HTTPS implemented**. That ships a working ideation capability on
  desktop and mobile using remote services, which already works.
- **Next phase** — `mobile-skill-portability`: wire `knowme_plugin_host` into
  `gen_ui_ffi`, prove wasmtime on-device, harden the remote-execution allow-list, add
  FFI bindings, and measure P2P.

Attempting both at once risks the pattern this pack keeps hitting: the interesting
architecture gets built and the *evidence* that it works does not.
