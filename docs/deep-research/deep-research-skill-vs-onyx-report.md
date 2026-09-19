# Deep Research: Prometheus Skill Pack vs. Onyx — Gap Analysis and Subagent Architecture Plan

**Date:** 2026-09-07
**Scope:** `skills/research/deep-research` (prometheus-skill-pack, post `research-agent-hardening`, 11/12 changes complete) vs. `backend/onyx/deep_research/` + `backend/onyx/tools/fake_tools/research_agent.py` (GQAdonis/onyx-app @ `06aa2b0`, 2026-09-04)
**Goal:** Make the skill-pack deep-research skill at least as good as Onyx on long, multi-thread research jobs while keeping the verification/provenance guarantees Onyx does not have.

---

## 1. Executive summary

The two systems are optimized for different things, and the fastest path to "as good or better" is to graft Onyx's *execution topology* onto the skill pack's *evidence contract*.

**Onyx wins on research execution.** Its entire architecture is one idea applied ruthlessly: a search-less orchestrator dispatches up to 3 isolated research agents per cycle, each agent runs its own search → read → think loop in a fresh context, and each returns a *deliberately long, fact-only intermediate dossier* (up to 10K tokens) with inline citations. A deterministic step then renumbers citations by `document_id` so the final writer sees one unified citation map. That is the whole trick, and it is the reason Onyx holds a top-tier spot on DeepResearch Bench (RACE overall ~54, in the same band as the leaders; it has been reported as #1 at points in 2026).

**The skill pack wins on everything after execution.** Stage contracts enforced by a driver, resumable checkpoints, claim labels (`verified`/`unverified`/`blocked`/`inferred`), verifier-before-reviewer ordering, adversarial review, Feynman gate, a derived (never chosen) `verification_status`, schema-validated packages, provenance sidecars, model routing, hooks, and a Rust daemon. Onyx has none of this: no source scoring, no claim-level verification, no contradiction resolution beyond "flag it," no persistent package, no resume.

**The skill pack's weak point is exactly Onyx's strong point.** Stages 02–04 (search → retrieve → collect) run once, sequentially, in a single context, against a plan that is frozen at stage 01. There is no per-sub-question isolation, no gap-driven follow-up searching, no reflection loop, no time budget, and `threaded = true` in `skill.toml` is documented as "future intent." Worse, evidence is compressed into `graph.json` claims *before* synthesis, and stage 09 writes the whole report in one pass — two lossy bottlenecks that Onyx specifically avoids.

**Recommendation in one sentence:** Replace the linear 02–04 block with a *thread scheduler* (constrained director + isolated research workers + deterministic merge), keep 05–08 unchanged, and split stage 09 into outline → parallel section writers → coherence editor, all bound by the existing claim ledger so accuracy does not degrade as output length grows.

---

## 2. What Onyx actually does (from the code)

Everything below is read from `dr_loop.py`, `research_agent.py`, `orchestration_layer.py`, `research_agent.py` (prompts), and `citation_utils.py`.

### 2.1 Pipeline

| Phase | Mechanism | Hard limits |
|---|---|---|
| **Clarification** (optional) | One LLM step with a single tool `generate_plan`. If the model asks a question instead of calling the tool, the turn ends and waits for the user. Skipped when the query is >3 sentences or `SKIP_DEEP_RESEARCH_CLARIFICATION`. | ≤5 questions |
| **Plan** | No tools. Output is a numbered list, ≤6 steps, each "a standalone exploration question … that can be researched independently." A USER-typed reminder is appended to stop the model from answering instead of planning. | ≤6 steps |
| **Orchestrator loop** | System prompt rebuilt every cycle with `{current_cycle_count}/{max_cycles}` and the plan. Tools: `research_agent(task)`, `generate_report()`, and `think_tool(reasoning)` for non-reasoning models. `tool_choice=REQUIRED`, `max_tokens=1024` (the orchestrator is only allowed to emit tool calls). **No search or fetch tools.** | 8 cycles (4 for reasoning models); ≤3 `research_agent` calls in parallel; 30-min wall clock forces `generate_report` |
| **Research agent** (worker) | Receives *only* the task string — no plan, no chat history, no sibling context. Fresh `msg_history`. Tools: `web_search` (≤3 queries/call), `open_urls` (batch), internal `search`, plus `think_tool` and `generate_report`. Nudged to open pages after every search. Only one tool *type* per turn. | 8 cycles; 12-min force-report; 30-min hard timeout returns a placeholder string |
| **Intermediate report** | Worker's history is re-prompted with `RESEARCH_REPORT_PROMPT`: "free of formatting or commentary … facts only … no title, no sections, no conclusions … EXTREMELY THOROUGH … several pages long … don't lose any details … flag statements that seem untrustworthy or contradictory … cite every fact inline." Citation markers kept as-is (`CitationMode.KEEP_MARKERS`). | 10K tokens (comment: ~5K is empirically good) |
| **Merge** | `collapse_citations()` — deterministic. Renumbers each worker's `[n]` markers into the global map keyed by `document_id`, reusing an existing number when the same document was already cited by another worker. | pure function, no LLM |
| **Final report** | `FINAL_REPORT_PROMPT` + plan as reference. Only the *cited* documents are passed as `final_documents`. History is the orchestrator history (all intermediate reports + think-tool traces), truncated to `max_input_tokens`. | 20K output tokens; `DR_REPORT_LLM_TIMEOUT_S` (60s default, configurable) |

### 2.2 Design properties worth copying

1. **Constrained orchestrator.** It cannot search, so it cannot start answering; it is forced to write self-contained task briefs. The prompt explicitly says the worker "has no additional context about the user's query, research plan, other research agents, or message history."
2. **Two levels, never three.** Query → orchestrator → worker. Each extra hop re-summarizes and distorts; Onyx refuses the third hop.
3. **Lossless handoff by verbosity.** Intermediate reports are *told to be long*. The system pays tokens to avoid the information loss that summarization causes. This is the single most important accuracy mechanism in the codebase.
4. **Explicit reflection for non-reasoning models.** `think_tool` after every search+read batch, and between every `research_agent` call at the orchestrator level. The tool result is literally "Acknowledged, please continue." — it exists only to force a reasoning turn. A token processor streams the `reasoning` argument as if it were native reasoning.
5. **Gap-driven re-planning.** The orchestrator prompt: "new discoveries from research may lead to a deviation from the original research plan … ensure that the new directions are thoroughly investigated." The plan is a reference, not a contract.
6. **Budgets at every level.** Cycle caps, per-agent force-report at 12 min, hard timeout at 30 min with a timeout callback that returns a placeholder instead of killing the run, orchestrator force-report at 30 min, output token caps on every LLM call to stop null-token loops.
7. **Deterministic citation unification.** `document_id`-keyed renumbering means the same URL cited by three workers becomes one reference in the final report.
8. **Strict-provider invariants.** Every `tool_use` id gets a `TOOL_CALL_RESPONSE`, even on failure (synthetic failure message), so Bedrock/strict providers do not reject the next request.

### 2.3 Where Onyx is weak

- **No verification.** A worker's "flag it if it seems untrustworthy" is the entire trust model. No credibility scoring, no read-before-you-label, no dead-link handling, no claim labels.
- **Document-level citations, not claim-level.** `[3]` points at a document, not at a passage. FACT-style citation accuracy depends entirely on the model.
- **Single-pass final report** capped at 20K output tokens. For 8 cycles × 3 workers × 10K-token dossiers the input can exceed the context window; `construct_message_history` silently truncates oldest history, so early findings can drop out of the final synthesis.
- **No contradiction resolution.** Contradictions are flagged inside dossiers and left to the final writer.
- **Nothing persists** except chat messages and tool-call rows. No plan replay (a TODO in the file header), no package, no resume, no provenance.
- **No quality gate.** The report is delivered as generated.
- **Only three tools** (web search, open URL, internal search); the header TODO admits non-search tools are out of scope.
- **Orchestrator context is a single growing chat history.** Works up to ~8 cycles; would not scale to exhaustive jobs without the same truncation risk.

---

## 3. What the skill pack does today

### 3.1 Strengths (keep all of these)

| Capability | Where | Onyx equivalent |
|---|---|---|
| Stage contracts validated at every boundary; driver refuses to advance on invalid artifacts | `references/stage-contracts.md`, `run-research.sh` | none |
| Checkpoint + `--resume` that re-runs stages whose artifacts stopped validating | `checkpoint.json` | none |
| Claim labels `verified / unverified / blocked / inferred`; "verify meaning, not topic overlap"; "read before you label"; dead-link → `blocked` | `agents/source-verifier.md`, `okf-research-format.md` | none |
| Verifier-before-reviewer; review refused if `credibility.json` missing; `verification_status` derived by rule | SKILL.md "Agent tool duties" | none |
| Adversarial review between 09 and 10; Feynman gate (`learn-grade`) | driver, stage 09 | none |
| Per-agent `tools:` allowlists with advisory fallback on harnesses that ignore the key | agents/*.md | orchestrator has no tools by construction |
| Content-addressed claim ids shared by `build-graph.sh` and `detect-contradictions.sh` | change-rah-010 | none |
| Numeric + semantic contradiction detection with resolution precedence (authority → recency → consensus → escalate) | stage 06 | "flag it" |
| Sycophancy penalty on sources; vendor-doc −10 | stage 05 | none |
| Schema-validated package, provenance sidecar written on every exit path, OKF alignment | `research-manifest.schema.json`, `write-provenance.sh` | none |
| Rust daemon: headless harness spawn, SSE mirroring, `research_export` schema check, cancel | `headless-execution.md` | Onyx's own server, but no export |
| Model routing per stage | frontmatter | single LLM |
| Fixture-driven test suite (driver contract, scoring/graph, review) | `tests/` | none in DR module |

### 3.2 Gaps relative to Onyx (the work)

| # | Gap | Evidence | Consequence on long jobs |
|---|---|---|---|
| G1 | **Strictly sequential; no worker isolation.** Stage 02 searches *all* sub-questions in one context, stage 03 scrapes all URLs, stage 04 indexes all chunks. | `pipeline-architecture.md`: "do not assume parallelism"; `threaded = true` is "future intent" | One context carries 6–10 sub-questions × 25–50 results × chunks. At `exhaustive` this is exactly the context blow-up Onyx avoids. |
| G2 | **Plan frozen at stage 01; no gap-driven follow-up.** | Stage 01 writes `plan.md`; no later stage may search except the verifier, and only to locate a primary source | Findings that reveal a new direction are never pursued. Onyx explicitly re-plans. |
| G3 | **No reflection loop.** No think-tool equivalent for medium-tier (often non-reasoning) models used in 02–04. | model routing: 02–04 = medium | Search quality degrades for cheaper models; no recorded "why I searched next." |
| G4 | **Lossy compression before synthesis.** Evidence path is chunk → registry → `graph.json` claim → report. The report-synthesizer reads only `graph.json`, `citations.json`, `contradictions.json`. | `agents/report-synthesizer.md` Input section | The synthesizer never sees prose evidence. Nuance, caveats, numbers-in-context are lost. Onyx keeps several-page dossiers in the writer's context. |
| G5 | **Single-pass report.** Stage 09 writes `report.md` in one LLM call. | stage-09 SKILL.md | For a 10K-word exhaustive report, one call risks output truncation, section drift, and repeated content. |
| G6 | **No time or cycle budgets.** Only a 2-retry policy. | `pipeline-architecture.md` Retry Policy | A stalled scrape or a runaway search has no force-complete path. |
| G7 | **No clarification stage.** | — | Ambiguous queries produce confident, off-target plans. |
| G8 | **No cross-worker citation unification step** (because there are no workers yet). URL normalization exists only in stage 02 dedupe. | stage-02 step 3 | Needed once G1 is fixed. |
| G9 | **Retrieval is search-then-scrape, not search-read-think-search.** Result counts are fixed per depth (10/25/50 per question). | stage-02 Input table | Breadth without adaptivity; the worker never decides "I have enough" or "I need one more angle." |
| G10 | **No empirical benchmark.** Playbook targets (`Feynman > B+`, `verification rate > 90%`) are internal; no RACE/FACT run. | playbook §9.4 | "As good as Onyx" is unfalsifiable today. |

---

## 4. Target architecture: threads inside the contract

The design principle: **Onyx's topology for stages 02–04 and 09; the skill pack's contract everywhere.** Stage numbers stay stable so `checkpoint.json`, `check-research-package.sh`, the daemon mirroring, and the manifest schema keep working.

```
QUERY
  │
  ▼ Stage 00 — Clarify (new, optional; skipped headless)
  ▼ Stage 01 — Planner            (unchanged: no search; plan.md + sub-questions)
  │
  ▼ Stage 02–04 — RESEARCH THREADS (replaces search/retrieve/collect internals)
  │   ┌──────────────────────────────────────────────────────────┐
  │   │ research-director (no search/fetch; Read/Grep/Glob only)  │
  │   │  cycle 1..N:                                             │
  │   │    dispatch ≤3 research-worker (fresh context, task brief)│
  │   │    each worker: search → read → think, ≤8 cycles, 12 min │
  │   │    each writes threads/<tid>/{dossier.md,sources.json,   │
  │   │                               claims.json,reflections.md} │
  │   │    director reads dossiers, records gaps, re-plans        │
  │   │  merge-threads.sh (deterministic): registry + url-list +  │
  │   │    chunks + content-addressed claim dedupe                │
  │   └──────────────────────────────────────────────────────────┘
  ▼ Stage 05 — Verify  (unchanged contract; fan out verifier workers per source batch)
  ▼ Stage 06 — Resolve (unchanged)
  ▼ Stage 07 — Graph   (unchanged; now also links claims → thread dossier spans)
  ▼ Stage 08 — Cite    (unchanged)
  ▼ Stage 09 — REPORT (multi-pass)
  │    09a outline-architect  → report/outline.json
  │    09b section-writer ×K  → report/sections/NN.md  (parallel, fresh context each)
  │    09c assemble + citation check (script)
  │    09d coherence-editor   → report.md (Read/Edit only; claim-set diff must be empty)
  │    Feynman gate + adversarial review (unchanged)
  ▼ Stage 10 — Export (unchanged)
```

### 4.1 Stage 02–04: the thread scheduler

**Roles**

| Agent | Tools (frontmatter + restated) | Duty |
|---|---|---|
| `research-director` (new) | `Read, Grep, Glob, Write` (writes only `threads/index.json` and `plan.md` ledger) | Owns the cycle loop. Reads `plan.md`, dispatches workers with self-contained briefs, reads returned dossiers, writes a gap analysis to the plan ledger, decides next threads or `done`. **Never searches, never fetches.** |
| `research-worker` (new) | `WebSearch, WebFetch, Read, Write` (writes only under `threads/<tid>/`) | One thread. Fresh context. Receives the brief only. search → open pages → think → repeat. Ends by writing the dossier. |

**Worker brief** (the argument the director passes; mirror Onyx's "1–2 descriptive sentences" but add what the skill pack can give):

```json
{
  "thread_id": "t03",
  "task": "Investigate reported production failure modes of Qdrant at >10M vectors, including memory pressure, snapshot/restore, and cluster rebalancing, from 2025 onward.",
  "must_cover": ["failure symptoms", "root causes", "mitigations", "version affected"],
  "avoid": ["pricing", "benchmarks already covered by t01"],
  "recency_months": 18,
  "excluded_domains": ["reddit.com"],
  "budget": { "cycles": 8, "minutes": 12, "max_sources": 12 }
}
```

`avoid` is the one thing Onyx cannot do (its workers know nothing about siblings). The director can populate it because it has read the other dossiers, and it keeps the two-level rule intact — the worker still receives nothing but the brief.

**Worker loop rules** (put in `agents/research-worker.md`):

1. First call may be `search` without reflection; every subsequent `search` must be preceded by a `reflect` entry appended to `threads/<tid>/reflections.md` (what was found, what is missing, what query addresses it). This is the think-tool, made durable — you get Onyx's reasoning-forcing effect *and* an audit trail.
2. After any search that returns results, open at least the top 2–3 promising pages (Onyx's open-URL nudge).
3. Stop when: budget exhausted, or the reflection concludes coverage is complete, or two consecutive searches yield no new source.
4. On stop, write the dossier.

**Dossier contract** (`threads/<tid>/dossier.md`) — copy Onyx's prompt almost verbatim, it is the accuracy mechanism:

> Facts only. No title, no sections, no conclusions, no analysis. As long as necessary to preserve every relevant statement; several pages is expected. Include the context of each fact to avoid misattribution. Flag any statement that seems untrustworthy or contradicts another. Cite every fact inline as `[src:<sha8>]` where `<sha8>` is the content-addressed id from `sources.json`. Remove obvious duplicates and irrelevancies only.

Cap at ~8–10K tokens per dossier. Unlike Onyx, **persist it** — it becomes an artifact the report writers and the verifier can quote from.

**Structured sidecars written by the worker:**

- `sources.json` — `[{id: sha8(canonical_url), url, title, fetched_at, chunk_ids[]}]`
- `claims.json` — `[{id: sha(claim_text), text, source_id, quote: "<verbatim ≤40 words>", chunk_id, thread_id}]`. The `quote` field is what lets stage 05 do "verify meaning, not topic overlap" mechanically later.
- `chunks/` — the fetched page text, same `chunk-<n>.json` shape stage 03 emits today.

**Director cycle** (bounded like Onyx: `max_cycles` by depth — shallow 2, deep 4, exhaustive 8; ≤3 workers per cycle; job-level force-complete timer):

1. Cycle 1: one thread per sub-question (batched 3 at a time).
2. After each batch: read dossiers, append to `plan.md` a `## Coverage` table (sub-question × threads × status) and a `## Gaps` list.
3. Dispatch follow-up threads for gaps or new directions discovered ("deviate from the plan" is allowed and logged as a decision).
4. Stop when coverage is complete, budget is hit, or the last cycle "yielded minimal new information."

**Merge** (`scripts/merge-threads.sh`, deterministic, no LLM — the analogue of `collapse_citations`):

- Union all `threads/*/sources.json` keyed by canonical URL (reuse stage 02 normalization) → `sources/registry.json` and `sources/url-list.json` in the *existing* stage 02/04 shapes.
- Copy chunks → `sources/chunk-<n>.json` (existing stage 03 shape).
- Union `claims.json` by content-addressed id; keep all `(thread_id, source_id, quote)` provenance tuples per claim.
- Rewrite `[src:<sha8>]` markers in dossiers to global `[N]` numbers, keeping a `citation-map.json` so the same document is one number everywhere.

The driver's stage 02/03/04 validators then pass unchanged. That is why this is a refactor of *how* 02–04 are produced, not of the contract.

### 4.2 Stage 05: parallel verification

Keep the verifier agent exactly as written; add fan-out. The driver splits `registry.json` into batches of ~10 sources, dispatches one `source-verifier` per batch (each with the `claims.json` entries for its sources and the `quote` spans), and merges `credibility-partial-<k>.json` into `credibility.json`. Because each claim now carries a verbatim quote, the verifier's "read before you label" rule becomes checkable: the fetched chunk either contains the quote or the label is `unverified`.

### 4.3 Stage 09: long output without loss

The reason long reports degrade is that one context has to hold all evidence *and* generate all prose. Split the two.

| Pass | Agent | Input | Output | Tools |
|---|---|---|---|---|
| 09a | `outline-architect` | `plan.md`, `graph.json` claim index, dossier headers | `report/outline.json`: sections ↔ sub-questions ↔ claim ids, target word count per section, citation numbers already assigned | Read, Write |
| 09b | `section-writer` ×K (parallel, fresh context each) | its section spec, **only** the claims and dossier excerpts it references, the global citation map, style rules | `report/sections/NN.md` | Read, Write (its own file only) |
| 09c | script | all sections | `report/draft.md`; fails on orphan `[N]` markers, missing references, claims used outside their labels | — |
| 09d | `coherence-editor` | full draft | `report.md`: executive summary, transitions, cross-section dedupe, consistent terminology | Read, Edit (no Write of new claims) |
| gate | existing | `report.md` | Feynman grade, adversarial review, derived `verification_status` | unchanged |

Invariants that keep accuracy from drifting with length:

- Section writers may cite only claim ids listed in their section spec. The assembler diffs `{claim ids cited}` ⊆ `{claim ids assigned}`.
- The coherence editor may reorder, cut, and rewrite prose, but the assembler re-runs after 09d and requires the cited-claim set to be unchanged (or strictly smaller, with removed ids logged in `plan.md` decision log).
- Every `[N]` still resolves through `citations.json` → `graph.json` → claim label, so the existing rule "`verified`/`confirmed`/`checked` describe only verified claims" is enforced per section, not per report.
- Word budget by depth (shallow 1.5K, deep 4K, exhaustive 10K+) is distributed by the outline, so no single LLM call is asked for more than ~3K words.

This is the piece that lets the skill pack go *past* Onyx: Onyx's final writer is one 20K-token call over truncated history; this design has no ceiling on report length and no evidence truncation.

### 4.4 Budgets and failure semantics

Add to `checkpoint.json` and enforce in the driver (mirroring Onyx's numbers, tune later):

```json
"budgets": {
  "job_force_report_minutes": 30,
  "thread_minutes": 12,
  "thread_cycles": 8,
  "director_cycles": 4,
  "max_parallel_threads": 3,
  "dossier_max_tokens": 10000
}
```

- A thread that exceeds its budget writes whatever it has as a dossier with `status: partial` in `threads/index.json`; the director sees it and may re-dispatch.
- A thread that dies returns a synthetic dossier (`status: failed`, reason) so the director's ledger has a row for every dispatch — Onyx's "every tool_use gets a response" invariant, applied to files.
- Job-level force-complete skips remaining director cycles and proceeds to 05 with what exists; `verification_status` will derive accordingly. Nothing is silently dropped.

### 4.5 Stage 00: clarification (optional)

Only when the query is under ~3 sentences and stage 01's planner flags ambiguity (ask it to emit `clarification_needed: [...]` in `plan.md` frontmatter). On Claude Code use `AskUserQuestion`; otherwise `pmpo-elicit`. In daemon mode (`--daemon-job`) default to `--no-clarify` and record the assumptions the planner made in the decision log, exactly as `SKIP_DEEP_RESEARCH_CLARIFICATION` does in Onyx.

---

## 5. How subagents map to your harnesses

The skill already has the pattern (agents with `tools:` frontmatter, driver in runner or checkpoint mode). Two dispatch strategies, both should be supported:

**A. In-session subagents (default on Claude Code / Codex).** The harness running the driver dispatches `research-worker` as a subagent per thread. Pros: zero new infrastructure; `tools:` is enforced. Cons: parallelism and isolation depend on the harness; advisory only on Kimi/OpenCode/Cursor.

**B. Process-level workers (recommended for `exhaustive`, and the natural fit for `prometheus-research`).** The driver in runner mode, or the Rust daemon directly, spawns one headless harness per thread (`claude -p` / `codex exec`, same resolution as `headless-execution.md`) with `RESEARCH_THREAD_ID`, the brief on disk at `threads/<tid>/brief.json`, and a per-thread `harness.log`. A `tokio::sync::Semaphore(3)` plus `JoinSet` with per-task timeouts gives true parallelism, hard isolation, kill-on-timeout (which Python threads in Onyx explicitly cannot do), and per-thread logs. The daemon already polls `checkpoint.json` every 500 ms; add `threads/index.json` to the poll and emit one `agent.status` per thread state change so the `stage_timeline` / new `thread_panel` A2UI components show fan-out live.

Either way the director itself never gets search/fetch tools — that is the invariant to protect, and on advisory harnesses the merge script is the gate: any source in a dossier that is not in that thread's `sources.json` is a CRITICAL finding, the same rule you already apply to citations not in `citations.json`.

---

## 6. Proposed file changes

```
skills/research/deep-research/
  agents/
    research-director.md        NEW  (Read, Grep, Glob, Write[ledger only])
    research-worker.md          NEW  (WebSearch, WebFetch, Read, Write[threads/<tid>/ only])
    outline-architect.md        NEW  (Read, Write)
    section-writer.md           NEW  (Read, Write[own section only])
    coherence-editor.md         NEW  (Read, Edit)
    source-verifier.md          EDIT (accept a batch + claim quotes)
    report-synthesizer.md       RETIRE or keep as the `direct`-scale fallback
  scripts/
    dispatch-threads.sh         NEW  (runner-mode fan-out; process-level workers)
    merge-threads.sh            NEW  (deterministic registry/claims/citation merge)
    assemble-report.sh          NEW  (09c: concat + citation/claim-set checks)
    run-research.sh             EDIT (budgets, thread loop, 09 sub-passes, force-complete)
  references/
    thread-contracts.md         NEW  (brief schema, dossier rules, sidecar shapes, index.json)
    stage-contracts.md          EDIT (02–04 "produced by threads"; 09 sub-artifacts; budgets)
    schemas/thread-brief.schema.json, thread-index.schema.json, report-outline.schema.json  NEW
  templates/
    dossier.md, thread-brief.json, report-outline.json  NEW
  tests/
    thread-scheduler.sh         NEW  (fixture workers: complete, partial, failed, timeout)
    report-assembly.sh          NEW  (orphan marker, claim-set drift, label misuse)
    bench/                      NEW  (see §7)
substrate/prometheus-research/src/
  threads.rs                    NEW  (Semaphore + JoinSet scheduler, per-thread timeout/kill)
  a2ui/registry.rs              EDIT (add `thread_panel`)
openspec/specs/research-pipeline-execution/spec.md   EDIT (threads, budgets, 09 passes)
```

Package additions (all additive; schema `format_version` → `2.1.0`):

```
<package_id>/
  threads/index.json            # every dispatch: tid, task, status, cycles, minutes, sources_count
  threads/<tid>/brief.json | dossier.md | reflections.md | sources.json | claims.json | chunks/
  citation-map.json             # src sha8 → global [N]
  report/outline.json | sections/NN.md | draft.md
manifest.json: + threads_dispatched, threads_completed, threads_partial, director_cycles,
               budget_hit (bool), report_passes ["outline","sections","assemble","edit"]
```

---

## 7. Proving "as good as or better"

Onyx's claim rests on DeepResearch Bench (100 PhD-level tasks, RACE for report quality, FACT for effective citations and citation accuracy). Add `tests/bench/` that runs a 10-task English subset through both `deep` and `exhaustive`, scores RACE with the published judge prompts, and computes FACT from `citations.json` + claim labels. Track four numbers per run in `EVAL-RESULTS.md` (you already do this for the learn skills): RACE overall, effective citations, citation accuracy, and **verified-claim ratio** — the last is the metric Onyx cannot report and is your differentiator. Target for the first threaded release: match Onyx on RACE within noise, exceed it on citation accuracy.

---

## 8. Phased rollout (KBD-style)

| Change | Content | Depends on |
|---|---|---|
| **drt-001 thread contracts** | `thread-contracts.md`, schemas, templates, `merge-threads.sh` with fixture threads; stage 02–04 validators unchanged and passing on merged output | — |
| **drt-002 worker + director agents** | the two agent files, reflection rule, dossier prompt, in-session dispatch in the driver, budgets in `checkpoint.json`, force-complete | drt-001 |
| **drt-003 process-level scheduler** | `threads.rs`, `dispatch-threads.sh`, daemon thread mirroring, `thread_panel` | drt-002 |
| **drt-004 parallel verify** | batch fan-out, quote-span check, partial merge | drt-001 |
| **drt-005 multi-pass report** | outline → sections → assemble → edit, claim-set invariants, word budgets | drt-001 |
| **drt-006 clarify stage + bench** | stage 00, `tests/bench/`, `EVAL-RESULTS.md` baseline vs. Onyx numbers | drt-002, drt-005 |

drt-001, drt-004, and drt-005 are independent of each other and can run as parallel KBD changes; drt-002 is the critical path.

---

## 9. Trade-offs to decide up front

- **Cost.** Three parallel workers × 8 cycles × long dossiers is 3–6× the tokens of today's linear 02–04. Onyx accepts this; routing workers to the `medium` tier and directors/writers to `frontier` (your existing routing) is the mitigation. Add `--max-parallel 1` as a cheap mode.
- **Dossier length vs. context.** Persisting dossiers means the section writers can be fed excerpts rather than everything; that is the advantage over Onyx, but it depends on 09a assigning claims to sections well. Start with one section per sub-question.
- **Advisory harnesses.** On Kimi/OpenCode/Cursor the director's no-search rule is advisory. The merge-script gate (sources must originate from a worker's `sources.json`) is the enforcement, so make it a CRITICAL review finding from day one.
- **Contract stability.** Keeping stage numbers 02–04 and 09 means the daemon, the schema checker, and every existing fixture keep working. Resist renumbering.
- **Do not go three levels deep.** It will be tempting to let a worker spawn sub-workers for a large sub-question; Onyx's authors resisted it for a reason (each hop re-summarizes). Let the director dispatch another thread instead.

---

## Appendix A — Side-by-side

| Dimension | Onyx | Skill pack today | Skill pack proposed |
|---|---|---|---|
| Planner tools | none | none (Read/Grep/Glob) | unchanged |
| Orchestrator search | none (by design) | n/a (no orchestrator) | none (`research-director`) |
| Worker isolation | fresh context, brief only | none | fresh context, brief + `avoid` |
| Parallelism | ≤3 workers/cycle | sequential | ≤3 in-session or process-level |
| Re-planning | yes, per cycle | no | yes, logged in plan ledger |
| Reflection | think_tool (ephemeral) | none | `reflections.md` (durable) |
| Handoff | 10K-token fact dossier | 2K chunks → graph claims | dossier persisted + claims w/ quotes |
| Citation merge | deterministic by document_id | n/a | deterministic by canonical URL + claim hash |
| Source scoring | none | 5-dim + sycophancy | unchanged, parallel |
| Claim labels | none | verified/unverified/blocked/inferred | unchanged, quote-checkable |
| Contradictions | flagged in prose | detected + resolved + escalated | unchanged |
| Final report | 1 call, ≤20K tokens, history truncation | 1 call | outline + K sections + editor, no ceiling |
| Quality gate | none | Feynman + adversarial review | unchanged |
| Budgets | cycles + minutes at both levels | retries only | cycles + minutes, force-complete |
| Persistence / resume | none | package + checkpoint | + threads/, report/ |
| Benchmark | DeepResearch Bench (top tier) | none | `tests/bench/` RACE/FACT + verified ratio |

## Appendix B — Onyx source map (fork @ 06aa2b0)

- `backend/onyx/deep_research/dr_loop.py` — clarification, plan, orchestrator loop, forced report, limits (`MAX_ORCHESTRATOR_CYCLES=8/4`, `DEEP_RESEARCH_FORCE_REPORT_SECONDS=1800`, `MAX_FINAL_REPORT_TOKENS=20000`, min 50K input tokens)
- `backend/onyx/tools/fake_tools/research_agent.py` — worker loop (`MAX_RESEARCH_CYCLES=8`, 12-min force, 30-min timeout), intermediate report (10K tokens), `run_research_agent_calls` parallel runner, `collapse_citations` merge
- `backend/onyx/prompts/deep_research/orchestration_layer.py` — clarification/plan/orchestrator/final-report prompts, ≤3 parallel rule, "deviate from plan" rule
- `backend/onyx/prompts/deep_research/research_agent.py` — worker prompt, `RESEARCH_REPORT_PROMPT` ("EXTREMELY THOROUGH … several pages … facts only … flag contradictions")
- `backend/onyx/deep_research/dr_mock_tools.py` — `generate_plan`, `research_agent(task)`, `generate_report`, `think_tool(reasoning)` definitions
- `backend/onyx/deep_research/utils.py` — think-tool token processor (streams `reasoning` arg as reasoning)
- `backend/onyx/chat/citation_utils.py::collapse_citations` — document_id-keyed renumbering
- `backend/onyx/configs/chat_configs.py` — `DR_REPORT_LLM_TIMEOUT_S`, `SKIP_DEEP_RESEARCH_CLARIFICATION`

---

## Revision 2026-09-08 — after reading `substrate/prometheus-research`

**Findings from the crate.** Single `[lib]+[[bin]]`, no workspace. Dependencies: clap, axum 0.8, rmcp 1.8, reqwest (event emit/health only), jsonschema, chrono, uuid, nix. No liter-llm, ractor, search client, or LLM client. `lib.rs` exports `a2ui, agui, config, http_server, job, mcp_server`. `JobCheckpoint.tokens_used` / `sources_found` are never populated (daemon mirrors the package checkpoint, which carries no usage). `spawn.rs`/`daemon.rs` already implement harness resolution, prompt-on-disk, env inheritance, 500 ms polling, token-authenticated event ingest, cancel, exit mapping. The A2UI registry has eight components, none thread-aware. UAR and liter-llm could not be read (local MCP servers hung); those items remain assumptions.

**Revised decisions.**
- D-1: `trait ThreadRunner` with `HarnessRunner` (per-thread headless harness via existing `spawn.rs`; `Semaphore(3)` + `JoinSet`, per-task timeout + kill) shipped first; `NativeRunner` (UAR kernel + liter-llm) second, gated on verified UAR-REM correctness fixes. Bench both on identical briefs.
- D-2: job supervisor = existing `job` + `http_server`; director = harness session (later UAR agent) with no search tools; A2UI emitted by supervisor from `threads/index.json`.
- D-3: bash driver remains contract owner; stage 02 invokes `prometheus-research threads run --package <dir> --max-parallel 3`, which returns after `merge-threads` writes stage 02/03/04 artifacts in current shapes.
- D-4: liter-llm routing/spend only on the native path; harness path derives `tokens_used` from harness JSON usage output.

**Revised change list.** drt-000 workspace split (`research-core`, `research-threads`, `research-server`, `research-cli`) → drt-001 thread contracts + `merge-threads` in Rust → drt-002 HarnessRunner + `threads run` + driver hook → drt-003 director/worker agents + budgets. Parallel: drt-004 supervisor surface (`GET /api/v1/jobs`, thread SSE events, `thread_panel`, MCP `research_threads`), drt-005 multi-pass report, drt-006 parallel verify. Gated: drt-007 NativeRunner (blocked on UAR fixes). drt-008 bench against both runners.

**Open verifications.** liter-llm Rust API availability from UAR `Cargo.toml`/`versions.toml`; UAR-REM-002 status for the three correctness defects; nature of `universal-agent-runtime/crates/prometheus-skill-system` (nested checkout — submodule, worktree, or drifted copy of the skill pack).
