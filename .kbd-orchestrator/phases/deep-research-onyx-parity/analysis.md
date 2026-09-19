# Analysis — deep-research-onyx-parity

**Phase:** `deep-research-onyx-parity`
**Date:** 2026-09-08
**Mode:** stack specified (Rust + bash, the pack's existing stack — no stack-discovery needed)
**Research budget:** 4 tiers available; **Tier 1 + Tier 2 sufficed**. 15 queries, ~19 minutes (within the 8-per-tier / 20-minute cap). Tier 4 not needed.
**Inputs:** `assessment.md` (5/5 goals NOT MET; V1–V4 resolved), `docs/deep-research/deep-research-skill-vs-onyx-report.md`

---

## 1. What the research actually established

The assessment resolved *where things are*. Analyze answers *what to build with*.
The decisive move was reading Onyx's own source rather than trusting the report's
summary of it, because the entire target architecture rests on Onyx's constants.

### 1.1 Onyx's mechanism, verified at source (not from the report)

| Claim in the report | Verified? | Evidence |
|---|---|---|
| `MAX_ORCHESTRATOR_CYCLES = 8`, 4 for reasoning models | **YES** | `dr_loop.py:97,100` |
| Force report at 30 min | **YES** | `dr_loop.py:85` — `DEEP_RESEARCH_FORCE_REPORT_SECONDS = 30 * 60` |
| Final report capped at 20K tokens | **YES** | `dr_loop.py:80` — `MAX_FINAL_REPORT_TOKENS = 20000` |
| Worker force-report at 12 min | **YES** | `research_agent.py:91` — `RESEARCH_AGENT_FORCE_REPORT_SECONDS = 12 * 60` |
| Worker cycle cap | **YES** | `research_agent.py:269,280` — `MAX_RESEARCH_CYCLES` |
| Orchestrator has **no** search/fetch tools | **YES** | `dr_mock_tools.py` defines exactly four: `generate_plan`, `research_agent`, `generate_report`, `think_tool`; `get_orchestrator_tools()` returns only those |
| `think_tool` exists solely to force a reasoning turn | **YES** | `dr_mock_tools.py:112` — `THINK_TOOL_RESPONSE_MESSAGE = "Acknowledged, please continue."` |
| Citation merge is deterministic, no LLM | **YES** | `citation_utils.py::collapse_citations` is a pure function; docstring: reuses the existing number "if a citation refers to a document that already exists … matched by document_id" |
| "Deviate from the plan" is explicit | **YES** | `orchestration_layer.py:70-71,162,206` |
| Intermediate dossier capped at 10K tokens, with a note that ~5K performs better | **YES** | `research_agent.py:93` — `MAX_INTERMEDIATE_REPORT_LENGTH_TOKENS = 10000`, preceded verbatim by `# May be good to experiment with this, empirically reports of around 5,000 tokens are pretty good.`; applied at `:149` as `max_tokens=` |
| Worker hard timeout 30 min (distinct from the 12-min force-report) | **YES** | `research_agent.py:88` — `RESEARCH_AGENT_TIMEOUT_SECONDS = 30 * 60` |
| **≤3 parallel research agents** | **YES, but prompt-only** | `orchestration_layer.py:88,116,181,197` — stated **four times** in prose. There is **no code cap**; parallelism runs through `run_functions_tuples_in_parallel` (`research_agent.py:82,666`) with no arity limit |

**The single most useful finding.** Onyx repeats its own concurrency limit four
times in prompt text because a prompt cannot enforce it. A model that ignores the
instruction spawns four or ten workers and Onyx has no defence. Our scheduler
holds the cap in code, so this is a place where the pack can be *strictly better*
than the system it is copying, not merely equal — and it costs nothing extra.

### 1.2 Repository facts

| Fact | Value |
|---|---|
| `onyx-dot-app/onyx` | reachable, 31,975 stars, pushed 2026-09-08, license **NOASSERTION** |
| `GQAdonis/onyx-app` | a real fork of the above, pushed 2026-09-08 — the tree the report read |

**License caution.** GitHub reports `NOASSERTION` for Onyx, meaning its licence
is not a recognised SPDX identifier and cannot be assumed permissive. This
matters for exactly one thing: **prompt text**. The report proposes copying
Onyx's dossier prompt "almost verbatim". Any verbatim reuse of Onyx prompt
strings needs a licence check first. Every *mechanism* (bounded cycles, isolated
workers, deterministic merge keyed by document identity) is an idea, not
copyrightable expression, and is safe to reimplement.

---

## 2. Build-vs-adopt calls

### 2.1 The thread scheduler — **BUILD, on primitives already present**

The report's revised D-1 proposes a `HarnessRunner` with `Semaphore(3)` +
`JoinSet` and per-task timeout. Research finding: **the crate already depends on
`tokio` with `features = ["full"]`** (`Cargo.toml:25`), which includes
`sync::Semaphore`, `task::JoinSet`, and `time::timeout`. Neither is used yet.

**No new dependency is required for bounded concurrency.** This is a decisive
argument against the report's `drt-000` workspace split as a *precondition*: the
scheduler is roughly 150 lines against APIs already compiled into the binary.
Splitting four crates before any threading behaviour exists is restructuring
ahead of evidence.

The process-spawn half is also already built: `job/spawn.rs` and `job/daemon.rs`
carry harness resolution, `Command::new`, prompt-on-disk, env inheritance,
500 ms polling, token-gated event ingest, cancel, and exit-code mapping. Thread
workers are a second consumer of that code, not new infrastructure.

### 2.2 Dispatch strategy — **BOTH, with harness-native first**

| Option | Verdict | Reasoning |
|---|---|---|
| **A. In-session subagents** | **ADOPT first, gated by a smoke test** | Zero new infrastructure. The pack already ships `tools:` frontmatter on four research agents (`agents/*.md`) and documents Agent-tool duties (`deep-research/SKILL.md:260`, `adversarial-review/SKILL.md:280`) — that much is citable in-repo. **What is *not* established in-repo is parallel subagent dispatch:** no pack artifact documents a fan-out primitive, and the harness capability I relied on is not verifiable from this repository. The first thread change must therefore open with a smoke test that actually dispatches two workers concurrently and proves isolation, before the scheduler is planned against it |
| **B. Process-level workers** | **ADOPT second** | True isolation, kill-on-timeout (which Onyx's Python threadpool explicitly cannot do), per-thread logs. Required for `exhaustive`. Builds on `spawn.rs`. **Blocked by carried-in defect D-A** until the launchd `PATH` is fixed |
| **C. Native runner (UAR kernel + liter-llm)** | **REJECT this phase** | The assessment established UAR has exactly one crate and it is this pack's own submodule; there is no kernel to build against. `UAR-REM-002` does not exist. liter-llm 1.18.2 is genuinely available as a path dependency, but availability of one half does not make the design viable |

### 2.3 Workspace split (`drt-000`) — **REJECT as a precondition, revisit later**

`prometheus-research` is one `[lib]` + `[[bin]]` with no `[workspace]` stanza.
The report puts a four-crate split first. Nothing in the five goals requires it,
the scheduler needs no new dependency that would motivate it, and the pack's
one-Cargo-build-at-a-time constraint makes a large restructure expensive to
iterate on. **Defer**; if compile times or dependency bleed become real problems
after threading lands, split then with evidence.

### 2.4 Citation merge — **BUILD, reusing our own claim ids**

Onyx keys its merge on `document_id`. The pack already emits **content-addressed
claim ids** shared by `build-graph.sh` and `detect-contradictions.sh`
(change-rah-010). The merge should key on canonical URL for sources and the
existing content-addressed hash for claims. This is a pure function like Onyx's,
and it is the enforcement point for the director's no-search rule on harnesses
that ignore `tools:`.

### 2.5 Benchmark (G5) — **ADOPT DeepResearch Bench's criteria, BUILD the rest**

The first vet caught G5 defaulting to build with no candidate evaluated. Research
corrects that: **`Ayanami0730/deep_research_bench` is Apache-2.0**, 827 stars,
last pushed 2026-05-11, and it is the reference implementation of the very
benchmark Onyx's standing is quoted against. It ships
`data/criteria_data/criteria.jsonl`, the 100-task `data/prompt_data/query.jsonl`,
the English and Chinese criteria prompts, and `deepresearch_bench_race.py`.

**Adopt the criteria, judge prompts, and a query subset as data** (permissive
licence, attribution required), **pinned at commit `469cce54`** (2026-05-11) so a
RACE score is reproducible against a fixed rubric; **do not** adopt the Python
runner wholesale into a bash/Rust suite. The repo has been quiet about four
months, which for a benchmark is closer to a feature than a defect: a rubric that
moves makes scores incomparable across runs. FACT metrics and the pack's verified-claim ratio still need
building, so coverage is partial.

The reason this matters: a home-grown rubric produces a number comparable to
nothing. Scoring against the published criteria is what makes "on par with Onyx"
a falsifiable claim rather than a slogan — which is G5's entire purpose.

### 2.6 Multi-pass report (G3) — **BUILD, no adoptable alternative**

Also evaluated rather than defaulted. Onyx offers nothing to adopt here: its
final report is one call capped at 20K tokens over a history that is silently
truncated, which is precisely the failure G3 exists to prevent. The pack has no
report-assembly or eval-harness skill to reuse. And the invariant that makes the
split safe — cited claim ids must be a subset of assigned claim ids — depends on
the pack's own content-addressed claim ids, which no external library knows
about. Genuine build, recorded with its reasoning so spec does not re-litigate it.

### 2.7 Reflection / think-tool — **BUILD, and make it durable**

Onyx's `think_tool` returns a fixed acknowledgement and is discarded. Writing the
same reflection to `threads/<tid>/reflections.md` costs nothing extra and yields
both the reasoning-forcing effect and an audit trail the sidecar can cite. Strict
improvement over the source design.

---

## 3. Decisions

| # | Decision | Verdict | Provenance |
|---|---|---|---|
| **D-01** | Concurrency primitives | `tokio` `Semaphore` + `JoinSet` + `time::timeout`, already a dependency. No new crate | Tier 1: `Cargo.toml:25` |
| **D-02** | Concurrency cap enforcement | **In code, not in prompt.** Onyx states its ≤3 rule four times in prose with no code cap; we enforce it structurally | Tier 1: `orchestration_layer.py:88,116,181,197` vs no cap in `research_agent.py` |
| **D-03** | Dispatch order | Harness-native subagents first, process-level workers second, both supported. Native/UAR runner rejected this phase | assessment V2; report D-1 |
| **D-03a** | Strategy A is **provisional** | Sequential single-agent dispatch is citable in-repo; **parallel** subagent dispatch is not. Strategy A is adopted *subject to* a smoke test in the first thread change proving two workers run concurrently with isolated contexts. If that fails, strategy B (process-level, already proven by rah-011's headless run) becomes the primary path and D-A's `PATH` fix becomes a blocking prerequisite rather than an enabler | adversarial round 2, finding 2 |
| **D-04** | Workspace split | **Deferred**, not adopted as a precondition | Tier 1: no new dep needed; one-build constraint |
| **D-05** | Merge key | Canonical URL for sources; existing content-addressed claim hash for claims | change-rah-010 artifacts |
| **D-06** | Reflection artifact | Durable `reflections.md`, not an ephemeral ack | Tier 1: `dr_mock_tools.py:112` |
| **D-07** | Budgets | Adopt Onyx's numbers as **starting** values (8/4 cycles, 12-min thread, 30-min job), all verified at source, all tunable per depth | Tier 1: `dr_loop.py:80,85,97,100`; `research_agent.py:91` |
| **D-08** | Onyx prompt text | Reimplement mechanisms freely; **do not copy prompt strings verbatim** without a licence check — Onyx is `NOASSERTION` | Tier 1: GitHub licence field |
| **D-09** | D-A / D-B ownership | Fix **in this phase**: D-A blocks dispatch strategy B outright. C-01 requires same-change manifest regeneration; C-05 requires bash 3.2 for launchd-reachable scripts | assessment §3 |
| **D-10** | G5 benchmark source | **Adopt** `deep_research_bench` criteria, judge prompts, and a query subset (Apache-2.0, attribution). Build FACT metrics and verified-claim ratio on top. Do not port the Python runner | Tier 1: GitHub API licence + tree listing |
| **D-11** | G3 report split | **Build.** No adoptable alternative in Onyx (single truncating call) or in the pack (no such skill), and the claim-set invariant is specific to our id scheme | Tier 1: `dr_loop.py:80`; `ls -d skills/*/eval-harness` empty |
| **D-11b** | D-B is itself a **C-01 event** | The stub-vs-repo driver mismatch (1920 vs 30718 bytes) *is* generated output out of sync with its source. Fixing it means rerunning the plugin-generation generator, proving byte-identical output across two runs, and passing `check:distribution`, `validate:codex`, and `check-harness-adapters.js` in the same change — the same discipline the predecessor's rah-011 applied. C-03 (below) is a *separate* question about docs | assessment §3; rah-011 precedent |
| **D-11c** | **Correction to D-11b, made at spec:** the D-A plist edit does **not** trigger a services-manifest regeneration | `scripts/generate-service-manifest.mjs:35-36` reads only `shared/launchagents/*.plist` and `shared/systemd/*`. The research plist lives at `substrate/prometheus-research/` and is not an input; `shared/services.manifest.json` lists eleven services and `com.prometheus.research` is not among them. D-11b over-applied C-01 to the plist half. The republish half stands. Separately: that the research daemon is a launchd service the manifest does not track is a real gap, raised as an open question in drt-007 rather than fixed as a side effect | adversarial spec review, round 1 finding 8 |
| **D-12** | Does a D-B republish trigger **C-03**? | **No, provided the fix is regeneration only.** C-03 governs Codex *plugin surface* documentation — `docs/codex-plugin.md` and the CLAUDE.md Codex section — and is triggered by a change to a plugin manifest, MCP declaration, hook registration, or installer contract. Republishing the generation so the installed driver matches the repo alters none of those: the driver is skill payload, not plugin surface. **But** if the fix is widened to a version bump or a manifest edit, C-03 applies and those two documents must be updated in the same change. Precedent: `kbd-control-plane-recovery/plan.md:74` reasons identically | assess handoff deferral, resolved here |

---

## 4. Open questions for spec

1. **Depth ceiling enforcement.** Two levels is agreed. Is the merge-script gate
   (a source in a dossier absent from that thread's `sources.json` is CRITICAL)
   sufficient, or does the director also need a structural guard?
2. **Budget defaults per depth.** Onyx's constants are single-valued. The pack
   has three depths. Proposed: shallow 2 director cycles, deep 4, exhaustive 8,
   with thread budgets constant. Needs a cost estimate before it is fixed.
3. **`--max-parallel 1` as a deliverable.** With D-02 enforcing the cap in code,
   a serial mode is one argument, not a feature. Confirm it is in scope.
4. **Bench composition.** RACE needs judge prompts and a judge model; the pack
   routes `judge` to `k3` and `critic` to MiniMax-M3 (preflight `ok`, 2 distinct
   models). Is a 10-task subset enough to be meaningful, and do we run Onyx
   locally for a true head-to-head, or report our own numbers with Onyx's
   published figure marked **unverified** (assessment V4)?
5. **Dossier token cap.** Onyx caps intermediate reports at 10K
   (`research_agent.py:93`, now verified in §1.1) with an inline note that ~5K
   performs better empirically. Which do we start at? A cap that is too generous
   costs tokens on every thread; too tight reintroduces the summarisation loss the
   dossier exists to avoid.

---

## 5. Confidence and limits

**High confidence** on Onyx's mechanism: every load-bearing constant and both
key mechanisms were read from the source repository, not from the report.

**Explicitly not established:**
- Parallel in-session subagent dispatch. Nothing in this repository documents it; the claim came from harness context, not evidence. D-03a gates strategy A on a smoke test rather than assuming it.
- Onyx's benchmark standing (assessment V4) — still **unverified**; no bench was run here.
- `UAR-REM-002` (assessment V2) — the identifier does not exist; the native runner is rejected on that basis, not deferred pending it.
- Whether a 10-task RACE subset is statistically meaningful — open question 4.

**Budget:** Tier 1 (GitHub source + local tree) and Tier 2 (direct source
reading) answered every question that mattered. Tier 3 registry health checks
were unnecessary because the only new runtime capability rides on `tokio`, which
is already pinned. Tier 4 was not reached.

---

## Unresolved review findings

Adversarial review: k3 judge, producer `claude-opus-5`, artifact mode, two rounds
(cap). Receipts under `review/analyze/` (`round1/`, then round 2). Both rounds
returned **verdict PASS**; both passed the findings sycophancy gate at 0.0 strict.

### Round 1 (PASS: 2 WARNING, 1 SUGGESTION) — all accepted

| # | Finding | Disposition |
|---|---|---|
| 1 | G3 and G5 had **no candidate at all**: every `fit_for_gap` covered only G1/G2/G4, so two of five goals silently defaulted to BUILD | **Accepted — the most valuable finding of the stage.** Researched both. G5 changed verdict: `deep_research_bench` is Apache-2.0 and ships the RACE criteria, judge prompts, and query set, so the criteria are **adopted** rather than reinvented. G3 stays build, but now with the reasoning recorded (§2.6) instead of by omission |
| 2 | The C-03 question the assess handoff deferred to analyze was silently dropped | **Accepted.** Answered as D-12: republishing the driver does not touch the Codex plugin surface, so C-03 does not apply *provided* the fix is regeneration only; a version bump or manifest edit would trigger it |
| 3 | Open question 5 quoted a 10K/~5K Onyx constant absent from the verified table, in an artifact whose method is source verification | **Accepted.** Verified at source and added to §1.1: `research_agent.py:93` sets `MAX_INTERMEDIATE_REPORT_LENGTH_TOKENS = 10000`, preceded verbatim by the comment that ~5K performs better |

### Round 2 (PASS: 2 WARNING, 1 SUGGESTION) — all accepted

| # | Finding | Disposition |
|---|---|---|
| 1 | D-B was analysed only for C-03 (docs) while D-B *is itself* a C-01 generated-artifact desync | **Accepted.** Added D-11b: the fix is a C-01 regeneration event requiring byte-identical proof and the drift validators in the same change, following rah-011's precedent |
| 2 | The claim that the harness exposes parallel `Workflow` primitives had **no verifiable evidence** in the packet | **Accepted, and it was the right catch.** Sequential Agent-tool dispatch is citable in-repo (`SKILL.md:260`, `adversarial-review/SKILL.md:280`); **parallel** dispatch is not documented anywhere in this repository. The claim came from harness context, which is not evidence a plan can rest on. Strategy A is downgraded to **adopt-provisional** (coverage 0.5) and gated by D-03a: the first thread change must smoke-test concurrent dispatch before the scheduler depends on it. If it fails, strategy B becomes primary and D-A's `PATH` fix becomes blocking |
| 3 | The benchmark adoption was unpinned, so a RACE score would not be reproducible | **Accepted.** Pinned at commit `469cce54` (2026-05-11). The ~4-month quiet period is recorded as acceptable for a benchmark, where a moving rubric would make scores incomparable |

No finding was rejected in either round. Round 1 changed a verdict (G5 build →
adopt) and round 2 downgraded another (strategy A → provisional), which is the
review doing exactly what it exists to do at a stage whose output is decisions.

---

## Spec-stage review outcome (recorded here because it corrected an analyze decision)

The spec-stage adversarial review ran two rounds over the whole change set, as
the spec skill requires (a spec is only coherent against its siblings). Fifteen
findings, **all accepted, none rejected**. Two corrected decisions made at
analyze:

- **D-11c** (already recorded above): the D-A plist edit does *not* trigger a
  services-manifest regeneration. `generate-service-manifest.mjs` reads only
  `shared/launchagents` and `shared/systemd`; the research plist is neither.
- **The dispatch runtime was underspecified.** Round 1 finding 3 caught that
  drt-002's code-enforced cap and drt-003's in-session director are **two
  different runtimes**: a `tokio::Semaphore` in the scheduler process cannot bound
  subagent dispatches inside a harness session. D-02's "enforced in code" claim is
  now scoped to strategy B (process workers), with strategy A's bound stated
  honestly as advisory-plus-merge-gate. This does not weaken D-02 — the
  improvement over Onyx still holds on the path the scheduler owns — but the
  earlier wording claimed an enforcement the architecture cannot deliver on both
  paths.
