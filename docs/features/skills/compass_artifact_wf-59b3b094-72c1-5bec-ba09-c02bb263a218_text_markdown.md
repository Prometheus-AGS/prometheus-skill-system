# PAGS-SPEC-PSP-IP-002: The Skill Router — Inverting the Instruction Plane
### A specification for PSP-owned skill selection, injection, and activation observability across coding-agent harnesses and the Universal Agent Runtime
*Rev 1.0 (spec for build) — Travis James, Prometheus AGS — supersedes the C1/C2/C3 definitions in PAGS-SPEC-PSP-IP-001 rev 1.1*

---

## TL;DR

- **The inversion is worth building, but the confident wins are narrow and structural, not headline efficacy.** PSP can take ownership of skill *selection* (out-of-harness intent classification over full skill bodies) and *observation* (a per-goal Activation Ledger) on all five harnesses plus UAR-native, guaranteeing that load-bearing skills are *present in context deterministically* — which no harness does today, because every harness routes on frontmatter descriptions only, under an opaque token budget, with silent eviction.
- **Presence is not compliance, and net task-outcome improvement is genuinely uncertain until measured.** Injection guarantees instructions are in context; attention, prompt position, competing instructions, and RL-trained harness disposition still shape adherence. The spec is honest about this: M0 measurement precedes any efficacy claim, and the transitional three-mode instrumentation earns the claim from ledger data rather than asserting it.
- **The enabling mechanics are verified per harness and are uneven.** Claude Code has a real `--append-system-prompt` (append-only, `--print` mode); Codex has **no append flag at all** (must ride `developer_instructions`/`model_instructions_file` under a 32 KiB AGENTS.md cap); OpenCode exposes `session.prompt` + plugin hooks; Kimi/MiniMax expose `--skills-dir`/`--agent-file` + MCP. These asymmetries, plus the volatility of undocumented flags, are the primary execution risk.

---

## 1. Executive Summary

**The inversion thesis.** Today every coding-agent harness — Claude Code, Codex CLI, OpenCode, Kimi Code, MiniMax CLI ("MMX") — owns skill selection through an opaque, budget-constrained, probabilistic algorithm that matches the user's turn against skill *frontmatter descriptions only*, never the skill body, and silently drops skills when a context budget overflows. PSP currently sits *downstream* of this: we author ~140 skills, install them into harness directories, and hope the harness surfaces the right one. This spec inverts that relationship. PSP becomes the **instruction plane of record**: it performs its own out-of-harness skill selection (intent classification over the full skill registry), plans injection per harness, injects load-bearing skill bodies directly into the harness's prompt-assembly surface, serves a `request_skill` callback for emergent mid-phase needs, and logs every selection decision to an Activation Ledger under the goal DID. Harness skill directories become **projections** of the PSP registry, not sources of truth.

**What changes.** The Skill Router is added as Plane-2.5 inside the existing three-plane goal architecture (portable Goal Contract JSON → thin in-harness `psp-goal-runner` compliance skill → out-of-harness Rust Goal Supervisor in UAR). At phase-dispatch time — when the Supervisor spawns a fresh harness session for a phase, Ralph-style — the Router selects skills for *that phase's intent*, injects them through the harness's native prompt surface, and records the decision. This composes cleanly with the goal abstraction: fresh-session phasing already gives us a clean context per phase, and the Router decides what instructions populate that context. The native `/goal` primitives in Claude Code (v2.1.139+, released May 12 2026) and Codex remain fast paths *within* a phase; the outer Supervisor stays contract-of-record.

**Expected effect (honest uncertainty ranges).** For Travis's ~140-skill library on Claude Code's default 1% listing budget, the majority of skills are silently non-triggering at any given moment (the budget math in §2.1 is unambiguous on this; the exact fraction must be confirmed against his actual window in M0). The Router eliminates that failure mode by construction: selected skills are *present in context* with 100% reliability, not surfaced probabilistically. But **presence is not compliance.** We expect the Router to shift the dominant failure mode from "the right skill was never in context" — a selection/recall failure the harnesses own and hide — to "the right skill was in context but the model under-attended to it," an adherence failure shaped by position, competing instructions, and disposition. Net task-outcome improvement is uncertain and must be measured per harness via M0 baselines before any efficacy claim is made. I will not assert a headline percentage until the Activation Ledger produces one. The confident claims are narrow and structural: **deterministic presence** and **complete observability**, neither of which any harness provides today.

---

## 2. Problem Statement (with evidence)

### 2.1 Harness selection is opaque, budget-bound, and silently lossy

**Claude Code (verified).** Skill selection routes on the frontmatter `description` field only; the model never reads the body to decide. The skill listing sent to the model has a hard character budget governed by `skillListingBudgetFraction`, introduced quietly in **v2.1.129** (confirmed by ClaudeFast: *"Claude Code 2.1.129 added skillListingBudgetFraction, silently dropping skills past 1% of context"*), defaulting to `0.01` (1% of the context window). A companion setting `skillListingMaxDescChars` defaults to 1536 and truncates individual descriptions before the budget is applied. When the combined listing overflows, Claude Code **drops descriptions starting with the least-used skills** — eviction ranking is reported as roughly `usageCount × 0.5^(days/7)`, so a brand-new skill scores zero and is first to be dropped. The user sees only a fleeting startup warning; a real one from claude-code issue #56710 reads: *"122 descriptions dropped (full descriptions kept for most-used skills) (5.3%/1% of context)… run /skills to disable some, or raise skillListingBudget."* An evicted skill remains invocable by explicit `/skill-name` slash command, but is invisible to the model's auto-selection pass. Critically, the budget is calculated against a fixed ~200K-token baseline rather than the model's actual window (issue #57941), so 1M-context users get ~5× less room than the docs imply.

The scale math is empirically grounded. Alexey Pelykh's measurement (GitHub gist, conducted Dec 4–5 2025 with Claude Opus 4.5, contributed to claude-code issue #11045) found: *"Claude Code has an undocumented ~16,000 character budget for skill metadata. With typical 263-char descriptions, only ~42 skills fit… 21 skills (33%) were completely hidden from the agent—it couldn't discover or invoke them."* (~109 chars of XML overhead per skill; compressing descriptions to ≤130 chars fits ~67.) Back-of-envelope from other practitioner sources: each skill consumes ~75–150 tokens in the listing; at the default 1% on a 200K-baseline calculation, roughly 15–25 skills survive before truncation. **Travis runs ~140 skills.** The conclusion is not in doubt: at the default budget, most of the library is silently dark at any moment — only the precise fraction on his specific context window is a measurement task for M0.

**Codex CLI (verified).** Instructions come from AGENTS.md files, concatenated root→cwd (nearest-wins; `AGENTS.override.md` beats `AGENTS.md` at each level), capped at 32 KiB (`project_doc_max_bytes`); Codex **skips empty files and stops adding content once the combined size reaches the cap, truncating silently with no warning.** Skills (SKILL.md, adopted December 2025, stored in `~/.agents/skills/` or `~/.codex/skills/`) are, per practitioner reports, static prompt injection with no live selection observability; files are truncated if long (community guidance: keep under ~4KB for reliable loading). Skills, MCP, subagents, and plugins compose into a five-layer stack (plugins were elevated to first-class in v0.117.0, March 26 2026).

**OpenCode (verified).** At each model step, OpenCode advertises permitted skills that have a description and do not set `opencode/autoinvoke: false`; per the v2 docs, *"the advertisement contains only each skill's ID, name, and description; it does not add every skill body to the prompt."* Body is added only when the model calls the `skill` tool with an exact ID. Same description-only routing pathology, same lack of provenance.

**Kimi Code / MiniMax (verified).** Both support SKILL.md directories (`--skills-dir`; `~/.minimax/skills` via `MINIMAX_SKILLS_DIR`) and `/skill:<name>` / `/skill <name>` invocation, but selection is either manual slash-command or model-discretion; no operator-facing record of what was active. Kimi Code also ships a native goal mode (`/goal status|pause|resume|cancel|replace|next`).

### 2.2 Zero activation observability (the gap no vendor fills)

No harness emits a machine-readable record of *which skills were considered, which were selected, which were dropped for budget, at what version, and where in the prompt they landed.* The operator cannot answer "was skill X active in phase 3, and at what position?" except by reading transcripts and inferring. This is the gap the Activation Ledger closes, and it is the most defensible part of the whole proposal.

### 2.3 Position and adherence effects are real but bounded — treat position as second-order

The foundational reference is Liu, Lin, Hewitt, Paranjape, Bevilacqua, Petroni & Liang, **"Lost in the Middle: How Language Models Use Long Contexts,"** TACL vol. 12, pp. 157–173 (2024), doi:10.1162/tacl_a_00638: *"performance is often highest when relevant information occurs at the beginning or end of the input context, and significantly degrades when models must access relevant information in the middle of long contexts, even for explicitly long-context models."* This is corroborated mechanistically by "Found in the Middle: Calibrating Positional Attention Bias" (2024), which traces the effect to positional attention bias and RoPE long-distance decay.

**But the evidence is mixed at instruction scale, and the spec must not over-weight position.** "Boosting Instruction Following at Scale" (Elder, Duesterwald & Muthusamy, IBM T.J. Watson, submitted Oct 16 2025) attributes instruction-following degradation primarily to *"the degree of tension and conflict that arises as the number of instructions is increased"* rather than to position, and its "Instruction Boosting" method improves IF-rate by up to 7 points for two instructions and up to 4 points for ten. Counting-Stars (2403.11802) could not strongly corroborate lost-in-the-middle beyond 16K. **Conclusion for the spec:** prioritize *presence* first; treat *position* as a second-order optimization tuned empirically via the ledger, not a load-bearing a-priori guarantee. Minimize the number and mutual tension of injected instructions — that is better-supported leverage than positional placement.

---

## 3. Architecture

### 3.1 Placement within the three-plane goal architecture

The Skill Router is **Plane 2.5**: it lives in UAR alongside the Rust Goal Supervisor and is invoked at each phase dispatch. Data flow per phase:

1. Supervisor decides to spawn a fresh harness session for phase *P* of goal *G* (Ralph-style).
2. Supervisor hands the Router: the Goal Contract, phase *P*'s objective/scope, the target harness identity, and its computed context budget.
3. **Intent Classifier** produces an intent feature vector for phase *P* (hybrid retrieval over the registry, optional LLM refinement).
4. **Injection Planner** solves a budget-constrained tiered selection: full-body (Tier 1), name+compressed-summary (Tier 2), deferred/callback-available (Tier 3).
5. **Injection Driver** for that harness materializes the plan through the harness's native prompt surface (flags/files/SDK).
6. Supervisor spawns the session; the `request_skill` MCP service is registered so the in-harness model can pull Tier-3 skills mid-run.
7. Every decision is written to the **Activation Ledger** under `did:uar:goal/<G>/phase/<P>`.
8. On phase completion, the **Compliance Evaluator** tests, per Tier-1 skill, whether the verifiably-present skill was actually complied with, feeding the PMPO v2 Evolution Loop.

Text data-flow diagram:

```
GoalContract ─┐
Phase intent ─┼─▶ [Intent Classifier] ──features──▶ [Injection Planner] ──InjectionPlan──▶ [Harness Injection Driver] ──▶ fresh harness session
Harness+budget┘                                          │                                        │
                                                         ▼                                        ▼
                                                 requires/conflicts                        request_skill MCP  ◀── model asks mid-run
                                                 graph closure                                    │
                                                         │                                        ▼
                                                         └──────────────▶ [Activation Ledger] ◀── denials/calls logged
                                                                                 │
                                             post-phase artifacts ──▶ [Compliance Evaluator] ──▶ PMPO v2 Evolution Loop
```

### 3.2 Components

- **PSP Skill Registry** — single source of truth. Full skill bodies indexed (not just descriptions), content-hash versioned, immutable versions, Cedar-gated. Aligns with UAR's existing capability-gated WASM skill registration flow (declared capabilities validated against actual import surface). Backed by surreal-memory-server's existing HNSW + BM25 hybrid index.
- **Intent Classifier** — hybrid retrieval (HNSW vector + BM25) fused via Reciprocal Rank Fusion (Cormack, Clarke & Büttcher, SIGIR 2009, doi:10.1145/1645953.1646039; **k=60**, the original default now standard across OpenSearch/Elasticsearch/Azure AI Search/Weaviate/Qdrant, with benchmarks landing in k∈[40,80]) over full skill bodies, optionally refined by a small classifier (local Qwen via candle-vllm, or Claude Haiku).
- **Injection Planner** — budget-constrained knapsack-style tier solver.
- **Per-harness Injection Drivers** — one adapter per harness (§4.4), dispatched through `knowme:harness` / BossFang.
- **request_skill callback service** — an MCP tool served by the Supervisor.
- **Activation Ledger** — SurrealDB provenance store, OpenTelemetry GenAI-compatible.
- **Compliance Evaluator** — LLM-judge-with-rubric + deterministic postcondition checks.

### 3.3 Cargo crate boundaries (capability inversion preserved)

The Supervisor and Router must not be able to perform write actuation. Proposed crates:
- `psp-registry` — registry types, content-hash versioning, hybrid-index client. No actuator deps.
- `psp-router-core` — Intent Classifier + Injection Planner. Pure logic; depends only on `psp-registry` and classifier-client traits. No harness deps, no write actuators.
- `psp-injection-drivers` — per-harness adapters. Depends on harness process/SDK surfaces (read/spawn only).
- `psp-ledger` — SurrealDB writer + OTel exporter.
- `psp-compliance` — evaluator, read-only over artifacts.
- `psp-request-skill-mcp` — the callback MCP server; depends on `psp-router-core` + Cedar, returns bodies, cannot mutate the workspace.

Capability inversion at the Cargo level: `psp-router-core` and the Supervisor crate declare no dependency on any write-actuator crate, enforced the same way UAR already validates declared-vs-actual WASM import surfaces.

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
  conflicts_with: [skill_id],      // negative-selection set
  capabilities: [capability],      // Cedar-gated, validated vs import surface
  compliance_criteria: [ ... ],    // PSP frontmatter extension, see 4.7
  embedding: vector,               // HNSW
  bm25_terms: sparse,
}
```

### 4.2 Router pipeline contract

Input: `(goal_contract, phase, harness_id, context_budget_tokens)`. Output: `InjectionPlan` + a persisted `ActivationRecord`.

Pipeline: intent features → hybrid retrieval (RRF over vector+BM25) → optional LLM rerank → dependency-graph closure (`requires`) → conflict pruning (`conflicts_with`) → tier assignment under budget → plan.

**Why full-body indexing beats description-only:** the harness pathology is precisely that descriptions are lossy proxies for bodies. Indexing bodies lets a short phase-intent query match on load-bearing procedural detail that never appears in a 1024-char description. For skill bodies that exceed a single chunk, chunk on markdown H2/H3 boundaries and index chunk-level, aggregating to skill-level via max-chunk score (BM25 degrades on sub-page chunks, so keep chunks section-sized, not sentence-sized).

**Latency/cost envelopes (targets, to be validated):**
- Pure hybrid retrieval: single-digit-to-low-tens of ms on CPU; ≈$0. (Consistent with the OATS result that outcome-aware refinement runs "within single-digit millisecond CPU budgets.")
- + Haiku rerank: +hundreds of ms + small token cost; use only when the retrieval confidence margin is thin.
- Fine-tuned local classifier (Qwen / ModernBERT-class, à la vLLM Semantic Router): tens of ms; worth it only once ledger data density warrants — mirroring the OATS finding that learned re-rankers *hurt or match baseline when outcome data is sparse relative to the tool-set size*. Start with the zero-cost hybrid retrieval; add learned components only when data justifies.

### 4.3 Injection plan schema

```
InjectionPlan {
  harness_id,
  budget_tokens,
  tier1_full_body: [ {skill_id, version, content_hash, target_position} ],
  tier2_summary:   [ {skill_id, version, compressed_summary, target_position} ],
  tier3_deferred:  [ {skill_id, version} ],   // callback-available only
  rejected:        [ {skill_id, reason} ],    // budget | conflict | low_score
  injection_method: enum,
  total_injected_tokens: int,
}
```

Budget model per harness: `budget = context_window − goal_contract − STATUS.md − headroom`; Tier 1 fills by descending selection score until the body-injection sub-budget is exhausted; overflow demotes to Tier 2 (name+compressed summary), then Tier 3 (deferred). **Keep Tier 1 small** — see the token-cost model in §5/§8.

### 4.4 Per-harness Injection Driver table (verified mechanics)

| Harness | Primary injection surface | Append vs replace | Notes / limits |
|---|---|---|---|
| **Claude Code** | `--append-system-prompt "<bodies>"` (only valid with `--print`/`-p`); Agent SDK `systemPrompt:{type:"preset",preset:"claude_code",append:"..."}` | **Append** (preserves default coding identity, tool guidance, safety) | `--system-prompt`/`--system-prompt-file` *replace* the entire prompt and are mutually exclusive with each other; append can combine with either. `--append-system-prompt` and `--system-prompt` cannot both be set. |
| **Claude Code (hook path)** | `UserPromptSubmit` hook → stdout auto-injected as context; `SessionStart` (source `startup`/`resume`/`clear`/`compact`) stdout auto-injected via `hookSpecificOutput.additionalContext` | Additive | Special events whose stdout is auto-added to context. `UserPromptSubmit` default timeout 30 s; on timeout the additionalContext is **discarded silently** (a notice appears in transcript as of v2.1.196). Use `SessionStart(compact)` / `PreCompact` for C2 re-injection. |
| **Codex CLI** | config.toml `model_instructions_file` (**REPLACE** — "Replacement for built-in instructions instead of AGENTS.md") and `developer_instructions` (**APPEND** — "Additional developer instructions injected into the session," as role=developer messages); set at runtime via `--config`/`-c`; custom-agent `.toml` layers via `agents.<name>.config_file` | See cells | **No `--append-system-prompt`/`--system-prompt` flag exists** — issues #11588 (closed, unimplemented, no PR) and #11117 requested it and were not fulfilled. The plain `instructions` config field is "reserved for future use" — do not use. `experimental_instructions_file` is the legacy name for `model_instructions_file`. AGENTS.md layering capped at 32 KiB, silent truncation. MCP supported (shared host config). |
| **OpenCode** | Server/SDK `client.session.prompt({ path:{id}, body:{ parts:[{type:"text",text:"<bodies>"}] } })`; agent config `prompt` field (`{file:...}`); `OPENCODE_CONFIG_CONTENT` env for custom-agent system prompt | Agent `prompt` **replaces** provider prompt; session `parts` are **additive** user-context | System prompt assembled in `session/prompt.ts` from a provider `.txt` (anthropic.txt / beast.txt / gemini.txt / codex_header.txt / qwen.txt) + AGENTS.md/CLAUDE.md walk. `chat.message` / `session.compacted` plugin hooks available for injection *and* for logging which skills loaded. |
| **Kimi Code** | `kimi --print -p "<prompt>"`; `--agent-file` custom agent with `system_prompt_path`; `--skills-dir`; `--mcp-config-file` | Agent file sets fixed system context; `-p` is additive user message | No dynamic-initial-prompt-then-interactive mode (issue #2240 open). `/skill:<name>` reads SKILL.md and sends as prompt. `--output-format` only with `--prompt`. MCP supported. |
| **MiniMax CLI** (unofficial Rust `minimax-cli`, Hmbown) | `MINIMAX_SKILLS_DIR`, `~/.minimax/mcp.json`, `config.toml`; `/skills` + `/skill <name>` | Additive prompt composition | **MCP tools execute without TUI approval prompts** — trust boundary; only enable servers you trust. Compaction knobs (`MINIMAX_COMPACTION_*`) exposed as env. Distinct from MiniMax's *official* `mmx-cli` (media/generation, agent-skill-installable, no MCP needed). |
| **UAR-native** | Direct — Router composes the SkillService prompt via the NativeSkill trait; full control over position and provenance | N/A (reference implementation) | This is the reference path; every harness driver is a lossy approximation of it. |

### 4.5 request_skill MCP tool schema

```
tool request_skill {
  input:  { capability_description: string, phase_id: string, reason: string },
  output: { skill_id, version, content_hash, body, cedar_decision } | { denied, reason }
}
```

Abuse/loop guards: per-phase call cap (default 10); dedupe identical `capability_description` within a phase (return cached body, no re-classification); denial when Cedar rejects the capability for this goal/phase scope; every call and denial logged to the ledger. Cedar policy shape authorizes `action == "request_skill"` on `resource == skill:<id>` under `principal == goal:<G>` with **scope-narrowing only** (never widening) — consistent with UAR's existing delegation-chain discipline. This is the mitigation for the recall gap of dispatch-time classification, but it depends on the model *knowing to ask* — see §8.

### 4.6 Activation Ledger schema

```
ActivationRecord {
  goal_id (DID), phase_id, harness_id, timestamp,
  intent_features: {...},
  skills_selected: [ {skill_id, version, content_hash, tier, injection_method,
                      prompt_position, token_offset_start, token_offset_end, token_cost} ],
  skills_rejected: [ {skill_id, reason, score} ],
  request_skill_calls: [ {capability_description, resolved_skill_id, cedar_decision, ts} ],
  compliance_results: [ {skill_id, method, passed, evidence_ref} ],  // filled post-phase
}
```

**Fields that make outcome-correlation actually work:** `content_hash` (so a compliance regression can be tied to a specific body version), `prompt_position` + `token_offset_*` (so position effects can be measured, not assumed), `tier`, and `token_cost` (so cost/benefit per skill is derivable). OTel GenAI mapping: emit as spans with `gen_ai.system_instructions` (injected bodies), `gen_ai.operation.name = "skill_selection"`, and skill content as span **events** — *not* attributes — to respect the convention's size/PII guidance (attributes are always indexed; events can be dropped at the Collector). Export via OTLP so the surreal store plus optional Braintrust/Datadog/W&B Weave can consume it. Operator UI: per-goal timeline showing, per phase, exactly which skills were active, at what version/hash, in which tier and position, plus the rejected set and mid-run requests — the observability no harness provides.

### 4.7 Compliance-criteria authoring format (SKILL.md PSP extension)

Add PSP-namespaced frontmatter (harness-ignored — the Agent Skills spec mandates that unknown frontmatter fields are dropped; only `name`/`description` are required, with `license`/`compatibility`/`metadata`/`allowed-tools` optional):

```yaml
x-psp-compliance:
  - id: C-postcondition-1
    kind: deterministic        # machine-checkable
    check: "grep -q 'CHANGELOG' STATUS.md"
  - id: C-rubric-1
    kind: llm-judge
    rubric: "The commit was split into semantic groups; no `git add .` was used."
```

`deterministic` checks run in `psp-compliance` (read-only); `llm-judge` checks feed a rubric to an LLM judge that receives **only artifacts, never generation history** — consistent with the S-01–S-08 sycophancy-correction gate. Rubric-based LLM-judge is well-supported (≈80–90% agreement with human raters when rubrics are specific; calibrate against a small human-annotated set, and be aware of verbosity/self-preference bias per the Judge Reliability Harness literature).

### 4.8 Suppression / stub format

Harness skill directories become projections. For each PSP skill, the generated harness stub:
- Sets `disable-model-invocation: true` where supported (Claude Code confirmed — it "keeps a skill loaded but invisible to Claude's auto-selector"; OpenCode **ignores** this field per issue #11972, so there set `opencode/autoinvoke: false` or strip the description).
- Strips the description to a pointer (`"Managed by PSP Router — do not auto-invoke"`) so it contributes ~0 to the listing budget and cannot double-activate.
- Retains user-invocability (`/skill-name`) as an escape hatch.

**Live-hazard caveat:** on Claude Code, `disable-model-invocation: true` currently also blocks *explicit* slash-command invocation in some builds (issues #26251, #43809), and blocks subagents from loading a parent-referenced skill. The stub generator must detect harness version and fall back to description-stripping if the build exhibits this bug.

### 4.9 C1/C2/C3 redefinition (PAGS-SPEC-PSP-IP-002 component definitions)

- **C1 — was: trigger reliability via description engineering → now: Deterministic Skill Routing.** PSP owns selection out-of-harness; presence is guaranteed by construction, not by keyword-tuning descriptions against an opaque budget.
- **C2 — was: compaction re-anchor → now: mostly subsumed by fresh-session phasing.** Retained as `PreCompact` / `SessionStart(compact)` re-injection for long native-`/goal` phases that don't get a fresh session per phase. (OpenCode's ecosystem already has compaction-reinjection plugins — e.g. plugins that listen for `session.compacted` and re-inject the skills list — a pattern the driver can reuse rather than reinvent.)
- **C3 — was: tiered activation / compliance eval → now: Injection Tiering + Compliance-on-Ground-Truth.** Tiering is the Injection Planner; compliance is measured against *verified presence* (we know the skill was in context, at what position and version), and results feed the PMPO v2 Evolution Loop (Compile→Evaluate→Optimize→Promote) to optimize router weights and tier assignments against measured outcomes — the same outcome-aware refinement principle as OATS, applied to our own selection function.

---

## 5. Implementation Plan (M0 measurement-first)

Gated by the M1-first measurement discipline; each milestone maps to OpenSpec artifacts (proposal.md / tasks.md / design.md / spec-delta.md) and single-writer discipline.

**M0 — Measurement (no build).** Instrument the existing setup. From harness transcripts, compute the baseline native activation rate: for a sample of phases where a skill *should* have fired, how often did it? Build the token-cost model using measured SKILL.md body sizes. **Token-cost anchors:** description/metadata ≈100 tokens/skill (always loaded); recommended body ceiling <5,000 tokens (agentskills.io spec: *"Instructions (<5000 tokens recommended)… Keep your main SKILL.md under 500 lines"*); real-world observed per-skill body consumption **3,000–5,500 tokens** (claude-code issue #14882, via secondary — verify before formal quotation); Codex practical truncation ~4KB (~1,000 tokens); Claude Code Read-tool hard-fail at 10,000 tokens (~10KB, anthropics/claude-plugins-official #995); post-compaction per-skill budget 5,000 / combined cap 25,000 tokens (secondary — verify). **Exit:** baseline activation rate + cost model published; the "most skills dark" estimate confirmed or corrected against Travis's actual 140-skill listing on his actual context window.

**M1 — Registry + Classifier (`psp-registry`, `psp-router-core`).** Register skills, stand up hybrid retrieval, produce InjectionPlans offline. **Exit:** on a labeled phase-intent set, top-k retrieval recall ≥ agreed threshold; plan generation < target latency; RRF k tuned in [40,80].

**M2 — UAR-native driver + Ledger.** Wire Router into SkillService via NativeSkill; write ActivationRecords; OTel export. **Exit:** native path fully Router-driven; ledger renders in operator UI.

**M3 — Claude Code + Codex drivers.** `--append-system-prompt` (Claude) / `developer_instructions` (Codex) injection; stub generation; A/B against native selection with ledger instrumentation. **Exit:** Router-selected presence = 100% on both; measured task-outcome delta reported (not assumed).

**M4 — OpenCode / Kimi / MiniMax drivers + `request_skill`.** **Exit:** callback service live with Cedar + loop guards; all five harnesses driven.

**M5 — Compliance Evaluator + PMPO loop.** **Exit:** compliance telemetry closing the loop; router weights/tiers optimized against outcomes.

Repos: registry/router/drivers/ledger → `universal-agent-runtime`; skill bodies + compliance-criteria authoring + stubs → `prometheus-skill-pack`; harness dispatch → `librefang/BossFang` fork; goal integration → `know-me-system`.

---

## 6. Migration Plan (~140 skills)

1. **Extraction.** Parse each SKILL.md; frontmatter → registry metadata; body → indexed body; content hash → v1.0.0 immutable.
2. **Graph authoring.** Populate `requires`/`enhances` from existing capability cards; author `conflicts_with` for skills with incompatible conventions (negative selection).
3. **Compliance-criteria debt.** Authoring `x-psp-compliance` for 140 skills is the largest migration cost. Stage it: deterministic checks first (cheap, high-value), rubrics for load-bearing skills next, tail deferred.
4. **Stub generation.** Emit suppressed projections into each harness directory (§4.8), version-gated for the Claude Code `disable-model-invocation` bug.
5. **Transitional three-mode instrumentation.** Run pre-injection (Router) / callback / residual-native-triggering concurrently, with the ledger comparing Router vs native selection, so the efficacy claim is *earned from data* before native triggering is disabled.

---

## 7. Framing

**What PSP now claims.** "PSP owns skill *selection* and *observation* on every harness. Selected skills are present in context deterministically, and every selection decision is recorded under the goal DID with version, position, and cost."

**What PSP must NOT claim.** That injection guarantees compliance. **Presence ≠ adherence.** Attention, prompt position, competing/conflicting instructions (the better-supported degradation driver per the IBM instruction-boosting work), and RL-trained harness disposition (Codex's persistence bias vs Claude's consent-orientation) still shape whether a present instruction is followed. The exact line: **"PSP owns skill selection and observation on every harness; harnesses retain influence only over execution disposition."**

**Positioning vs vendors.** This is not adversarial to Anthropic/OpenAI; it is the layer they structurally cannot provide — cross-harness, operator-owned selection and provenance. It mirrors their own direction — Anthropic's Tool Search Tool / `defer_loading` (which lifted Opus 4 MCP accuracy from 49% to 74%, Opus 4.5 from 79.5% to 88.1%, at ~85% token reduction) and the progressive-disclosure design of the Agent Skills open standard — but generalizes it across harnesses and adds the provenance they omit. It compounds with the goal abstraction: the Supervisor already owns verification and phasing; owning the instruction plane means the contract-of-record now governs *what the agent knows*, not just *what it must achieve*.

---

## 8. Risks and Open Questions (with the scenario that hurts Prometheus)

1. **Efficacy is unproven.** The confident wins are presence + observability; net task-outcome improvement is uncertain. *Scenario:* we ship the inversion, presence goes to 100%, but adherence doesn't improve because the bottleneck was always disposition — and we've absorbed 140 skills of migration debt for a structural win the market undervalues. *Mitigation:* M0 baseline + M3 measured delta gate the efficacy narrative.

2. **Token cost of body-injection.** Full bodies cost far more per session than a description listing (observed 3,000–5,500 tokens each; recommended <5,000). Tier 1 must stay small. *Scenario:* over-eager Tier-1 selection blows the budget and crowds out the Goal Contract itself — the very artifact of record. *Mitigation:* budget model reserves Goal Contract + STATUS.md + headroom first; Tier-1 is what's left, not what's wanted.

3. **Injection-flag stability / ToS.** `--append-system-prompt`, Codex `developer_instructions`, hook-stdout injection are undocumented-to-semi-documented and move fast (the entire `skillListingBudgetFraction` regime appeared in weeks). *Scenario:* a harness update silently changes append semantics and every in-flight phase gets a malformed prompt. *Mitigation:* per-driver version detection; the ledger catches drift by construction. Maintain a **drift-surface inventory** (per harness: flag/file/field, version last verified, fallback).

4. **Recall gap of dispatch-time classification.** Skills whose need emerges mid-phase are missed by pre-injection; `request_skill` mitigates but depends on the model knowing to ask. Published tool-retrieval accuracy at scale is sobering: Anthropic's own Tool Search moved Opus 4 from 49→74%, and a third-party production test (Growth Method) concluded *"60% retrieval accuracy isn't production-ready when agents need to reliably take real-world actions."* *Scenario:* the model doesn't ask, the tail skill stays dark, and we've merely relocated the recall gap rather than closing it.

5. **Double-injection / conflict.** If suppression fails and both Router and native fire, conflicting conventions collide — and instruction conflict is the best-evidenced adherence-killer. `conflicts_with` + stub suppression mitigate; the Claude Code `disable-model-invocation` bug (§4.8) is a live hazard requiring the version-gated fallback.

6. **Classifier as a new failure surface.** The Router can mis-select. Unlike the harness, it's observable and optimizable — but a stale/bad embedding index degrades every phase silently until the ledger surfaces it. *Mitigation:* keep BM25 and vector indexes in sync (a known operational footgun); alert on selection-confidence collapse.

7. **Codex 32 KiB / no-append constraint.** With no append flag, Codex injection rides `developer_instructions` (append, role=developer) or `model_instructions_file` (replace) under the 32 KiB AGENTS.md cap; large Tier-1 plans may not fit. **Open question:** is `model_instructions_file` (replace) safe, or does replacing Codex's base prompt discard too much harness scaffolding to be worth it? Lean toward `developer_instructions` (additive) as the default Codex path and reserve replace for controlled experiments.

8. **Open question — native `/goal` phases.** Native `/goal` (Claude 2.1.139+, Codex, Kimi) runs multi-turn *within* a phase and does not spawn a fresh session per turn, so C2 compaction dilution reappears. Is the retained `PreCompact`/`SessionStart(compact)` re-injection a winnable fight, or should native `/goal` be treated as a fast-path-only that the outer Supervisor always overrides with fresh-session phasing when instruction fidelity matters? Recommendation: treat native `/goal` as an *inner accelerator* the Supervisor may use for a bounded number of turns, never as the contract-of-record — consistent with PAGS-SPEC-PSP-IP-001.

---

### Verified vs practitioner-reported (source-quality note)
- **Verified (primary/first-party):** Claude Code `--append-system-prompt` semantics and `skillListingBudgetFraction`/`skillListingMaxDescChars` (Claude Code docs + GitHub issues); Codex AGENTS.md 32 KiB cap and absence of an append flag (OpenAI config reference + issues #11588/#11117); OpenCode advertisement-vs-body loading (opencode.ai v2 docs); Anthropic Tool Search accuracy figures (anthropic.com engineering); Agent Skills progressive-disclosure spec (agentskills.io); lost-in-the-middle (Liu et al., TACL 2024); RRF (Cormack et al., SIGIR 2009); OTel GenAI conventions (opentelemetry.io).
- **Practitioner-reported (treat as indicative, confirm in M0):** the exact 3,000–5,500-token per-skill consumption and 25,000-token post-compaction combined cap (secondary blogs citing claude-code issue #14882); the `usageCount × 0.5^(days/7)` eviction formula; the ~15–25-skills-survive back-of-envelope. The ~16,000-char budget / 33%-hidden figure is a reproducible independent measurement (Pelykh gist) but on a specific Claude Code build — re-measure on Travis's version.