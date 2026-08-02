# Prometheus Skill Pack — Instruction-Plane Improvement Specification

**Document ID:** PAGS-SPEC-PSP-IP-001
**Date:** 2026-07-28 · **Revision:** 1.2 (production-convergence implementation record)
**Scope:** Architecture, functional specification, and implementation plan for closing the superpowers-derived capability gaps that survive evidence review, plus audit findings not covered by the in-progress Cross-Harness Control Plane work (Codex harness).
**Method:** Filesystem audit of the working tree at `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack`, comparative analysis against obra/superpowers, and web research to validate desirability before specifying anything. Confidence stated as ranges. Sources linked in §8.

---

## 0. Production-Convergence Status (rev 1.2)

This section supersedes older future-tense statements in §§1, 3–7 where they conflict. The instruction and control planes now share a production-convergence contract rather than proceeding as independent workstreams.

| Capability | Implemented state | Production status |
|---|---|---|
| Canonical workflow authority | `substrate/kbd-runtime` persists signed schema-v2 events in per-replica journals and folds them into one authoritative `project.loro` document per project. Compatibility JSON is a derived projection with causal-frontier metadata. | Implemented and covered by deterministic replay, migration, and convergence tests. |
| Durable ordering | Each acknowledged write holds one exclusive file lock across replay, frontier and idempotency validation, append, `fsync`, Loro import, and projection update. Sovereign Sync exchanges signed authoritative Loro deltas over iroh. | Local loopback, real peer exchange, crash recovery, torn-tail handling, and stale-frontier rejection are covered. |
| Device/event security | RFC 8785 canonical JSON, Ed25519 signatures, hash chaining, enrolled/revoked device checks, permission-protected REST bearer tokens, loopback binding, and platform/headless key-storage policies are implemented. | Local and simulated security contracts are implemented; external transport penetration and key-rotation acceptance still require production evidence. |
| Public control contract | Versioned command envelopes are shared by CLI, REST, MCP, and SSE. Typed phase/stage/change/task/completion/decision/blocker commands join status, pause, resume, cancel, CRDT claim acquire/renew/release, conflict resolution, audit, watch, migrate, and rollout gates. | Implemented. Remote mutations are device-signed and do not have an independent compatibility-file fallback. |
| Harness convergence | One capability manifest generates Claude, Codex, OpenCode, and Kimi adapters. Emergency pause is checked before runtime access. The pre-mutation fence was removed; adapters observe lifecycle events only. Prompt/Stop chains defer noncritical memory, summary, learning, and proposal work. | Claude and OpenCode active integration is covered locally. Generated Codex and Kimi artifacts still need real installed-host acceptance before release. |
| Compaction re-anchor | Native lifecycle events invoke one bounded renderer (4,800 characters, a conservative 1,200-token ceiling). There is no file sentinel on hosts that expose compaction events. | Implemented and fixture-tested. Sentinel fallback is reserved only for a host that genuinely lacks native lifecycle events. |
| Skill structure and inventory | `prometheus skill lint` validates 145 first-party skills, Agent Skills name/1,024-character description constraints, duplicate names, and description collisions. Clean installation verifies 145 unique payloads across 14 targets and rejects repository-specific absolute paths. | Structural gate passes 145/145. The weighted 0–100/prescription system described later in this document is intentionally not implemented because no activation evidence yet justifies mass rewriting. |
| Discovery budgets | `prometheus skill budget --harness … [--budget-chars …]` reports the exact 44,882-character inventory and refuses to invent a ceiling. Trace-backed records live in `evals/skill-activation/harness-budgets.json`. | **All four records remain explicitly unmeasured and therefore block a production-readiness claim.** |
| Skill evaluations | `evals/skill-activation/critical-36.json` contains 36 prompts for six critical skills: 6 explicit, 18 implicit, and 12 near-miss. `prometheus skill eval` schedules three trials per prompt and grades invocation, typed KBD traces, direct-write avoidance, lifecycle behavior, output contracts, per-skill 5/6 success, ≥90% implicit activation, 100% explicit invocation, and false positives. | Corpus and deterministic grader pass. **No live 108-trial trace exists for any harness yet; nightly cross-harness execution remains a release blocker.** |
| Migration and rollout | Migration uses the immutable project UUID, creates checksummed recoverable backups, imports one legacy state, labels uncertain phases `legacy-read-only`, and refuses mismatched legacy identities. Shadow comparison is read-only and byte-compares committed projections. Promotion requires 7 shadow days, 100 real mutations, 10,000 synthetic mutations, zero mismatches, then staged 3/3/7-day canaries. | Mechanism and threshold tests pass. Required elapsed-time and live-mutation evidence has not accrued. |

### Production release blockers

The code may ship only as shadow or opt-in canary until all of the following are evidenced:

1. Cross-process authenticated Loro delta transport over paired devices, including collision, disconnect, reconnect, and stale-frontier rejection tests.
2. Real native adapter installation and the pause-in-one/audit-in-another/resume-in-a-third scenario on Claude Code, Codex, OpenCode, and Kimi.
3. Trace-backed discovery budgets for all four harnesses, recorded against the current 145-skill/44,882-character inventory.
4. Three live trials of the 36-prompt corpus per harness meeting the committed thresholds.
5. Seven consecutive shadow days with ≥100 real and ≥10,000 synthetic mutations and zero unexplained projection mismatches, followed by the staged canaries.
6. External security review of transport authentication, enrollment/revocation, key rotation, REST token handling, snapshot recovery, and signed audit export.

The complete 145-skill scoring/rewrite and rationalization-harvesting program is a separate, evidence-gated follow-up. It must not be used to obscure the six release blockers above.

## 1. Scope Boundary — What This Document Excludes

The Cross-Harness Control Plane plan owns the **state/control plane**: event-sourced KBD runtime, lifecycle states, causal frontiers, CRDT claims, the Sovereign Sync `kbd-control:` domain, schema unification, CI consolidation, harness payload parity, and Stop-hook redesign. Audit of the working tree confirms its P0 operator-safety patch has substantially landed — `position-stop-gate.sh` now checks `stop_hook_active`, honors `.kbd-orchestrator/PAUSE` before any parsing, keys deduplication to session + state-revision fingerprint (explicitly excluding transcript size), recognizes suspended states, and is advisory-only. `waypoint-render.sh` implements suspended/terminal vocabularies and dual-casing reads. The mtime freshness check on `position.json` remains, consistent with P1 not yet landed.

**Verdict on that plan, corrected 2026-08-02:** the durable-journal diagnosis remains sound, while multi-project replication now uses a Loro authority, causal frontiers, and explicit CRDT claims rather than the superseded single-writer coordinator. Three historical flags follow:

1. **Replay determinism is under-specified.** "Byte-equivalent projections on repeated replay" requires canonical serialization (key ordering, number formatting). Specify RFC 8785 (JCS) or an equivalent canonical JSON form in the event store, or this acceptance test will flake.
2. **The two-device "observe within two seconds" acceptance test is network-dependent.** Bind it to loopback/simulated transport in CI; keep the wall-clock version as a manual acceptance run only.
3. **Resolved:** Loro is now the authoritative grow-only project event map, while per-replica journals are write-ahead ingestion logs. Deterministic folding, visible conflicts, signed adjudication, and causal frontiers make the merge semantics explicit. The runtime library is Rust.

Everything below is the **instruction plane**: whether skills fire, whether their content survives long sessions, and whether compliance is measurable. The two planes are orthogonal and can proceed in parallel without merge conflicts — the instruction plane touches `skills/*/SKILL.md` frontmatter, `shared/scripts/` prompt-side hooks, `tools/prometheus-cli`, and a new eval directory; the control plane touches `substrate/`, `.kbd-orchestrator` schemas, and steering hooks.

### 1.1 Amendment (rev 1.1) — The BossFang Dispatch Seam

BossFang (GQAdonis/librefang, workspace version 2026.7.11) is designated as the remote-dispatch surface for KBD runs. Audit of that tree confirms the dispatch layer already exists — 45 channel adapters, kernel with workflows/scheduler/RBAC/metering/budget, Merkle audit trail, WebAuthn, wasmtime-46 component sandbox, A2A/MCP/ACP (ACP pinned `=0.11.1` with session resume/fork), and SurrealDB pinned `=3.2.1` byte-identically with surreal-memory and UAR. This resolves the Archon inversion question **hybrid**: interactive harness sessions and BossFang-dispatched runs are signed mutators governed by project/replica identity, causal frontiers, claims, and conflict handling. Constraints this document now assumes and that the control-plane workstream should adopt:

1. **Single-writer preserved.** BossFang is a *client* of the KBD operator contract (`kbd_status/pause/resume/cancel/handoff` over MCP/REST/SSE) — never a second writer to `.kbd-orchestrator` state or the event journal.
2. **Adapter convergence is a named prerequisite.** The control-plane plan's harness adapters (in-harness steering: hooks → KBD events) and knowme's `knowme:harness` adapters (out-of-harness invocation: BossFang spawns/drives Claude Code/Codex/OpenCode) target the same harnesses from opposite directions. They must share one KBD event vocabulary and one identity/Cedar/audit plane. Add to the convergence list alongside HandDefinition ↔ UAR-AGENT-MD; do not let either workstream ship its adapter layer without this.
3. **Remote command scoping, deny-by-default.** Tiered: channel rendering of position/status is read-only (safe as a W1 notification-only Hand, shippable before control-plane P1); pause/cancel require authenticated operator; resume/dispatch require strong auth (WebAuthn) plus approval-gate echo to the channel.
4. **OFP carries no KBD authority.** OFP wire is plaintext-by-design per the librefang README; KBD mutations and journal replication travel only over the plan's chosen Loro/iroh Sovereign Sync domain. OFP may carry presence/notification at most.
5. **Accelerated decisions.** The librefang open-core boundary (already an open decision in the knowme strategy stack) gains a second consumer and should be decided before W2 dispatch work. `boss-uar-integration-plan.html` (v1.0, 2026-04-08) predates the Cherry Studio → librefang pivot and should be marked superseded to prevent agents building against its stale client architecture.

Instruction-plane consequence: `librefang-skills` is a third SKILL.md consumer with its own parser, loading semantics, and unknown description-budget behavior. C1 lint and budget therefore add BossFang as a target (see §4.1, §4.2, M1).

---

## 2. Research Findings — Which Superpowers Features Are Actually Worth Building

Each candidate was researched before being specified. Importance is rated by evidence strength × exposure of this codebase, not by how impressive the feature sounds.

### 2.1 Skill trigger reliability — **CRITICAL. Build it. Evidence is quantified and this repo's exposure is maximal.**

This was rated below the eval harness in my initial comparison. Research inverted that ranking:

- Community-measured baseline autonomous activation is roughly **50%** — "essentially a coin flip whether your carefully crafted skill will be used when relevant" ([Seleznov](https://medium.com/@ivan.seleznov1/why-claude-code-skills-dont-activate-and-how-to-fix-it-86f679409af1)).
- A 650-trial replication study found **88.9% overall activation but a 20× spread between description variants**: directive descriptions ("ALWAYS invoke when…") hit 100% in bare conditions while passive descriptions dropped to **37%** when hooks were present ([MCP.Directory summary](https://mcp.directory/blog/why-your-claude-skill-isnt-activating-2026-fixes)). The interaction effect matters for this repo specifically: prometheus-skill-pack is hook-heavy, and hooks *depressed* passive-description activation in that study.
- There is a **token budget for skill descriptions collected at session start; overflow silently drops skills** — "total description volume is what fills the budget. Collections that install a dozen skills at once are the usual way installs quietly cross the line" ([lazyskills troubleshooting](https://lazyskills.sh/troubleshooting/skills-not-triggering), [DEV root-cause analysis](https://dev.to/lizechengnet/why-claude-code-skills-dont-trigger-and-how-to-fix-them-in-2026-o7h)). **The skill pack installs ~140 payloads per harness.** This is the single most exposed installation profile for silent budget overflow that I found described anywhere. It is plausible — 60–80% likely in my estimate — that some prometheus skills are already being silently dropped from the candidate set in real sessions, which would present exactly as "the skill exists, tested fine in isolation, didn't fire."
- An audit of 214 community skills found **73% scored below 60/100** against Anthropic's own published criteria, with description quality the dominant failure mode ([pulser audit](https://dev.to/thestack_ai/i-audited-214-claude-code-skills-73-were-silently-broken-2m9a)).

The effective description template from the 650-trial study: domain identifier, imperative "ALWAYS invoke", explicit trigger topics, closing **negative constraint** that blocks the model's default workaround. Note that this last element is superpowers' anti-rationalization insight arriving independently from measurement — two unrelated sources converging on the same mechanism is the strongest desirability signal in this document.

### 2.2 Post-compaction instruction re-anchoring — **HIGH. Build it. Cheap, and the failure mode is documented upstream.**

- Post-compaction rule loss is a documented, open problem: CLAUDE.md governance rules "may be partially or fully lost after compaction" ([anthropics/claude-code#24460](https://github.com/anthropics/claude-code/issues/24460)); practitioners report agents disowning in-flight work and abandoning workflow rules after compaction in long sessions ([Porter](https://medium.com/@porter.nicholas/claude-code-post-compaction-hooks-for-context-renewal-7b616dcaa204)).
- **Correction to my earlier assessment:** I previously said nothing re-primes after compaction. That was too strong. The `SessionStart` matcher `"*"` on kbd-open plausibly covers compact-source restarts, and `position-on-prompt.sh` re-injects the position footer on every prompt — a genuine partial mitigation the plan should preserve. What is *not* re-anchored is the enforcement framing itself: AGENT_BASE_RULES precedence, the active skill index, and the KBD ownership rules that `_wr_normalize_next_command` exists to self-heal after the fact.
- **Implementation constraint discovered in research:** there is a reported bug where `SessionStart` compact-matcher stdout is not reliably injected into context ([anthropics/claude-code#15174](https://github.com/anthropics/claude-code/issues/15174)), and the PostCompact event cannot return `additionalContext` — the reliable injection channel post-compaction is the next `UserPromptSubmit` ([analysis](https://youmind.com/landing/x-viral-articles/claude-code-compact-solutions)). This is good news for this repo: it already owns a UserPromptSubmit hook chain, so the fix is an extension, not new infrastructure. Community precedent for the pattern: [post_compact_reminder](https://github.com/Dicklesworthstone/post_compact_reminder).

### 2.3 Activation + compliance eval harness (Drill analog) — **HIGH for activation evals, MEDIUM for full behavioral compliance. Build in that order.**

- Superpowers regression-guards skill *behavior* with a harness driving real tmux sessions of Claude Code/Codex/Gemini CLI, judged by an LLM verifier, and pressure-tests skill content adversarially ([superpowers CLAUDE.md](https://github.com/obra/superpowers/blob/main/CLAUDE.md), [DeepWiki](https://deepwiki.com/obra/superpowers/2-getting-started)).
- The 650-trial study demonstrates that activation evals are tractable at useful scale without a full behavioral harness: scripted prompts × conditions × trials, binary activation outcome. The forced-eval-hook experiment (84% activation via a 3-step commitment protocol) shows measurement directly produces fixes ([Seleznov](https://medium.com/@ivan.seleznov1/why-claude-code-skills-dont-activate-and-how-to-fix-it-86f679409af1)).
- Current test surface in the repo (`tests/features`, `tests/steps`, `tests/sycophancy-corpus`, cucumber.mjs, `shared/scripts/tests/`) validates scripts, schemas, and the sycophancy analyzer. **Nothing measures whether an agent in a live session invokes a skill when it should.** Given §2.1, this is the largest unmeasured risk in the system.
- Full behavioral compliance (does the agent *follow* the skill under adversarial pressure) is more expensive per data point and noisier to judge. It earns its cost only for the enforcement-critical skills (zeespec-interrogator, kbd-process-orchestrator, pmpo-outer-loop) where a compliance failure silently corrupts a whole phase. Rated MEDIUM: desirable, second in sequence, scoped to ≤6 skills.

### 2.4 Anti-rationalization content ("1% rule" / Red Flags tables) — **MEDIUM. Adopt selectively; do not port wholesale.**

Superpowers' rationalization tables are tuned for *its* workflow (brainstorm-before-code, TDD). Porting them verbatim would import the front-of-funnel conflict identified previously. What transfers cleanly: (a) the negative-constraint closing pattern in every description (validated independently in §2.1), and (b) short rationalization-preemption blocks in the ≤6 enforcement-critical skills only — written against *observed* prometheus-specific rationalizations harvested from `hook-log` records and the advisory log that `position-stop-gate.sh` already writes, not against superpowers' generic ones. Confidence that full-table porting would net-help: low (30–45%). Confidence that description-level negative constraints will help: high (75–90%), per the measured 20× spread.

### 2.5 Not worth building (researched and rejected)

- **A superpowers-style community skills repo / marketplace push.** The bus-factor and distribution problem is real (named in your own readiness report), but it is a go-to-market and community problem, not an engineering artifact. No spec here; it belongs in the gap-closure master plan's community workstream, which the plan already flags as the item most likely to slip.
- **Wholesale installation of superpowers alongside the pack.** Prior analysis stands: dueling front-of-funnel protocols, two instruction-priority hierarchies, discovery ambiguity in flat-directory harnesses.
- **Per-harness bootstrap parity work.** The control-plane plan's harness-adapter workstream owns this surface; duplicating it here would create exactly the multiple-writer problem that plan exists to kill.

---

## 3. Architecture

Three components, all instruction-plane, designed to not touch control-plane files.

```
┌──────────────────────────────────────────────────────────────────┐
│  C1: Trigger Reliability Layer (build-time + install-time)       │
│  prometheus skill lint     — description standards, 14-criteria  │
│  prometheus skill budget   — per-harness description-token model │
│  Emits: lint report, budget report, CI gate                      │
│  Lives in: tools/prometheus-cli (new `skill` subcommand crate)   │
└──────────────────────────┬───────────────────────────────────────┘
                           │ descriptions conform + fit budget
                           ▼
┌──────────────────────────────────────────────────────────────────┐
│  C2: Compaction Re-Anchor (runtime, prompt-side)                 │
│  compact-sentinel: PreCompact writes marker; UserPromptSubmit    │
│  detects marker → injects one-shot re-anchor block               │
│  (base-rules precedence + skill index digest + position)         │
│  Lives in: shared/scripts/ (extends existing hook chain)         │
└──────────────────────────┬───────────────────────────────────────┘
                           │ instructions survive long sessions
                           ▼
┌──────────────────────────────────────────────────────────────────┐
│  C3: Skill Eval Harness (offline, CI + scheduled)                │
│  Tier 1: activation evals — prompt corpus × skills × trials,     │
│          binary outcome, activation-rate regression gate         │
│  Tier 2: compliance evals — scripted sessions for ≤6 enforcement │
│          skills, LLM-verifier judged (sycophancy-corrected)      │
│  Lives in: evals/ (new top-level), driver in Rust                │
│  (`prometheus skill evals`), transcripts via harness headless    │
│  modes; liter-llm routes the verifier model                      │
└──────────────────────────────────────────────────────────────────┘
```

Design constraints honored: Rust-first (all three drivers are `prometheus-cli` subcommands or Rust crates; only the harness-launch shims are shell), no new MCP server (C3's verifier calls route through the existing liter-llm gateway), no writes to `.kbd-orchestrator` state (C2 uses its own sentinel under `~/.prometheus/`), graceful degradation everywhere (C2 injects nothing when no sentinel exists; C3 is offline-only).

The three components compound: C1 fixes the descriptions, C3 Tier 1 measures whether the fixes worked and prevents regression, C2 keeps the fixed instructions alive through long sessions, C3 Tier 2 verifies the enforcement-critical subset end-to-end. Any one alone is worth shipping; the sequence is chosen so each ships independently.

---

## 4. Functional Specification

### 4.1 C1 — `prometheus skill lint`

**Input:** one or more skill directories (default: all `skills/**/SKILL.md` plus imported submodules, read-only). Rev 1.1: accepts `--target bossfang <hands_dir>` to lint SKILL.md files consumed by librefang-skills/Hands; criteria weights may differ per consumer (librefang's parser and loading semantics are its own), so target-specific criteria profiles are a config concern, not separate code paths.
**Behavior:** validates each SKILL.md against weighted criteria: frontmatter completeness; description present on a single logical line; description length window (≥20 words, ≤ configured max); directive form (leads with domain identifier, contains an imperative invocation clause); explicit trigger topics (≥3); closing negative constraint present; no Prettier-vulnerable line wrapping; name uniqueness across the installed set; no near-duplicate descriptions (cosine similarity over embeddings via surreal-memory when reachable, token-overlap fallback when not).
**Output:** per-skill 0–100 score, machine-readable JSON + human table, `--prescribe` mode emitting a rewritten description draft per failing skill (template from §2.1, never auto-applied — human-gated like `pmpo-skill-creator --update`).
**Acceptance:** all first-party process and enforcement skills score ≥80; CI (`npm run validate:strict` delegating to the binary) fails on any first-party skill <60; imported submodules report-only.

### 4.2 C1 — `prometheus skill budget`

**Input:** installed-payload manifest per harness (reuses the parity inventory the control-plane plan mandates; until that lands, walks the flat-install target directories).
**Behavior:** models the session-start description collection per harness — sums description tokens (tokenizer via existing Rust tokenizer dep in forge-rs workspace, cl100k-class approximation acceptable), compares against a configurable per-harness budget ceiling with a warning band, and attributes consumption per skill, sorted.
**Output:** budget report per harness; exit non-zero when any harness exceeds the ceiling.
**Acceptance:** report runs in CI; the 140-payload Claude Code profile is characterized with a documented headroom number; rev 1.1: the BossFang profile (60 bundled librefang-skills + any mounted skill-pack skills per Hand) is characterized separately, since its collection semantics are independent of Claude Code's and its budget ceiling — if one exists — must be measured, not assumed; any future skill addition that crosses the warning band fails review. Open question to resolve empirically in M1: the true budget ceiling is undocumented upstream — derive it by bisection (install N skills, probe activation of a canary skill, vary N), and record the measured value with its date, since it may change under us.

### 4.3 C2 — Compaction re-anchor

**Behavior:** `PreCompact` (existing entry, additional command) writes a sentinel `~/.prometheus/compact-pending/<session_id>` alongside the existing kbd-close capture. A new early command in the existing `UserPromptSubmit` chain checks for the sentinel; when present, it consumes it (delete-before-emit, so the injection is strictly one-shot per compaction) and emits a bounded re-anchor block: (1) a 3–5 line digest of AGENT_BASE_RULES precedence (Rule 26 chain), (2) the skill-index digest — names + one-line triggers for the process/enforcement skills only, not all 140, (3) the current position footer via `waypoint_render` (reusing the existing pure renderer). Hard size cap ≤1,200 tokens; silence on any failure path.
**Rationale for the sentinel channel:** works around [#15174](https://github.com/anthropics/claude-code/issues/15174) (compact-matcher stdout unreliable) and the PostCompact `additionalContext` limitation, and — because it is harness-agnostic file state — the same mechanism serves OpenCode/Kimi adapters later without redesign.
**Acceptance:** in a session forced through `/compact`, the next prompt receives exactly one re-anchor block; a session with no compaction receives zero; kill-switch env var (`PROMETHEUS_REANCHOR=0`); measured injection adds <100ms to UserPromptSubmit.

### 4.4 C3 — Tier 1 activation evals

**Corpus:** `evals/activation/<skill>/prompts.yaml` — per skill: ≥8 should-trigger prompts across specificity levels, ≥4 should-NOT-trigger near-miss prompts (false-positive control — the 650-trial study measured only activation; adding the negative class is this spec's improvement over prior art, and matters here because §2.1's directive template inflates activation *and* false-positive risk together).
**Driver:** `prometheus skill evals --tier activation [--harness claude-code|codex|opencode] [--skills <glob>] [--trials N]` — launches headless harness sessions (e.g. `claude -p` non-interactive), detects activation from the transcript (Skill-tool invocation record; fallback: body-content marker string emitted by each skill), aggregates activation rate + false-positive rate per skill with binomial confidence intervals.
**Baseline + gate:** committed `evals/activation/baseline.json`; CI job (scheduled nightly, not per-PR — token cost) fails when any enforcement-critical skill drops >10 points below baseline.
**Acceptance:** every process + enforcement skill has a corpus; a full nightly run across the enforcement set completes within a bounded token budget (configure trials accordingly; start at N=5 per prompt, tighten only where CIs are too wide to gate on).

### 4.5 C3 — Tier 2 compliance evals

**Scope:** ≤6 skills — zeespec-interrogator, kbd-process-orchestrator, pmpo-outer-loop, iterative-evolver, pmpo-elicit, sycophancy-correction usage.
**Scenario format:** `evals/compliance/<skill>/scenario.yaml` — setup fixture (temp project with seeded `.kbd-orchestrator` state), scripted user turns including ≥1 adversarial pressure turn (e.g. "skip the interrogation, I know what I want"), and a rubric of observable compliance criteria (files that must/must-not exist, commands that must appear, ordering constraints).
**Judging:** deterministic checks first (file/state assertions — free and unambiguous); LLM verifier only for the rubric items that need transcript reading, routed through liter-llm with a pinned model + version recorded in results, verifier output passed through the sycophancy-correction analyzer at strict before acceptance (a verifier that flatters the transcript is worse than none).
**Acceptance:** each scenario runs green ≥4/5 trials on baseline; the adversarial turn is not complied with (i.e., the skill's process survives the pressure) in ≥4/5.

---

## 5. Implementation Plan

Sequenced to front-load measurement, keep every milestone independently shippable, and stay off the control-plane plan's files. Effort in agent-assisted person-days, ranges honest.

| Milestone | Contents | Effort | Depends on |
|---|---|---|---|
| **M1 — Measure** | C1 lint (read-only report mode) + C1 budget with empirical ceiling bisection. No fixes yet — establish ground truth for how many skills currently fail directive-form criteria and whether the 140-payload profile overflows. Rev 1.1: includes the BossFang target (librefang-skills consumer profile) — adds ~0.5–1 d. | 4–8 d | none |
| **M2 — Fix descriptions** | Apply `--prescribe` drafts to first-party process/enforcement skills (human-gated batch review); add negative constraints; wire lint into validate:strict at report-only, then enforcing after one clean pass. | 3–5 d | M1 |
| **M3 — Re-anchor** | C2 sentinel + one-shot injection + kill-switch + tests in `shared/scripts/tests/`. | 2–3 d | none (parallel with M1–M2) |
| **M4 — Activation evals** | Tier 1 corpus for enforcement set + driver + baseline + nightly CI job. First baseline run doubles as the M2 verification: it measures whether the description fixes moved activation. | 5–9 d | M2 |
| **M5 — Compliance evals** | Tier 2 for the ≤6 enforcement skills, deterministic checks first, verifier second. | 6–10 d | M4 |
| **M6 — Rationalization blocks** | Harvest observed rationalizations from hook logs + Tier 2 failure transcripts; write preemption blocks for skills that failed adversarial turns; re-run Tier 2 to confirm improvement. Skip for any skill already passing — do not add content without a measured failure. | 2–4 d | M5 |

Total: **22–38 days.** Against your stated capacity picture (90-day window already oversubscribed ~2×), the honest recommendation is: commit to M1–M3 (9–15 days, highest evidence-to-cost ratio, M1 may reveal a live silent-drop defect), and gate M4–M6 on M1's findings — if lint shows most descriptions already directive and budget shows headroom, M4+ importance drops a tier and can yield the schedule to the control-plane P1.

**Coordination with the control-plane work:** the only shared surface is the installed-payload inventory (C1 budget wants the parity manifest that plan produces). Until it exists, C1 walks directories; when it lands, C1 consumes the manifest. No other file overlap. C2's sentinel deliberately lives under `~/.prometheus/`, not `.kbd-orchestrator/`, so the event-runtime migration never has to reason about it. Rev 1.1: the §1.1 adapter-convergence prerequisite is control-plane-owned work, not scheduled here — but M4's `--harness` matrix should not add a BossFang-dispatched harness target until that convergence lands, or the evals will bake in a vocabulary that then changes.

---

## 6. Additional Audit Findings (outside both plans)

**F-1 · `pk-focus-on-prompt.sh` relevance heuristic is crude and pays rent on every prompt.** "Top-5 longest words" is a length-as-salience proxy; combined with the semantic path it can inject off-topic wiki context, and the sequential curl (3s cap) + `pk focus` (5s cap) chain can add up to ~8s of prompt latency in slow-degradation states (fast-fail paths exist, but a *hung-slow* surreal-memory hits the full caps). Fix: move keyword extraction into `pk` itself (BM25 over the wiki index — it already has one), run the two paths concurrently with a shared 3s deadline, and skip entirely for prompts under ~8 words (mostly commands and confirmations). Effort 1–2 d. Priority: medium — it is a per-prompt tax on every session.

**F-2 · Stop-chain worst-case latency ~2 minutes.** Eight sequential Stop commands, kbd-close capped at 60s and `propose-skill-update.sh` at 30s. Most have `|| true` fast paths, but the caps stack when services are slow rather than down. The control-plane plan will restructure steering hooks but does not address duration. Recommend: a total Stop-budget wrapper (single deadline ~45s shared across the chain, best-effort ordering: state-finalize → summary → the rest), and moving `propose-skill-update` to the existing `scheduled/` mechanism — it is not session-critical. Effort 1–2 d.

**F-3 · Imported-submodule skills are exempt from every gate.** `skills/imported/*` bypasses lint (by design in §4.1) but also currently bypasses description standards entirely, and their descriptions count against the same session-start budget as first-party skills. At minimum, C1 budget must include them in the token model (it does, per spec) and the lint report should flag imported descriptions that are budget-heavy so you can carry a local frontmatter override (description-only patch applied at install time, upstream untouched). Effort folded into M1.

**F-4 · No negative-trigger control anywhere in the current test surface.** Even before M4, the BDD suite could cheaply assert that the *installer* produces frontmatter that parses and that no two installed skills share a name or near-identical description — the naming-collision failure mode is documented as a silent wrong-skill-invocation source ([buildtolaunch audit](https://buildtolaunch.substack.com/p/claude-skills-not-working-fix)). Effort <1 d, belongs in the existing cucumber suite now.

**F-5 · Description drift risk from Prettier.** `.prettierignore` exists — verify it covers `skills/**/SKILL.md`. A reformatted multi-line description is a documented silent-kill ([DEV](https://dev.to/lizechengnet/why-claude-code-skills-dont-trigger-and-how-to-fix-them-in-2026-o7h)). One-line check; lint (§4.1) guards it permanently thereafter.

---

## 7. The Scenario That Hurts Prometheus

The failure mode this plan exists to prevent: **the control-plane work ships a world-class durable runtime while the instruction plane silently drops the skills that are supposed to drive it.** Every hour invested in event journals and causal conflict handling is downstream of a skill actually firing when the operator types a task. If the 140-payload profile is over budget today, the system's observed flakiness has a boring cause that no amount of state-machine rigor will fix — and the measurement (M1) costs 4–7 days. Conversely, the scenario in which this document over-invests: M1 comes back clean (descriptions fine, budget headroom ample), and M4–M6 then consume 13–23 days that the oversubscribed 90-day window cannot spare. That is why the plan gates, and why M1 is first.

---

## 8. Sources

Trigger reliability: [Seleznov, 650-trial activation study](https://medium.com/@ivan.seleznov1/why-claude-code-skills-dont-activate-and-how-to-fix-it-86f679409af1) · [MCP.Directory synthesis (20× description-variant spread)](https://mcp.directory/blog/why-your-claude-skill-isnt-activating-2026-fixes) · [lizecheng, token-budget root cause](https://dev.to/lizechengnet/why-claude-code-skills-dont-trigger-and-how-to-fix-them-in-2026-o7h) · [lazyskills, description-volume budget](https://lazyskills.sh/troubleshooting/skills-not-triggering) · [pulser, 214-skill audit, 73% <60](https://dev.to/thestack_ai/i-audited-214-claude-code-skills-73-were-silently-broken-2m9a) · [Agensi troubleshooting](https://www.agensi.io/learn/claude-code-skills-not-working-troubleshooting) · [buildtolaunch, naming-collision audit](https://buildtolaunch.substack.com/p/claude-skills-not-working-fix)

Compaction: [anthropics/claude-code#24460 (CLAUDE.md lost after compact)](https://github.com/anthropics/claude-code/issues/24460) · [anthropics/claude-code#15174 (compact-matcher injection bug)](https://github.com/anthropics/claude-code/issues/15174) · [anthropics/claude-code#43733 (PreCompact state persistence)](https://github.com/anthropics/claude-code/issues/43733) · [post_compact_reminder pattern](https://github.com/Dicklesworthstone/post_compact_reminder) · [PostCompact/UserPromptSubmit injection constraint](https://youmind.com/landing/x-viral-articles/claude-code-compact-solutions) · [Porter, post-compaction rule loss field report](https://medium.com/@porter.nicholas/claude-code-post-compaction-hooks-for-context-renewal-7b616dcaa204)

Superpowers architecture: [obra/superpowers](https://github.com/obra/superpowers) · [CLAUDE.md (Drill harness, pressure testing)](https://github.com/obra/superpowers/blob/main/CLAUDE.md) · [DeepWiki: bootstrap + 1% rule](https://deepwiki.com/obra/superpowers/2-getting-started) · [Claude Directory: per-harness bootstrap incl. post-compaction](https://www.claudedirectory.org/plugins/superpowers)

Context (positioning): [Requesty, loop engineering](https://www.requesty.ai/blog/loop-engineering-how-to-build-ai-agent-loops-that-run-themselves) · [explainx, graph engineering](https://explainx.ai/blog/graph-engineering-ai-agents-multi-agent-organizations-2026) · [Swarm Skills (arXiv 2605.10052)](https://arxiv.org/pdf/2605.10052)

Archon / dispatch (rev 1.1 context): [coleam00/Archon](https://github.com/coleam00/archon) · [Archon CLAUDE.md — governed automation engine positioning](https://github.com/coleam00/Archon/blob/dev/CLAUDE.md) · [DeepWiki — DAG engine, worktree isolation](https://deepwiki.com/coleam00/Archon) · [AgentConn — v1→v3 history, harness thesis](https://agentconn.com/blog/archon-open-source-harness-builder-ai-coding-deterministic-review/) · librefang facts from direct filesystem audit of `/Users/gqadonis/Projects/references/librefang` (README.md, Cargo.toml, 2026-07-28)

---

## 9. Revision Log

**1.2 — 2026-07-28.** Recorded the then-current production-convergence implementation rather than preserving the document as a stale proposal. Added §0 with the canonical state, security, and control-interface implementation, native lifecycle adapters, bounded reanchor, 145-skill structural lint and 14-target parity, 36-prompt deterministic grader, identity-safe migration, shadow/canary gates, and explicit release blockers. Corrected the prior sentinel-first compaction design: native host lifecycle events are authoritative when available. Deferred the full 145-skill scoring/rewrite and rationalization program until live activation and budget evidence exists. No discovery ceiling or production-readiness claim was inferred.

**1.3 — 2026-08-02.** Corrected the control-plane record after the journal/Loro recovery. The authoritative model is now one signed grow-only Loro event map per project, per-replica write-ahead journals, causal frontiers, explicit project/replica identity, and CRDT claims/conflict adjudication. The superseded coordinator and its dedicated database are no longer product behavior.

**1.1 — 2026-07-28.** Trigger: decision to use BossFang/librefang as the remote-dispatch surface, plus Archon comparative analysis. Changed: added §1.1 (BossFang dispatch seam — five constraints: single-writer preservation, adapter convergence prerequisite, deny-by-default remote command tiers, OFP excluded from KBD authority, accelerated open-core + stale-doc decisions); C1 lint gains a BossFang target (§4.1); C1 budget characterizes the BossFang profile separately (§4.2); M1 effort widened 4–7 d → 4–8 d; §5 coordination gains the eval-vocabulary sequencing note; sources extended. Why: BossFang's audited maturity (dispatch layer already built, SurrealDB/ACP pins structurally aligned with UAR) invalidated rev 1.0's implicit "dispatch is future work" assumption and introduced a third SKILL.md consumer that the measurement milestone must cover. Not changed: the M1-first gating logic, the C2 design, and the M4–M6 scope — the Archon inversion analysis resolved *hybrid*, which leaves the instruction plane's role intact for interactive sessions.

**1.0 — 2026-07-28.** Initial issue.
