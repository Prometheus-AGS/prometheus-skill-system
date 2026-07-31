# PAGS-SPEC-PSP-IP-002: The Skill Router — Inverting the Instruction Plane
### A specification for PSP-owned skill selection, injection, and activation observability across coding-agent harnesses and the Universal Agent Runtime
*Rev 1.1 (spec for build) — Travis James, Prometheus AGS — supersedes the C1/C2/C3 definitions in PAGS-SPEC-PSP-IP-001 rev 1.1*

**Rev 1.1 changes vs Rev 1.0**
- (a) Routing key generalized from free-text phase intent to the composite `(phase, change_id)`, with phase affinity/exclusion added to the registry schema as a **derived** field — §3.2, §4.1, §4.2.
- (b) The S-01–S-08 sycophancy isolation gate **promoted from an authoring note to a first-class architectural requirement**, enforced by phase exclusion — §4.8.
- (c) New §5: Capability Gap Records and the out-of-band acquisition pipeline. Runtime skill installation from untrusted sources is declared an explicit **non-goal**.
- (d) C3 compliance rollup relocated from the phase boundary to the **OpenSpec archive transition** — §4.10.
- (e) Cowork added to the harness matrix as an **uncharacterized dispatch target** pending M0 — §4.4.
- (f) Anthropic Tool Search explicitly ruled out as a *selection* mechanism and retained only as *ergonomics* for `request_skill` — §4.5.

---

## TL;DR

- **The inversion is worth building, but the confident wins are narrow and structural, not headline efficacy.** PSP takes ownership of skill *selection* (out-of-harness intent classification over full skill bodies) and *observation* (a per-goal Activation Ledger) across all harnesses plus UAR-native, guaranteeing that load-bearing skills are **present in context deterministically** — which no harness does today, because every harness routes on frontmatter descriptions only, under an opaque token budget, with silent eviction.
- **Presence is not compliance, and net task-outcome improvement is genuinely uncertain until measured.** Injection guarantees instructions are in context; attention, prompt position, competing instructions, and RL-trained harness disposition still shape adherence. M0 measurement precedes any efficacy claim; the transitional three-mode instrumentation earns the claim from ledger data rather than asserting it.
- **The one *correctness* argument — not merely an efficiency argument — is phase exclusion.** The S-01–S-08 gate currently isolates the critic on the *conversation* axis (no generation history) but not on the *instruction* axis (generation-class skills may still be in the prompt). A critic running with the builder's instructions present is not a clean critic. Phase-scoped routing with **active exclusion** makes AC-08 isolation a structural property of the instruction plane rather than a convention. This alone justifies the routing-key change.
- **The enabling mechanics are verified per harness and are uneven.** Claude Code has a real `--append-system-prompt` (append-only, `--print` mode); Codex has **no append flag at all** (must ride `developer_instructions` / `model_instructions_file` under a 32 KiB AGENTS.md cap); OpenCode exposes `session.prompt` + plugin hooks; Kimi/MiniMax expose `--skills-dir` / `--agent-file` + MCP. These asymmetries, plus the volatility of undocumented flags, are the primary execution risk.
- **Runtime acquisition of skills from the open internet is out of scope, deliberately.** Gaps are *recorded*, not patched in-flight. The legitimate dynamic path is a **signed registry** (entitlements-as-Verifiable-Credentials), not a runtime installer — §5.

---

## 1. Executive Summary

**The inversion thesis.** Today every coding-agent harness — Claude Code, Codex CLI, OpenCode, Kimi Code, MiniMax CLI ("MMX") — owns skill selection through an opaque, budget-constrained, probabilistic algorithm that matches the user's turn against skill *frontmatter descriptions only*, never the skill body, and silently drops skills when a context budget overflows. PSP currently sits *downstream* of this: we author ~140 skills, install them into harness directories, and hope the harness surfaces the right one. This spec inverts that relationship. PSP becomes the **instruction plane of record**: it performs its own out-of-harness skill selection, plans injection per harness, injects load-bearing skill bodies directly into the harness's prompt-assembly surface, serves a `request_skill` callback for emergent mid-phase needs, and logs every selection decision to an Activation Ledger under the goal DID. Harness skill directories become **projections** of the PSP registry, not sources of truth.

**What changes at Rev 1.1.** The Skill Router is Plane-2.5 inside the existing three-plane goal architecture (portable Goal Contract JSON → thin in-harness `psp-goal-runner` compliance skill → out-of-harness Rust Goal Supervisor in UAR). Rev 1.1 sharpens *when* and *on what key* routing fires: the routing key is the pair `(phase, change_id)` — the kbd/PMPO phase supplies a **declared categorical prior** over cognitive mode, and the OpenSpec change supplies **scope**. Full re-routing binds to fresh-context boundaries only (phase transition, change transition); between boundaries, `request_skill` is the sole additive channel. Native `/goal` primitives in Claude Code (v2.1.139+, released May 12 2026) and Codex remain inner accelerators; the outer Supervisor stays contract-of-record.

**Expected effect (honest uncertainty ranges).** For a ~140-skill library on Claude Code's default 1% listing budget, the majority of skills are silently non-triggering at any given moment (the budget math in §2.1 is unambiguous; the exact fraction is an M0 measurement against the actual context window). The Router eliminates that failure mode by construction: selected skills are present with 100% reliability, not surfaced probabilistically. But **presence is not compliance.** We expect the dominant failure mode to shift from "the right skill was never in context" — a recall failure the harnesses own and hide — to "the right skill was in context but under-attended," an adherence failure shaped by position, instruction tension, and disposition. Net task-outcome improvement is uncertain and must be measured per harness before any efficacy claim. No headline percentage is asserted here; the Activation Ledger will produce one or it won't. The confident claims are narrow and structural: **deterministic presence**, **complete observability**, and — new at Rev 1.1 — **structurally enforced critic isolation**.

---

## 2. Problem Statement (with evidence)

### 2.1 Harness selection is opaque, budget-bound, and silently lossy

**Claude Code (verified).** Skill selection routes on the frontmatter `description` field only; the model never reads the body to decide. The skill listing sent to the model has a hard character budget governed by `skillListingBudgetFraction`, introduced quietly in **v2.1.129** (ClaudeFast: *"Claude Code 2.1.129 added skillListingBudgetFraction, silently dropping skills past 1% of context"*), defaulting to `0.01` (1% of the context window). A companion setting `skillListingMaxDescChars` defaults to 1536 and truncates individual descriptions before the budget is applied. When the combined listing overflows, Claude Code **drops descriptions starting with the least-used skills** — eviction ranking is reported as roughly `usageCount × 0.5^(days/7)`, so a brand-new skill scores zero and is first to be dropped. The user sees only a fleeting startup warning; a real one from claude-code issue #56710 reads: *"122 descriptions dropped (full descriptions kept for most-used skills) (5.3%/1% of context)… run /skills to disable some, or raise skillListingBudget."* An evicted skill remains invocable by explicit `/skill-name` slash command but is invisible to the model's auto-selection pass. Critically, the budget is calculated against a fixed ~200K-token baseline rather than the model's actual window (issue #57941), so 1M-context users get ~5× less room than the docs imply.

The scale math is empirically grounded. Alexey Pelykh's measurement (GitHub gist, Dec 4–5 2025, Claude Opus 4.5, contributed to claude-code issue #11045) found: *"Claude Code has an undocumented ~16,000 character budget for skill metadata. With typical 263-char descriptions, only ~42 skills fit… 21 skills (33%) were completely hidden from the agent—it couldn't discover or invoke them."* (~109 chars of XML overhead per skill; compressing descriptions to ≤130 chars fits ~67.) Other practitioner sources put per-skill listing cost at ~75–150 tokens; at the default 1% on a 200K-baseline calculation, roughly 15–25 skills survive before truncation. **PSP runs ~140 skills.** The conclusion is not in doubt: at the default budget most of the library is dark at any moment; only the precise fraction is an M0 task.

**Codex CLI (verified).** Instructions come from AGENTS.md files, concatenated root→cwd (nearest-wins; `AGENTS.override.md` beats `AGENTS.md` at each level), capped at 32 KiB (`project_doc_max_bytes`); Codex **skips empty files and stops adding content once the combined size reaches the cap, truncating silently with no warning.** Skills (SKILL.md, adopted December 2025, stored in `~/.agents/skills/` or `~/.codex/skills/`) are, per practitioner reports, static prompt injection with no live selection observability; files are truncated if long (community guidance: keep under ~4 KB for reliable loading). Skills, MCP, subagents, and plugins compose into a five-layer stack (plugins elevated to first-class in v0.117.0, March 26 2026).

**OpenCode (verified).** At each model step, OpenCode advertises permitted skills that have a description and do not set `opencode/autoinvoke: false`; per the v2 docs, *"the advertisement contains only each skill's ID, name, and description; it does not add every skill body to the prompt."* Body is added only when the model calls the `skill` tool with an exact ID. Same description-only routing pathology, same absence of provenance.

**Kimi Code / MiniMax (verified).** Both support SKILL.md directories (`--skills-dir`; `~/.minimax/skills` via `MINIMAX_SKILLS_DIR`) and `/skill:<name>` / `/skill <name>` invocation, but selection is either manual slash-command or model-discretion; no operator-facing record of what was active. Kimi Code also ships a native goal mode (`/goal status|pause|resume|cancel|replace|next`).

### 2.2 Zero activation observability (the gap no vendor fills)

No harness emits a machine-readable record of *which skills were considered, which were selected, which were dropped for budget, at what version, and where in the prompt they landed.* The operator cannot answer "was skill X active in phase 3, and at what position?" except by reading transcripts and inferring. This is the gap the Activation Ledger closes, and it is the most defensible part of the proposal.

### 2.3 Position and adherence effects are real but bounded — treat position as second-order

The foundational reference is Liu, Lin, Hewitt, Paranjape, Bevilacqua, Petroni & Liang, **"Lost in the Middle: How Language Models Use Long Contexts,"** TACL vol. 12, pp. 157–173 (2024), doi:10.1162/tacl_a_00638: *"performance is often highest when relevant information occurs at the beginning or end of the input context, and significantly degrades when models must access relevant information in the middle of long contexts, even for explicitly long-context models."* Corroborated mechanistically by "Found in the Middle: Calibrating Positional Attention Bias" (2024), which traces the effect to positional attention bias and RoPE long-distance decay.

**But the evidence is mixed at instruction scale, and the spec must not over-weight position.** "Boosting Instruction Following at Scale" (Elder, Duesterwald & Muthusamy, IBM T.J. Watson, submitted Oct 16 2025) attributes instruction-following degradation primarily to *"the degree of tension and conflict that arises as the number of instructions is increased"* rather than to position, and its "Instruction Boosting" method improves IF-rate by up to 7 points for two instructions and up to 4 points for ten. Counting-Stars (2403.11802) could not strongly corroborate lost-in-the-middle beyond 16K. **Conclusion:** prioritize *presence* first; treat *position* as a second-order optimization tuned empirically via the ledger. Minimizing the number and mutual tension of injected instructions is better-supported leverage than positional placement — which is precisely what phase exclusion (§4.8) delivers.

### 2.4 Instruction-axis contamination of the critic (new at Rev 1.1)

The S-01–S-08 sycophancy-correction gate, contractually specified in the San Saba SOW as AC-08, requires that the critic receive **only the artifact**, never the generation-pass conversation history. That constraint is enforced today on the *conversation* axis and only there. It is not enforced on the *instruction* axis.

Consider a Reflect phase that runs in a session where the Execute phase's skills are still resident: build conventions, "make it work" heuristics, the architecture skill that argued for the very design now under review. The generation *history* is gone; the generation *disposition* is still in the prompt. The critic is wearing the builder's instructions. Under the IBM finding above — that competing-instruction tension is the dominant adherence degradation driver — this is not a cosmetic concern: it is a direct, measurable pressure toward exactly the sycophantic self-ratification the gate exists to prevent.

**No amount of description engineering fixes this, because the mechanism is presence, not selection.** It is fixed by making exclusion a first-class routing output and binding it to fresh-context boundaries. See §4.8.

---

## 3. Architecture

### 3.1 Placement within the three-plane goal architecture

The Skill Router is **Plane 2.5**: it lives in UAR alongside the Rust Goal Supervisor and is invoked at each *routing boundary* (§3.3). Data flow per boundary:

1. Supervisor decides to spawn a fresh harness session for phase *P* of change *C* within goal *G* (Ralph-style).
2. Supervisor hands the Router: the Goal Contract, the routing key `(phase, change_id)`, the phase objective/scope, the target harness identity, and the computed context budget.
3. **Intent Classifier** produces an intent feature vector — phase prior (categorical, declared) composed with change-scope retrieval (hybrid, over full bodies).
4. **Injection Planner** solves a budget-constrained tiered selection *and computes the exclusion set*: Tier 1 full-body, Tier 2 name+compressed-summary, Tier 3 deferred/callback-available, Tier X excluded.
5. **Injection Driver** materializes the plan through the harness's native prompt surface; stub suppression enforces exclusions.
6. Supervisor spawns the session; the `request_skill` MCP service is registered so the model can pull Tier-3 skills mid-run (never Tier-X — exclusions are Cedar-denied).
7. Every decision is written to the **Activation Ledger** under `did:uar:goal/<G>/change/<C>/phase/<P>`.
8. On **change archive** (not phase end), the **Compliance Evaluator** produces the rollup that feeds the PMPO v2 Evolution Loop.

Text data-flow diagram:

```
GoalContract ────┐
(phase, change) ─┼─▶ [Intent Classifier] ──features──▶ [Injection Planner] ──InjectionPlan──▶ [Injection Driver] ──▶ fresh harness session
Harness+budget ──┘         │  phase prior              │  + exclusion set                       │  + stub suppression      │
                           │  + scope retrieval        │                                        │                          │
                           ▼                           ▼                                        ▼                          ▼
                  phase-affinity/exclude      requires/conflicts closure          request_skill MCP  ◀── model asks mid-run
                  (derived, §4.1)             conflict pruning                    (Tier 3 only; Tier X denied)
                                                       │                                        │
                                                       └────────────▶ [Activation Ledger] ◀──────┘
                                                                              │
                                        archived change artifacts ──▶ [Compliance Evaluator] ──▶ PMPO v2 Evolution Loop
                                                                              │
                                                       gaps ──▶ [Capability Gap Records] ──▶ out-of-band acquisition (§5)
```

### 3.2 The routing key: `(phase, change_id)`

Phase and change are **two different keys and must not be conflated.**

- **Phase (kbd/PMPO) is a cognitive-mode key.** It says *how* the agent is working. It selects process skills: `openspec-new-change`, `iterative-evolver`, `artifact-refiner`, `sycophancy-correction`, `openspec-verify-change`. Crucially it is a **declared categorical feature, not an inferred one** — the Supervisor already knows the phase. Rev 1.0's classifier was inferring, from free-text phase objectives, something already known. Supplying phase as a prior collapses a large fraction of the search space at zero cost and raises precision.
- **Change (OpenSpec) is a scope key.** It says *what* is being worked on. `design.md` plus the touched paths select domain skills: Rust/Axum conventions, FFI boundaries, Cedar policy authoring, whatever the change actually spans.

**Composition rule:** process skills are selected by the phase prior; domain skills by change-scope retrieval; the union is pruned by `conflicts_with` and by the phase `exclude` set, then tiered under budget.

### 3.3 Routing boundaries: full re-route vs. incremental pull

**Injection is monotonic within a session.** A skill body can be added mid-run; it cannot be removed. This single constraint determines where routing may fire:

| Condition | Mechanism | Rationale |
|---|---|---|
| **Fresh context available** — phase transition, change transition (the Supervisor is spawning a session anyway) | **Full re-route.** Selection is a genuine selection, including exclusions. | Only here can Tier-X exclusion be enforced, because exclusion means *absent from the assembled prompt*. |
| **No fresh context** — mid-phase, inside a native `/goal` run | **`request_skill` callback only.** Additive by nature. | Honest about what is mechanically possible; prevents pretending re-selection occurred. |
| **Sub-phase (per-task checkbox in `tasks.md`)** | **No routing.** | Re-routing here yields accumulation, not selection — and accumulation is the failure mode §2.3 identifies as the dominant adherence killer. Explicitly rejected. |

This reframes `request_skill`: it is not merely a recall-gap mitigation (Rev 1.0), it is **the in-session half of a two-tier routing model** — full re-selection at boundaries, incremental pull between them.

### 3.4 Components

- **PSP Skill Registry** — single source of truth. Full bodies indexed (not just descriptions), content-hash versioned, immutable versions, Cedar-gated. Aligns with UAR's capability-gated WASM skill registration (declared capabilities validated against actual import surface). Backed by surreal-memory-server's HNSW + BM25 hybrid index.
- **Intent Classifier** — phase prior composed with hybrid retrieval (HNSW + BM25) fused via Reciprocal Rank Fusion (Cormack, Clarke & Büttcher, SIGIR 2009, doi:10.1145/1645953.1646039; **k=60**, the original default now standard across OpenSearch/Elasticsearch/Azure AI Search/Weaviate/Qdrant, benchmarks landing in k∈[40,80]), optionally refined by a small classifier (local Qwen via candle-vllm, or Claude Haiku).
- **Injection Planner** — budget-constrained tier solver **plus exclusion-set computation**.
- **Per-harness Injection Drivers** — one adapter per harness (§4.4), dispatched through `knowme:harness` / BossFang.
- **`request_skill` callback service** — MCP tool served by the Supervisor.
- **Activation Ledger** — SurrealDB provenance store, OpenTelemetry GenAI-compatible.
- **Compliance Evaluator** — LLM-judge-with-rubric plus deterministic postcondition checks, rolled up at change archive.
- **Capability Gap Recorder** — emits `CapabilityGapRecord` when retrieval returns nothing above threshold (§5).

### 3.5 Cargo crate boundaries (capability inversion preserved)

The Supervisor and Router must be structurally incapable of write actuation:

- `psp-registry` — registry types, content-hash versioning, hybrid-index client. No actuator deps.
- `psp-router-core` — Intent Classifier + Injection Planner + exclusion computation. Pure logic; depends only on `psp-registry` and classifier-client traits. No harness deps, no write actuators.
- `psp-injection-drivers` — per-harness adapters. Process/SDK surfaces, spawn-and-read only.
- `psp-ledger` — SurrealDB writer + OTel exporter.
- `psp-compliance` — evaluator, read-only over artifacts.
- `psp-request-skill-mcp` — callback MCP server; depends on `psp-router-core` + Cedar; returns bodies, cannot mutate the workspace.
- `psp-gap-recorder` — emits gap records. **Write-capable acquisition lives outside this graph entirely** (§5.3) — the supervisor may *propose*, never *admit*.

Capability inversion at the Cargo level: `psp-router-core` and the Supervisor crate declare no dependency on any write-actuator crate, enforced as UAR already validates declared-vs-actual WASM import surfaces.

---

## 4. Functional Specification

### 4.1 Registry schema

```
SkillRegistryEntry {
  skill_id: string,
  version: semver,
  content_hash: sha256,            // immutable per version
  frontmatter: { name, description, license, compatibility, metadata },
  body: markdown,                  // full body, indexed
  body_tokens: int,
  requires: [skill_id],            // dependency graph (from capability cards)
  enhances: [skill_id],
  conflicts_with: [skill_id],      // symmetric negative selection
  phase_affinity: {                // NEW at Rev 1.1 — DERIVED, not authored
    affinity: [assess|analyze|plan|execute|reflect|verify|archive],
    exclude:  [assess|analyze|plan|execute|reflect|verify|archive],
    provenance: derived_from_ledger | human_confirmed | authored,
    confidence: float,
  },
  capabilities: [capability],      // Cedar-gated, validated vs import surface
  compliance_criteria: [ ... ],    // PSP frontmatter extension, §4.7
  embedding: vector,               // HNSW
  bm25_terms: sparse,
}
```

**`phase_affinity` is derived, not authored.** Authoring affinity for ~140 skills on top of the compliance-criteria debt is the wrong sequencing. Bootstrap it from the ledger: run the transitional three-mode instrumentation (§7.5), observe the empirical distribution of which skills get selected in which phases, propose affinities from that distribution, and require human confirmation to promote `provenance` from `derived_from_ledger` to `human_confirmed`. This is an M1/M2 artifact, **not** migration work.

**`exclude` is stricter than `conflicts_with`.** `conflicts_with` is pairwise and symmetric (two skills whose conventions collide). `exclude` is phase-scoped and unary (this skill must not be present during this cognitive mode, regardless of what else is selected). Exclusion is what §4.8 requires; conflict pruning cannot substitute for it.

### 4.2 Router pipeline contract

**Input:** `(goal_contract, phase, change_id, phase_objective, harness_id, context_budget_tokens)`
**Output:** `InjectionPlan` + persisted `ActivationRecord` (+ zero or more `CapabilityGapRecord`).

Pipeline:
1. **Phase prior** — retrieve candidate process skills by `phase_affinity.affinity ∋ phase`.
2. **Scope retrieval** — hybrid RRF retrieval over full bodies, query built from `change_id` artifacts (`design.md`, `proposal.md`, touched paths) + phase objective.
3. **Optional LLM rerank** — only when the retrieval confidence margin is thin.
4. **Dependency closure** — pull in `requires`.
5. **Conflict pruning** — drop lower-scoring member of any `conflicts_with` pair.
6. **Phase exclusion** — remove every skill with `phase_affinity.exclude ∋ phase`. **This step is non-overridable and runs last among pruning steps** (§4.8).
7. **Tier assignment** under budget.
8. **Gap detection** — if a required capability class yields no candidate above threshold, emit `CapabilityGapRecord` (§5.2) and continue with what exists.

**Why full-body indexing beats description-only:** the harness pathology is precisely that descriptions are lossy proxies for bodies. Indexing bodies lets a short phase/scope query match load-bearing procedural detail that never appears in a 1024-char description. For bodies exceeding one chunk, chunk on markdown H2/H3 boundaries and index chunk-level, aggregating to skill-level via max-chunk score (BM25 degrades on sub-page chunks — keep chunks section-sized, not sentence-sized).

**Plan cache.** Key on `(phase, change_id, content_hash(change_artifacts), harness_id, budget)`. Most transitions do not move this key, so recomputation is bounded — this is the mitigation for the classifier-invocation multiplication that the two-boundary design would otherwise cause (§9.9).

**Latency/cost envelopes (targets, to be validated):**
- Pure hybrid retrieval + phase prior: single-digit-to-low-tens of ms on CPU; ≈$0.
- + Haiku rerank: +hundreds of ms, small token cost; conditional on thin margin.
- Fine-tuned local classifier (Qwen / ModernBERT-class, à la vLLM Semantic Router): tens of ms; justified only once ledger data density warrants — mirroring the OATS finding that learned re-rankers *hurt or match baseline when outcome data is sparse relative to tool-set size*. Start with zero-cost hybrid retrieval; add learned components only when data justifies.

### 4.3 Injection plan schema

```
InjectionPlan {
  harness_id, routing_key: {phase, change_id}, budget_tokens,
  tier1_full_body: [ {skill_id, version, content_hash, target_position} ],
  tier2_summary:   [ {skill_id, version, compressed_summary, target_position} ],
  tier3_deferred:  [ {skill_id, version} ],          // callback-available
  tierX_excluded:  [ {skill_id, reason: phase_exclude|conflict} ],  // Cedar-denied to request_skill
  rejected:        [ {skill_id, reason: budget|low_score, score} ],
  injection_method: enum,
  total_injected_tokens: int,
}
```

Budget model: `budget = context_window − goal_contract − STATUS.md − headroom`. Tier 1 fills by descending selection score until the body sub-budget is exhausted; overflow demotes to Tier 2, then Tier 3. **Keep Tier 1 small** — §6/§9 token model.

**`tierX_excluded` is distinct from `rejected`.** Rejected skills are budget/score casualties and remain `request_skill`-eligible. Excluded skills are **Cedar-denied at the callback** — the model cannot pull them back in mid-phase. Without this, phase exclusion would be trivially defeatable by a model that asks.

### 4.4 Per-harness Injection Driver table (verified mechanics)

| Harness | Primary injection surface | Append vs replace | Notes / limits |
|---|---|---|---|
| **Claude Code** | `--append-system-prompt "<bodies>"` (only valid with `--print`/`-p`); Agent SDK `systemPrompt:{type:"preset",preset:"claude_code",append:"..."}` | **Append** (preserves default coding identity, tool guidance, safety) | `--system-prompt`/`--system-prompt-file` *replace* the entire prompt and are mutually exclusive with each other; append can combine with either. `--append-system-prompt` and `--system-prompt` cannot both be set. |
| **Claude Code (hook path)** | `UserPromptSubmit` hook → stdout auto-injected as context; `SessionStart` (source `startup`/`resume`/`clear`/`compact`) stdout auto-injected via `hookSpecificOutput.additionalContext` | Additive | Special events whose stdout is auto-added to context. `UserPromptSubmit` default timeout 30 s; on timeout the additionalContext is **discarded silently** (a notice appears in transcript as of v2.1.196). Use `SessionStart(compact)` / `PreCompact` for C2 re-injection. |
| **Codex CLI** | config.toml `model_instructions_file` (**REPLACE** — "Replacement for built-in instructions instead of AGENTS.md") and `developer_instructions` (**APPEND** — "Additional developer instructions injected into the session," as role=developer messages); runtime via `--config`/`-c`; custom-agent `.toml` layers via `agents.<name>.config_file` | See cells | **No `--append-system-prompt`/`--system-prompt` flag exists** — issues #11588 (closed, unimplemented, no PR) and #11117 requested it and were not fulfilled. The plain `instructions` config field is "reserved for future use" — do not use. `experimental_instructions_file` is the legacy name for `model_instructions_file`. AGENTS.md layering capped at 32 KiB, silent truncation. MCP supported. |
| **OpenCode** | Server/SDK `client.session.prompt({ path:{id}, body:{ parts:[{type:"text",text:"<bodies>"}] } })`; agent config `prompt` field (`{file:...}`); `OPENCODE_CONFIG_CONTENT` env for custom-agent system prompt | Agent `prompt` **replaces** provider prompt; session `parts` are **additive** user-context | System prompt assembled in `session/prompt.ts` from a provider `.txt` (anthropic.txt / beast.txt / gemini.txt / codex_header.txt / qwen.txt) + AGENTS.md/CLAUDE.md walk. `chat.message` / `session.compacted` plugin hooks available for injection *and* for logging which skills loaded. |
| **Kimi Code** | `kimi --print -p "<prompt>"`; `--agent-file` custom agent with `system_prompt_path`; `--skills-dir`; `--mcp-config-file` | Agent file sets fixed system context; `-p` additive | No dynamic-initial-prompt-then-interactive mode (issue #2240 open). `/skill:<name>` reads SKILL.md and sends as prompt. `--output-format` only with `--prompt`. MCP supported. |
| **MiniMax CLI** (unofficial Rust `minimax-cli`, Hmbown) | `MINIMAX_SKILLS_DIR`, `~/.minimax/mcp.json`, `config.toml`; `/skills` + `/skill <name>` | Additive prompt composition | **MCP tools execute without TUI approval prompts** — trust boundary; enable only trusted servers. Compaction knobs (`MINIMAX_COMPACTION_*`) exposed as env. Distinct from MiniMax's *official* `mmx-cli` (media/generation). |
| **Cowork** *(NEW at Rev 1.1 — UNCHARACTERIZED)* | **Unknown.** Runs skills and plugins, therefore a legitimate seventh dispatch target with its own prompt-assembly surface and budget. | Unknown | **Explicitly not specced.** Cowork is a *dispatch target*, **not** a skill-lookup service — a common category error worth naming. Its extension surface is not sufficiently characterized to write a driver against; guessing here would produce a driver that silently no-ops. **Added to the M0 drift-surface inventory as a characterization task**; driver deferred to M4 pending that. |
| **UAR-native** | Direct — Router composes the SkillService prompt via the NativeSkill trait; full control over position, exclusion, and provenance | N/A (reference implementation) | Reference path. Every harness driver is a lossy approximation of it — and only here is exclusion enforceable with certainty rather than by stub suppression. |

### 4.5 `request_skill` MCP tool schema

```
tool request_skill {
  input:  { capability_description: string, phase_id: string, reason: string },
  output: { skill_id, version, content_hash, body, cedar_decision }
        | { denied, reason: excluded_for_phase | capability_denied | cap_exceeded }
}
```

Guards: per-phase call cap (default 10); dedupe identical `capability_description` within a phase (return cached body, no re-classification); **Tier-X skills are hard-denied** with `excluded_for_phase`; every call and denial logged. Cedar authorizes `action == "request_skill"` on `resource == skill:<id>` under `principal == goal:<G>` with **scope-narrowing only**, consistent with UAR's delegation-chain discipline.

**On Anthropic's Tool Search / deferred-loading pattern.** It is cited in §8 as validating the *efficiency* thesis, and it is explicitly **rejected as a selection mechanism**. Tool Search is in-harness, model-invoked, runtime retrieval over definitions — architecturally the inverse of the Router. Delegating selection to it reinstates exactly what the inversion eliminates: probabilistic model choice with no ledger entry, no Cedar decision, no version pin. The one place the pattern *is* correct is `request_skill`, which is already a search tool; the distinction is **who serves it**. Model `request_skill`'s ergonomics on Tool Search by all means; never outsource the resolution.

### 4.6 Activation Ledger schema

```
ActivationRecord {
  goal_id (DID), change_id, phase_id, harness_id, timestamp,
  routing_key: {phase, change_id},
  plan_cache_hit: bool,
  intent_features: {...},
  skills_selected: [ {skill_id, version, content_hash, tier, injection_method,
                      prompt_position, token_offset_start, token_offset_end, token_cost} ],
  skills_excluded: [ {skill_id, reason} ],
  skills_rejected: [ {skill_id, reason, score} ],
  request_skill_calls: [ {capability_description, resolved_skill_id, cedar_decision, ts} ],
  capability_gaps:  [ gap_record_ref ],
  compliance_results: [ {skill_id, method, passed, evidence_ref} ],  // filled at change archive
}
```

**Fields that make outcome-correlation work:** `content_hash` (ties a compliance regression to a specific body version), `prompt_position` + `token_offset_*` (position effects measured, not assumed), `tier`, `token_cost` (per-skill cost/benefit derivable), and — new — `skills_excluded` (so a phase-exclusion regression is visible as data).

**OTel GenAI mapping:** emit as spans with `gen_ai.system_instructions` (injected bodies), `gen_ai.operation.name = "skill_selection"`, skill content as span **events**, not attributes (attributes are always indexed; events can be dropped at the Collector) — respecting the convention's size/PII guidance. Export via OTLP so the Surreal store plus optional Braintrust/Datadog/W&B Weave can consume it.

**Operator surface (Rev 1.1 change):** the **change-level rollup is primary**; per-phase records are drill-down. A per-phase compliance score on a Plan phase is close to meaningless in isolation; the operator question is "across this change, what was active, at what versions, and did the archived artifact comply?" Leading with phase records turns the ledger into noise at precisely the moment it should become evidence.

### 4.7 Compliance-criteria authoring format (SKILL.md PSP extension)

PSP-namespaced frontmatter, harness-ignored (the Agent Skills spec mandates unknown frontmatter fields are dropped; only `name`/`description` are required, with `license`/`compatibility`/`metadata`/`allowed-tools` optional):

```yaml
x-psp-compliance:
  - id: C-postcondition-1
    kind: deterministic        # machine-checkable
    check: "grep -q 'CHANGELOG' STATUS.md"
  - id: C-rubric-1
    kind: llm-judge
    rubric: "The commit was split into semantic groups; no `git add .` was used."
x-psp-phase:                   # optional authoring override of derived affinity
  affinity: [execute]
  exclude:  [reflect, verify]
```

`deterministic` checks run in `psp-compliance` (read-only). `llm-judge` checks feed a rubric to a judge that receives **only artifacts, never generation history** — the same S-01–S-08 isolation, applied to the skill as the thing under review. Rubric-based LLM-judge is well-supported (≈80–90% agreement with human raters when rubrics are specific); calibrate against a small human-annotated set and account for verbosity/self-preference bias per the Judge Reliability Harness literature.

### 4.8 Phase exclusion and the sycophancy gate *(promoted to architectural requirement at Rev 1.1)*

**Requirement AC-08-S (structural).** For any phase whose cognitive mode is critical review — `reflect`, `verify`, and any phase running a `sycophancy-correction` pass — the Router MUST:

1. Route on a **fresh context**. A critic phase may never be a continuation of the generation session it reviews. (This is already the Supervisor's default; AC-08-S makes it non-optional for critic phases specifically.)
2. Inject **only critic-class skills** — those whose `phase_affinity.affinity` includes the critic phase.
3. **Actively exclude** every generation-class skill via `tierX_excluded`, not merely decline to select it.
4. **Hard-deny** excluded skills at the `request_skill` callback (§4.5), so exclusion cannot be defeated mid-phase.
5. Record the exclusion set in the `ActivationRecord`, making the isolation **auditable rather than assumed**.

**Rationale.** AC-08 as contracted requires the critic receive only the artifact. Rev 1.0 satisfied this on the conversation axis alone. Generation-class skills resident in a critic's prompt constitute instruction-axis contamination (§2.4): the critic reasons under the builder's conventions, and under the IBM instruction-tension finding this measurably biases toward ratification. Phase exclusion converts a convention into a structural property of the instruction plane — the same move as capability inversion at the Cargo level, applied to instructions instead of code.

**Verification.** The Compliance Evaluator asserts, per critic phase, that `tierX_excluded ⊇ {skills with affinity ∋ execute}` and that no `request_skill` call for an excluded skill was granted. A violation is a **gate failure**, not a warning.

### 4.9 Suppression / stub format

Harness skill directories become projections. For each PSP skill, the generated harness stub:
- Sets `disable-model-invocation: true` where supported (Claude Code confirmed — keeps a skill loaded but invisible to the auto-selector; OpenCode **ignores** this field per issue #11972, so there set `opencode/autoinvoke: false` or strip the description).
- Strips the description to a pointer (`"Managed by PSP Router — do not auto-invoke"`) so it contributes ~0 to the listing budget and cannot double-activate.
- Retains user-invocability (`/skill-name`) as an escape hatch — **except** for skills in the active phase's exclusion set, where the escape hatch is itself a hole in AC-08-S and must be closed for the duration of critic phases.

**Live-hazard caveat:** on Claude Code, `disable-model-invocation: true` currently also blocks *explicit* slash invocation in some builds (issues #26251, #43809) and blocks subagents from loading a parent-referenced skill. The stub generator must detect harness version and fall back to description-stripping where that bug is present.

### 4.10 C1/C2/C3 redefinition (IP-002 component definitions)

- **C1 — was: trigger reliability via description engineering → now: Deterministic Skill Routing on `(phase, change_id)`.** PSP owns selection out-of-harness; presence guaranteed by construction, not by keyword-tuning descriptions against an opaque budget. Includes exclusion as a first-class output.
- **C2 — was: compaction re-anchor → now: mostly subsumed by fresh-session phasing.** Retained as `PreCompact` / `SessionStart(compact)` re-injection for long native-`/goal` phases that do not get a fresh session per phase. OpenCode's ecosystem already has compaction-reinjection plugins (listening on `session.compacted`) — reuse that pattern rather than reinvent it.
- **C3 — was: tiered activation / compliance eval → now: Injection Tiering + Compliance-on-Ground-Truth, rolled up at change archive.** Tiering is the Injection Planner. Compliance is measured against *verified presence* (we know the skill was in context, at what position, version, and tier). **The rollup boundary moves from phase to `openspec-archive-change`**: at archive we hold the complete artifact set plus every `ActivationRecord` for every phase in the change. A change is the smallest **complete unit of verified work**, which makes it the meaningful unit for compliance — and the right granularity to feed PMPO v2's Evolution Loop (Compile→Evaluate→Optimize→Promote) when optimizing router weights, tier assignments, and derived phase affinities.

---

## 5. Capability Gaps and the Acquisition Boundary *(new at Rev 1.1)*

### 5.1 Non-goal, stated explicitly

**Runtime installation of skills fetched from the open internet is out of scope for PSP, permanently and by design.** Discovery is valuable; in-flight installation is not. It breaks three invariants already held elsewhere in the estate:

1. **It dissolves the registry as source of truth.** IP-002's premise is content-hash-versioned, immutable, Cedar-gated entries. A skill fetched mid-run from a mutable URL makes `content_hash → outcome` correlation unreproducible — and that correlation is the entire value of the ledger.
2. **A markdown skill has no import surface to validate.** UAR's WASM registration has a structural guarantee: declared capabilities checked against actual module imports. SKILL.md has no equivalent. It can carry `allowed-tools`, instruct command execution, or carry injection payloads, with nothing to statically verify against. Admitting the *least verifiable* artifact type through the *least guarded* door inverts the estate's whole security posture.
3. **It violates capability inversion.** Admitting a skill is a **write to the instruction plane**. By the rule that the supervisor cannot depend on write actuators, the supervisor must be able to *propose* acquisition and structurally unable to *perform* it. In a Ralph-style run with no human present for hours, a bad skill admitted at Assess shapes every subsequent phase — blast radius is the entire goal.

### 5.2 The constructive version: Capability Gap Records

When retrieval returns nothing above confidence threshold for a capability the phase needs, that is not a failure — it is a **finding**.

```
CapabilityGapRecord {
  gap_id, goal_id, change_id, phase_id, timestamp,
  intent_features: {...},
  best_candidates: [ {skill_id, score, why_insufficient} ],
  capability_class: string,          // clustered across runs
  resolution: open | acquired(skill_id@version) | synthesized(skill_id@version) | wontfix,
}
```

The Router emits the record and **continues with what exists**. Gap frequency then becomes measurable — "12 gaps across 40 runs, 8 recurring in 2 capability classes" is a roadmap input and a Compile-step signal for an Evolution Loop operating on **capability coverage**, not merely router weights. This is KDD-shaped: the gap is knowledge the system discovered about its own limits, becoming a durable artifact rather than a silent runtime patch.

### 5.3 Out-of-band acquisition pipeline (human-gated, outside the goal loop)

Gaps feed a pipeline that *may* search marketplaces, repos, and the web — because it runs **outside** the goal loop, with a human at the gate:

1. **Candidate retrieval** — marketplaces, repositories, documentation.
2. **Adversarial review** — the sycophancy-correction critic reads the candidate **artifact only**, same S-01–S-08 isolation, with the skill as the thing under review.
3. **Cedar evaluation** — declared capabilities assessed against the goal classes the skill would serve.
4. **Human admission** — at a pinned version and content hash. Non-delegable.
5. **Staged promotion** — admitted skills enter at **Tier 3 (callback-available only)** and promote to Tier 1 only after compliance data justifies it. New skills never start load-bearing.

### 5.4 Synthesis often beats acquisition

For a gap in a well-documented domain, **generate the skill rather than import one.** Context7 and docfork are already wired; retrieve primary documentation, synthesize a candidate skill, and run it through the identical admission gate (§5.3 steps 2–5). Provenance is cleaner (exact derivation known), there is no third-party supply chain, and output already conforms to house conventions. Acquisition is the better path only where the value is genuinely **tacit practice** rather than documented fact.

### 5.5 The legitimate dynamic path: signed distribution, not a runtime installer

If dynamic acquisition matters operationally, the correct investment is **not** a runtime installer — it is the **signed trust channel already designed in the plugins strategy**: entitlements as W3C Verifiable Credentials, author DID issues, user DID holds, offline verification, revocation by lifetime. Under that model the Supervisor may fetch and inject any skill whose VC verifies against a trusted issuer without a human in *that specific* loop, because admission already happened upstream at issuance.

That is shared spend: it is the same infrastructure the KnowMe plugin marketplace requires, not a PSP one-off. It also cleanly preserves capability inversion — verification is a read; issuance is the write, and it lives with the issuer.

---

## 6. Implementation Plan (M0 measurement-first)

Each milestone maps to OpenSpec artifacts (`proposal.md` / `tasks.md` / `design.md` / `spec-delta.md`) under single-writer discipline.

**M0 — Measurement (no build).** From harness transcripts, compute baseline native activation rate: across a sample of phases where a skill *should* have fired, how often did it? Build the token-cost model from measured body sizes. **Also: characterize Cowork's extension surface** and start the drift-surface inventory (per harness: flag/file/field, version last verified, fallback).
*Token-cost anchors:* description/metadata ≈100 tokens/skill (always loaded); recommended body ceiling <5,000 tokens (agentskills.io: *"Instructions (<5000 tokens recommended)… Keep your main SKILL.md under 500 lines"*); observed per-skill body consumption **3,000–5,500 tokens** (claude-code issue #14882, via secondary — verify before formal quotation); Codex practical truncation ~4 KB (~1,000 tokens); Claude Code Read-tool hard-fail at 10,000 tokens (anthropics/claude-plugins-official #995); post-compaction per-skill 5,000 / combined 25,000 tokens (secondary — verify).
**Exit:** baseline activation rate + cost model published; the "most skills dark" estimate confirmed or corrected against the actual 140-skill listing on the actual context window; Cowork characterized or formally deferred.

**M1 — Registry + Classifier + phase prior** (`psp-registry`, `psp-router-core`). Register skills, stand up hybrid retrieval, implement the `(phase, change_id)` composite key, produce InjectionPlans offline. **Begin phase-affinity derivation** from M0 transcript data.
**Exit:** top-k retrieval recall ≥ agreed threshold on a labeled phase/change set; plan generation under target latency; RRF k tuned in [40,80]; first derived `phase_affinity` proposals generated for human confirmation.

**M2 — UAR-native driver + Ledger + phase exclusion.** Wire Router into SkillService via NativeSkill; write ActivationRecords; OTel export; **implement AC-08-S end-to-end on the native path** (exclusion set, Cedar denial at callback, evaluator assertion).
**Exit:** native path fully Router-driven; ledger renders in operator UI with change-level rollup primary; AC-08-S verified by the evaluator on a critic phase.

**M3 — Claude Code + Codex drivers.** `--append-system-prompt` (Claude) / `developer_instructions` (Codex) injection; stub generation with version-gated fallback; A/B against native selection with ledger instrumentation.
**Exit:** Router-selected presence = 100% on both; measured task-outcome delta **reported, not assumed**; exclusion enforced via stub suppression with known-gap documentation where the harness cannot guarantee it.

**M4 — OpenCode / Kimi / MiniMax drivers, Cowork driver (if characterized), `request_skill`.**
**Exit:** callback service live with Cedar + loop guards + Tier-X hard denial; all characterized harnesses driven.

**M5 — Compliance Evaluator + archive rollup + Evolution Loop.**
**Exit:** compliance rollup fires at `openspec-archive-change`; telemetry closes the loop; router weights, tier assignments, and derived affinities optimized against outcomes.

**M6 — Capability Gap pipeline.** Gap recorder in-loop; out-of-band acquisition pipeline with human gate and staged Tier-3 promotion; synthesis path via Context7/docfork.
**Exit:** gap records accumulating and clustered by capability class; at least one gap resolved through each of the acquisition and synthesis paths, both admitted at pinned hash.

Repos: registry/router/drivers/ledger → `universal-agent-runtime`; skill bodies, compliance criteria, stubs → `prometheus-skill-pack`; harness dispatch → `librefang`/BossFang; goal integration → `know-me-system`.

---

## 7. Migration Plan (~140 skills)

1. **Extraction.** Parse each SKILL.md; frontmatter → registry metadata; body → indexed body; content hash → v1.0.0 immutable.
2. **Graph authoring.** Populate `requires`/`enhances` from existing capability cards; author `conflicts_with` for skills with incompatible conventions.
3. **Phase affinity — derived, not authored.** Do **not** hand-author affinity for 140 skills. Bootstrap from ledger observation (§4.1); human confirmation promotes provenance. **Exception:** critic-class and generation-class skills relevant to AC-08-S should be human-confirmed early, since exclusion correctness gates the sycophancy contract and cannot wait for statistical confidence.
4. **Compliance-criteria debt.** The largest migration cost. Stage it: deterministic checks first (cheap, high-value), rubrics for load-bearing skills next, tail deferred.
5. **Stub generation.** Emit suppressed projections into each harness directory (§4.9), version-gated for the Claude Code `disable-model-invocation` bug.
6. **Transitional three-mode instrumentation.** Run pre-injection (Router) / callback / residual-native-triggering concurrently, with the ledger comparing Router vs native selection — so the efficacy claim is **earned from data** before native triggering is disabled. This same run produces the phase-affinity distribution for step 3.

---

## 8. Framing

**What PSP now claims.** "PSP owns skill *selection* and *observation* on every harness. Selected skills are present in context deterministically; excluded skills are structurally absent; and every selection decision is recorded under the goal DID with version, tier, position, and cost."

**What PSP must NOT claim.** That injection guarantees compliance. **Presence ≠ adherence.** Attention, position, competing instruction tension (the better-supported degradation driver per IBM), and RL-trained harness disposition (Codex persistence bias vs Claude consent-orientation) still shape whether a present instruction is followed. The exact line: **"PSP owns skill selection and observation on every harness; harnesses retain influence only over execution disposition."**

**Positioning vs vendors.** Not adversarial to Anthropic/OpenAI — this is the layer they structurally cannot provide: cross-harness, operator-owned selection and provenance. It mirrors their direction (Anthropic's Tool Search / `defer_loading` lifted Opus 4 MCP accuracy from 49% → 74%, Opus 4.5 from 79.5% → 88.1%, at ~85% token reduction; the Agent Skills open standard is built on progressive disclosure) while generalizing across harnesses and adding the provenance they omit. It compounds with the goal abstraction: the Supervisor already owns verification and phasing; owning the instruction plane means the contract-of-record now governs **what the agent knows**, not merely what it must achieve.

**And one claim that is now stronger than efficiency.** With AC-08-S, PSP can state that critic isolation is *structurally enforced and auditable* — a contractual assurance (San Saba AC-08) backed by a mechanism rather than a convention. That is a governance claim no harness vendor is positioned to make, and it is worth more in enterprise settings than any token-efficiency number.

---

## 9. Risks and Open Questions

1. **Efficacy is unproven.** Confident wins are presence, observability, and critic isolation; net task-outcome improvement is uncertain. *Scenario that hurts Prometheus:* we ship the inversion, presence hits 100%, adherence doesn't improve because the bottleneck was always disposition — and we've absorbed 140 skills of migration debt for a structural win the market undervalues. *Mitigation:* M0 baseline + M3 measured delta gate the efficacy narrative; lead with the governance claim, which does not depend on the efficacy result.
2. **Token cost of body-injection.** Bodies cost far more than a description listing (observed 3,000–5,500 tokens each). *Scenario:* over-eager Tier 1 crowds out the Goal Contract itself — the artifact of record. *Mitigation:* budget reserves Goal Contract + STATUS.md + headroom first; Tier 1 is what remains, not what is wanted.
3. **Injection-flag stability / ToS.** `--append-system-prompt`, Codex `developer_instructions`, hook-stdout injection are undocumented-to-semi-documented and move fast (the entire `skillListingBudgetFraction` regime appeared in weeks). *Mitigation:* per-driver version detection; drift-surface inventory maintained from M0; the ledger catches drift by construction.
4. **Recall gap of boundary-time classification.** Skills whose need emerges mid-phase are missed; `request_skill` mitigates but depends on the model knowing to ask. Published tool-retrieval accuracy is sobering (Anthropic's own Tool Search moved Opus 4 from 49→74%; a third-party production test concluded *"60% retrieval accuracy isn't production-ready when agents need to reliably take real-world actions"*). *Scenario:* the model doesn't ask, the tail skill stays dark, and we've relocated the recall gap rather than closed it.
5. **Exclusion enforceability is harness-dependent.** Only the UAR-native path can guarantee absence; on third-party harnesses, exclusion is enforced by stub suppression, which the Claude Code `disable-model-invocation` bug can defeat. *Scenario:* AC-08-S is claimed as structural but is only advisory on the harness the client actually uses. *Mitigation:* per-harness exclusion-guarantee tier documented explicitly; critic phases preferentially routed to UAR-native or to harnesses where suppression is verified.
6. **Double-injection / conflict.** If suppression fails and both Router and native fire, conflicting conventions collide — the best-evidenced adherence killer. `conflicts_with` + stub suppression mitigate; the Claude Code bug is a live hazard requiring version-gated fallback.
7. **Classifier as a new failure surface.** The Router can mis-select. Unlike the harness it is observable and optimizable — but a stale embedding index degrades every phase silently until the ledger surfaces it. *Mitigation:* keep BM25 and vector indexes in sync (a known operational footgun); alert on selection-confidence collapse.
8. **Codex 32 KiB / no-append constraint.** With no append flag, injection rides `developer_instructions` (append, role=developer) or `model_instructions_file` (replace) under the 32 KiB cap; large Tier-1 plans may not fit. **Open question:** is `model_instructions_file` (replace) safe, or does replacing Codex's base prompt discard too much harness scaffolding? Lean toward `developer_instructions` as default; reserve replace for controlled experiments.
9. **Routing-cost multiplication (new at Rev 1.1).** Two boundaries (phase *and* change) multiply classifier invocations and ledger volume. *Mitigation:* plan cache keyed on `(phase, change_id, content_hash(change_artifacts), harness_id, budget)`; change-level rollup as primary operator surface with per-phase as drill-down.
10. **Phase-affinity bootstrap circularity (new at Rev 1.1).** Affinity is derived from ledger observation, but early ledger data comes from a Router that lacks good affinities. *Mitigation:* transitional three-mode run supplies native-selection data as the bootstrap distribution; AC-08-S-relevant affinities are human-confirmed early rather than waiting on statistics.
11. **Gap-pipeline latency (new at Rev 1.1).** Human-gated acquisition means a discovered gap may take days to close, during which runs proceed under-provisioned. *Accepted deliberately* — the alternative is runtime installation, whose failure mode is unbounded (§5.1). Synthesis (§5.4) is the latency mitigation where the domain is documented.
12. **Cowork unknown (new at Rev 1.1).** An uncharacterized dispatch target is an unquantified hole in the "every harness" claim. *Mitigation:* characterize in M0; until then, scope claims to characterized harnesses explicitly rather than implying universality.
13. **Open question — native `/goal` phases.** Native `/goal` runs multi-turn *within* a phase without fresh sessions, so C2 dilution reappears and exclusion cannot be re-enforced mid-run. Is retained `PreCompact`/`SessionStart(compact)` re-injection a winnable fight, or should native `/goal` be treated as fast-path-only that the Supervisor overrides with fresh-session phasing whenever instruction fidelity matters? *Recommendation:* treat native `/goal` as an **inner accelerator** bounded to a turn budget, never the contract-of-record — consistent with IP-001 — and **prohibit it entirely for critic phases**, where AC-08-S requires guaranteed context freshness.

---

### Verified vs practitioner-reported (source-quality note)

- **Verified (primary/first-party):** Claude Code `--append-system-prompt` semantics and `skillListingBudgetFraction` / `skillListingMaxDescChars` (Claude Code docs + GitHub issues); Codex AGENTS.md 32 KiB cap and absence of an append flag (OpenAI config reference + issues #11588 / #11117); OpenCode advertisement-vs-body loading (opencode.ai v2 docs); Anthropic Tool Search accuracy figures (anthropic.com engineering); Agent Skills progressive-disclosure spec (agentskills.io); lost-in-the-middle (Liu et al., TACL 2024); RRF (Cormack et al., SIGIR 2009); OTel GenAI conventions (opentelemetry.io).
- **Practitioner-reported (indicative; confirm in M0):** the 3,000–5,500-token per-skill consumption and 25,000-token post-compaction combined cap (secondary blogs citing claude-code issue #14882); the `usageCount × 0.5^(days/7)` eviction formula; the ~15–25-skills-survive back-of-envelope. The ~16,000-char budget / 33%-hidden figure is a reproducible independent measurement (Pelykh gist) but on a specific build — re-measure on the current version.
- **Uncharacterized (Rev 1.1):** Cowork's extension and prompt-assembly surface. Not specced rather than guessed.
