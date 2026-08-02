# Prometheus Skill Pack — Deep-Research Audit & Validation of the Instruction-Plane Improvement Specification

**Document ID:** PAGS-AUDIT-IP-001
**Date:** 2026-07-28 · **Revision:** 1.2 production-convergence update
**Scope:** Validation of PAGS-SPEC-PSP-IP-001 (the "Fable 5" Instruction-Plane Improvement Specification), additional audit findings from a fresh read of the working tree at `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack`, and a deep-research survey of industry practice in skill reliability, agent observability, BDD video evidence, and multi-harness control planes.
**Method:** Filesystem audit of v1.6.0 (commit present at 2026-07-28 09:01), comparative analysis against obra/superpowers, LangGraph/Temporal, the Playwright/Cucumber BDD ecosystem, and the OpenTelemetry/LangSmith agent-observability stack. Confidence stated as ranges. Sources linked in §10.

> **Current-state correction (2026-08-02).** This historical instruction-plane audit is orthogonal to the recovered control plane. The control plane now owns signed journal ingestion, one Loro authority per project, project/replica identity, causal frontiers, CRDT claims and conflicts, Sovereign Sync's authoritative `kbd-control:` domain, schema unification, harness parity, and Stop-hook behavior. Sections below that audited the former coordinator have been updated to describe the replacement rather than prescribe retired behavior.

---

## Superseding Implementation Audit (rev 1.2)

The original audit below is preserved as the evidence and decision record from its filesystem snapshot. The following findings supersede its stale implementation claims:

| Original snapshot claim | Current verified state |
|---|---|
| `kbd-control:` did not exist. | `substrate/sovereign-sync` now exchanges signed authoritative Loro deltas over iroh. REST, MCP, SSE, CLI, journal ordering, idempotency, causal-frontier validation, signatures, enrollment/revocation, conflict handling, and projection metadata are implemented. |
| Progress semantics and migration fixtures were not required CI. | `scripts/test-kbd-control-plane.sh` runs state, hook, lifecycle, migration, progress-semantics, adapter, direct-writer, eval-corpus, and installed-payload fixtures. `.github/workflows/validate.yml` executes it on Linux and macOS, explicitly using `/bin/bash` for Bash 3.2 compatibility, then runs runtime, sync, and CLI checks. |
| Installed parity could not be verified and remained around 140. | The canonical inventory is 145 first-party skills. A hermetic clean install proves 145 unique payloads across 14 targets. Codex uses relocatable copied payloads; machine-specific Flint SDK paths were removed. |
| Prompt/Stop hooks could take minutes because synchronous memory, summary, learning, and proposal tasks remained. | Claude Prompt/Stop chains are reduced to the bounded canonical adapter. Noncritical work is placed in a deferred outbox. Stop acknowledgement and emergency pause do not depend on memory, network, state validity, or quorum. |
| Sentinel-based compaction was the preferred design. | Claude, Codex, OpenCode, and Kimi capability mappings now use native lifecycle events. The renderer is bounded to 4,800 characters. A sentinel is a fallback only for a host without native events. |
| There was no negative-trigger or activation contract. | A 36-prompt corpus covers six critical skills with 6 explicit, 18 implicit, and 12 near-miss cases. The deterministic grader schedules 108 trials per harness and grades invocation, typed command traces, direct-write avoidance, lifecycle, and output contracts. Live traces have not yet been captured. |
| Budget overflow was probable enough to motivate description rewriting. | The exact first-party inventory is 44,882 description/name characters. The repository now records all four harness budgets as explicitly **unmeasured**. No mass rewrite is authorized until real discovery traces demonstrate a problem. |
| Migration and rollout remained conceptual. | Migration now uses the immutable project UUID, checksummed backups, explicit lossy/legacy-read-only reporting, and identity mismatch rejection. Read-only shadow comparison and threshold-enforced local/four-harness/quorum canary stages are implemented. |

### Current production verdict

The implementation is substantially converged but is **not production-ready yet**. The remaining blockers are evidence and transport gaps, not another architecture rewrite:

1. Paired-device authorization, signed cross-process iroh/Loro exchange, collision notification, reconnect convergence, and real peer tests are implemented; live production certification remains the deployment gate.
2. Claude and OpenCode integration is active locally; generated Codex and Kimi adapters have not passed real native-host installation and mutation-guard scenarios.
3. Discovery budgets remain unmeasured for all four harnesses.
4. The 36-prompt corpus has a deterministic grader but no three-run live baseline per harness.
5. The required seven shadow days, 100 real mutations, 10,000 synthetic mutations, and staged canaries have not elapsed.
6. Transport/key/snapshot recovery requires an external security review.

The full 145-skill compliance rewrite, marketplace work, and broad BossFang dispatch expansion remain a separately gated follow-up. They should not displace the six blockers above.

---

## 0. Executive Summary

| Claim in the Fable 5 spec | Verified? | Evidence |
|---|---|---|
| P0 operator-safety patch has substantially landed in `position-stop-gate.sh` | **Yes** | Reads as written: `stop_hook_active` checked, `PAUSE` file unconditional, terminal/suspended states recognized, dedup keyed to `session + state-revision fingerprint` (transcript size explicitly excluded), advisory-only, exits 0 on every path (`shared/scripts/position-stop-gate.sh:1-115`). |
| `waypoint-render.sh` implements suspended/terminal vocabularies and dual-casing reads | **Yes** (corroborated) | Both `position-stop-gate.sh` calls `_wr_is_terminal_status` and `_wr_is_suspended_status` from that library. |
| mtime freshness check on `position.json` remains | **Partially** | True for legacy readers; the canonical KBD runtime is now event-sourced (`substrate/kbd-runtime/src/lib.rs:436-446` replay reducer) and treats mtime as informational. |
| `kbd-runtime` exists as a Rust crate with append-only per-replica journals, lifecycle, causal frontiers, CRDT claims, conflicts, and integrity | **Yes** | `substrate/kbd-runtime` implements deterministic replay, signed schema-v2 events, one authoritative `project.loro`, atomic command execution under one file lock, idempotency, project/replica registry, adoption, conflict resolution, audit export, and migration/recovery tests. |
| `kbd-control:<project-id>` Sovereign Sync domain exists | **Yes** | `substrate/sovereign-sync` publishes and imports signed authoritative Loro deltas, validates grow-only event maps before persistence, supports same-machine loopback convergence, and emits typed conflict/claim events. |
| P0 schema normalization (`changes` as array) is canonical | **Yes** | `skills/process/kbd-process-orchestrator/references/schemas/progress.schema.json` defines `changes` as an ordered array with `primaryCounter: "implementation"` and a four-dimension `completion` object (primary/implementation/evidence/certification/publication). Schema v2 is declared canonical. |
| `test-progress-semantics.sh` is fixed and in CI | **Partially — script exists, not in CI** | `skills/process/kbd-process-orchestrator/shared/lib/tests/test-progress-semantics.sh` exists, asserts pre-mutation counter correctness, accepts legacy ledgers, rejects contradictory ones. **Not invoked** by the `test-kbd-control-plane.sh` wrapper nor by `npm test` (see §3.4). |
| 140/140 portable installed payloads for every harness | **Cannot fully verify — harness-specific install trees exist** | `.claude/`, `.opencode/`, `.codex-plugin/`, `.agents/`, `.cursor/`, `.windsurf/` all populated; counts vary. The Fable 5 spec flagged Kimi at 139 and Codex as containing machine-specific absolute paths; this audit confirms the existence of the structure but cannot enumerate the install tree without running `install:platforms`. |
| 236 SKILL.md files in the working tree | **Yes** | Direct count. 190 first-party + imported submodule SKILL.md files (per Fable 5 claim). |
| F-1 `pk-focus-on-prompt.sh` "top-5 longest words" heuristic is crude | **Yes — confirmed in code** | `shared/scripts/pk-focus-on-prompt.sh:33-40` — `awk 'length>4' \| sort -u \| awk '{print length, $0}' \| sort -rn \| head -5` (length as proxy for salience). Sequential curl (3s cap) + `pk focus` (5s cap) caps in `pk-focus-on-prompt.sh:51, 90`. |
| F-2 Stop-chain worst-case latency ~2 minutes | **Confirmed directionally** | Eight sequential Stop commands per the installable hook chain; per-script caps of 60s (`kbd-close`) and 30s (`propose-skill-update.sh` header comment). |
| F-3 Imported submodules are exempt from every gate | **Confirmed** | `scripts/skill-matrix.js:23-26` explicitly ignores `skills/imported/**`; `validate-skills.js --exclude-submodules` is the npm default. The 236-file count includes 46 SKILL.md files under `tools/cowork-skills/` and `tools/disk-space-guardian/.kimi/skills/` that share frontmatter but are not subject to description-budget enforcement. |
| F-4 No negative-trigger control in the current test surface | **Confirmed** | `tests/features/` contains exactly two features (`forge-validate.feature`, `forge-enrich.feature`) and one draft (`okf-wiki-ingest.feature`). Step defs in `tests/steps/forge-steps.ts` are forge-binary-only; no behavior assertion that an installed skill is invoked or not invoked. `cucumber.mjs` runs only the two forge features. |
| F-5 Description drift risk from Prettier | **Confirmed — partial coverage** | `.prettierignore` exists but its current contents (visible at root) do not list `skills/**/SKILL.md` explicitly; markdown formatting via Prettier can re-wrap multi-line `description:` values, which is a documented silent-kill pattern (see §1.2). |

**Net read on the Fable 5 spec, corrected 2026-08-02:** the instruction-plane diagnosis and priority ordering remain useful. The control-plane recommendation has been superseded by per-replica journals, a project Loro authority, explicit identity, and visible causal conflict handling. The spec's instruction-plane M1–M3 sequence remains the right first cut.

**Three additions the spec does not contain** that this audit adds in §6–§7:
1. A **BDD-driven skill contract test** that runs every enforcement-critical skill in a controlled harness session, asserts it *fired* (positive) and asserts the *forbidden* skill did not fire on a near-miss prompt (negative). The Fable 5 spec sketches this in §4.4–4.5 but does not commit to running it as part of the same `npm test` cycle the BDD skill tests already use.
2. A **`prometheus skill trace` debugging command** that takes a failed session_id, replays the JSONL transcript, and surfaces "this skill should have fired here and didn't" — built on the `kbd-runtime` event store plus a new transcript→event projector. The spec's M2 "human-gated batch review" of `--prescribe` drafts is necessary but insufficient without a way to investigate why a specific skill didn't fire in production.
3. A **Sovereign-Sync `kbd-control:` domain** that the Fable 5 plan lists as P2 but that the in-flight Rust runtime is already structured to consume. The control-plane plan defers it; this audit argues it should be a co-P1 with the projection writer because **without replication the operator safety story is single-device**.

---

## 1. Validation of the Fable 5 Findings

### 1.1 Skill trigger reliability — **CRITICAL, build it. Confirmed with caveats.**

The 650-trial activation study ([MCP.Directory summary](https://mcp.directory/blog/why-your-skill-isnt-activating-2026-fixes)) and the forced-eval-hook experiment ([Seleznov, 84% activation with no false positives](https://medium.com/@ivan.seleznov1/why-claude-code-skills-dont-activate-and-how-to-fix-it-86f679409af1)) are the two strongest public data points on this question. They are corroborated by the [Claude Code hidden skill budget setting in v2.1.129](https://claudefa.st/blog/guide/mechanics/skill-listing-budget) — `skillListingBudgetFraction` (default 1% of context) and `skillListingMaxDescChars` (default 1536). When `skillListingBudgetFraction * context_window` overflows, descriptions are silently dropped, lowest-priority first. With [~140 SKILL.md files installed per harness](https://www.reddit.com/r/ClaudeAI/comments/1psgr91/claude_code_drops_skills_after_a_15k_description/), the prometheus-skill-pack is squarely in the exposed profile.

**Caveat the Fable 5 spec misses:** the budget fraction is **per-model-context-window**, not a flat 15K. On a 1M-token Opus window the budget is ~10K tokens, not 15K characters. On Sonnet 4.5 the same setting produces ~5K tokens. So the spec's "character budget" framing understates the installable footprint on long-context models and overstates it on short-context ones. The C1 budget tool should model both settings, not just one.

**Independent confirmation of the negative-constraint mechanism:** the [obrásuperpowers writing-skills methodology](https://www.youtube.com/watch?v=SiabL_tBbzY) — "skill design is TDD applied to documents" — requires a baseline failure pass before a skill is written, and describes the same closing negative constraint pattern. Two unrelated research streams (measurement + writing methodology) converging on the same mechanism is a high-confidence signal.

**The 84% vs 100% question:** the [Scott Spence forced-eval hook writeup](https://www.reddit.com/r/ClaudeCode/comments/1qzjy2h/claude_code_skills_went_from_84_to_100_activation/) shows that on **24 challenging prompts including "no skill" near-misses**, forced-eval hit 75% overall with no false positives, while LLM-eval hit 67% with 4 false positives. The honest read is: **forced-eval hooks are the most reliable mechanism for a 140-skill install where false positives are operationally expensive** (each one consumes context the user did not ask to spend). This is the mechanism the C1 `--prescribe` mode should encode as a reference pattern in the description template, not a deployable default — the spec already says "never auto-applied" which is right.

### 1.2 Compaction re-anchoring — **HIGH, build it. Confirmed; one spec correction needed.**

The Fable 5 spec's correction that `position-on-prompt.sh` is already a partial mitigation is correct. Audit of `shared/scripts/position-on-prompt.sh` (called from the `UserPromptSubmit` chain) confirms it re-injects the position footer on every prompt. The un-anchored surface — `AGENT_BASE_RULES.md` precedence (Rule 26 chain), active skill index, KBD ownership rules — is real.

**The spec's sentinel-channel design is the right call.** [anthropics/claude-code#15174 (compact-matcher stdout unreliable)](https://github.com/anthropics/claude-code/issues/15174) and the PostCompact `additionalContext` limitation make the UserPromptSubmit path the only reliable injection channel. The `~/.prometheus/compact-pending/<session_id>` sentinel under `~/.prometheus/` is exactly the right scope — it lives outside `.kbd-orchestrator/`, so the event-runtime migration never has to reason about it.

**What the spec doesn't address: which skills to re-anchor.** The plan says "process/enforcement skills only, not all 140" but doesn't enumerate them. From the current skill tree the enforcement-critical subset is small and well-defined: `kbd-process-orchestrator`, `pmpo-outer-loop`, `pmpo-elicit`, `zeespec-interrogator`, `iterative-evolver`, `adversarial-review`, `pmpo-evolver`, `kbd-evolve`, `kbd-goal`, `kbd-goal-check`. That is ten names, not 140. The re-anchor block should list exactly those ten with one-line triggers and silently skip the rest. This is a 1-day refinement of M3.

### 1.3 Activation + compliance eval harness — **HIGH for activation, MEDIUM for compliance. Confirmed with two additions.**

The [superpowers Drill harness](https://github.com/obra/superpowers/blob/main/CLAUDE.md) runs in real tmux sessions with an LLM verifier. The Fable 5 spec correctly rates activation evals as the higher-leverage Tier 1 (cheap, scales, regression-gates cleanly) and behavioral compliance as the higher-cost Tier 2. This audit agrees with that ranking and adds:

- **Activation evals should be hermetic.** The C3 driver should use `claude -p` (headless) against a fixture prompt corpus, capture the JSONL transcript, and assert on the `Skill` tool call log. The [Scott Spence experiment](https://www.reddit.com/r/ClaudeCode/comments/1qzjy2h/claude_code_skills_went_from_84_to_100_activation/) shows the harness-level activation detection is the variable; the eval harness must isolate the variable, which means pinning the harness version and the model version. Recorded in `evals/activation/baseline.json` along with `{harness_version, model, prompt_corpus_sha, run_timestamp}`. Without these pins the baseline is meaningless.
- **Compliance evals should run after a behavioral baseline exists.** Tier 2 is expensive per data point. The "10/10 skills pass Tier 2" is the wrong target; the right target is "the *enforcement-critical* skills pass Tier 2, and any skill that fails a scenario is either fixed or moved out of the enforcement-critical set." Fable 5's "≤6 skills" is closer to the right count, but the spec should publish the inclusion criterion, not the count.

### 1.4 Anti-rationalization content — **MEDIUM. Confirmed; do not port wholesale.**

The [superpowers "Red Flags" tables](https://deepwiki.com/obra/superpowers/2-getting-started) are tuned for *its* workflow (brainstorm-before-code, TDD). The Fable 5 spec's instinct to harvest rationalizations from observed hook logs rather than port wholesale is correct — the [4-step misbehaving-agent playbook](https://dev.to/yureki_lab/how-i-debug-a-misbehaving-ai-coding-agent-my-4-step-playbook-3m12) calls out exactly the same failure pattern ("the model claims right after each tool call… find the first turn where the model's summary of a tool result doesn't match the raw result") and arrives at the same fix ("fix at the right layer: instructions, verification, or tool contract"). The misbehavior is universal; the rationalizations are model-and-workflow-specific.

The cleanest port: **a `prometheus rationalizations` skill** (or a `references/rationalizations.md` per enforcement skill) that lists the three-to-five failure modes observed in the prometheus-skill-pack's own `hook-log` records. Harvest script is straightforward — `shared/scripts/sycophancy-check-reflection.sh` already runs the sycophancy analyzer; a sibling `harvest-rationalizations.sh` that ingests advisory logs and surfaces the most common "I think we're done" rationalizations, normalized to a vocabulary, would close the loop.

### 1.5 Items the spec rated below my own threshold

- **Skill ecosystem / community skills marketplace.** The spec correctly puts this in "not worth building" for engineering reasons. The community problem is real but separate. I would add: if a community marketplace is later pursued, **the `prometheus skill lint` should also be runnable on user-contributed skills** with a `--external` mode that skips the first-party 60-score floor and reports the score. This makes the marketplace a measurement problem rather than a moderation problem.
- **Per-harness bootstrap parity work.** The spec defers this to the control plane. Correct — but the C1 budget tool should *consume* the parity manifest the control plane produces, not re-walk the directory. The spec already says "until it exists, C1 walks directories" which is right.

---

## 2. Industry Research — Skills and the Instruction-Plane Problem

### 2.1 The skill activation problem is a budget problem and a trigger problem, in that order

The [Claude Code skills 2.0 announcement](https://www.youtube.com/watch?v=qXWz-V_XMOc&vl=en) added A/B test tooling and a Skill Creator revamp — but the most consequential shipped change in the same window was the [v2.1.129 `skillListingBudgetFraction` setting](https://claudefa.st/blog/guide/mechanics/skill-listing-budget). The default of 1% of the context window is the floor; the practical ceiling is reached when low-priority skills start dropping off the list. The [production-grade skills playbook](https://github.com/enuno/claude-command-and-control/blob/main/docs/best-practices/14-Production-Grade-Skills-Development.md) reports a **detection ceiling of 32–36 skills** before the system struggles with consistent selection — which is exactly the range that the prometheus-skill-pack exceeds by a factor of four.

The implication for the Fable 5 spec's M1 budget bisection: the right test is not "does the budget overflow" but **"given a fixed budget, which skills survive and which get dropped, and what is the activation rate of the dropped ones in a real session?"** A skill that survives the budget but never activates wastes the slot. A skill that activates 95% of the time is worth 4× the slot of one that activates 50%. The C1 budget tool should report a **per-skill weighted value** = (chars / budget) × (1 − activation_rate), not just characters.

### 2.2 Skills as probabilistic invocation, not deterministic dispatch

The [vitalets/playwright-bdd](https://vitalets.github.io/playwright-bdd/) approach is the right mental model: **a skill is not a function call, it is a search hit**. The LLM matches the user prompt against the description set, picks the closest description, and loads the body. This has three operational consequences:

1. **Description quality is the entire product.** A perfect skill body with a vague description never fires. A weak skill body with a sharp description fires but disappoints. The description has to do two jobs: trigger reliably on relevant prompts, and *not* trigger on near-miss prompts. The 84% / 20× spread in the 650-trial study is the most important number in the spec — it quantifies how wide that range is.
2. **Forbidden-skill matching is as important as preferred-skill matching.** The spec's "≥4 should-NOT-trigger near-miss prompts" per skill is the right negative-class control. Without it, "always invoke when X" descriptions fire on prompts where the skill is irrelevant. The [Scott Spence experiment's 24 challenging prompts](https://www.reddit.com/r/ClaudeCode/comments/1qzjy2h/claude_code_skills_went_from_84_to_100_activation/) including non-Svelte questions measured exactly this; the LLM-eval hook produced 4 false positives on the negative class. That is the failure mode the spec is guarding against.
3. **Skill composition is not transitive.** If `kbd-process-orchestrator` fires and then internally tries to invoke `zeespec-interrogator`, the inner skill still has to match against the LLM's description set. Nested skill invocation in autonomous loops is a real source of silent failures. The Fable 5 spec doesn't address this; the `evaluating-session.sh` hook chain logs advisory entries that could be a data source for it (see §6.2).

### 2.3 The 1% rule, applied surgically

The [superpowers `using-superpowers` skill](https://github.com/obra/superpowers/blob/main/skills/using-superpowers/SKILL.md) requires invocation of any relevant skill even on a 1% chance. Translated to the prometheus-skill-pack: this is too aggressive at 140 skills. The [production-grade playbook](https://github.com/enuno/claude-command-and-control/blob/main/docs/best-practices/14-Production-Grade-Skills-Development.md) reports that the **simple instruction hook** (a softer version of the 1% rule) drops to **0% on multi-skill prompts** — confirming that the all-skills-all-the-time model collapses at scale.

The right rule for prometheus-skill-pack is closer to: **"for the ≤10 enforcement-critical skills, invoke on 1% chance; for the rest, only on direct user invocation or matching description keywords."** This is what the C1 negative-constraint mechanism encodes in description form, but it should also be a `AGENT_BASE_RULES.md` rule — currently the enforcement precedence is implicit, not explicit. One paragraph in `AGENT_BASE_RULES.md` is a 1-day change that codifies what the description template already implies.

### 2.4 The hidden coupling: skills interact with the harness's permission model

The [Claude Code skills security writeup](https://labs.reversec.com/posts/2026/05/skill-issues-compromising-claude-code-with-malicious-skills-agents-part-1) notes that **"by default, Claude Code cannot take any non-idempotent actions without user approval"** — but skills can pre-approve tools via the `allowed-tools` frontmatter. The 140-payload Claude Code install pre-approves tools per skill, which is a real attack surface. The `bdd-lifecycle-loop` skill (which runs shell scripts) and `prometheus-cli` (which executes daemons) have wide `allowed-tools` scopes. A lint rule for `allowed-tools` minimum-necessary scope is a missing safety check. Effort 1 day; security impact: non-trivial.

---

## 3. Audit of the Current Codebase — Cross-Cutting Findings

### 3.1 The instruction plane has a working eval surface, but it does not measure what matters

The current test surface is:
- `tests/features/forge-validate.feature` and `forge-enrich.feature` — two features exercising the `forge` Rust binary's validate and enrich subcommands. Steps in `tests/steps/forge-steps.ts`. Run by `npm run cucumber`.
- `tests/features/drafts/okf-wiki-ingest.feature` — one draft, no step definitions (correctly excluded by `cucumber.mjs`).
- `tests/sycophancy-corpus/` — sycophancy analyzer fixtures, used by the `prometheus skill sycophancy` command.
- `shared/scripts/tests/` — 17 shell test scripts (`test-child-scope.sh`, `test-position-stop-gate.sh`, `test-progress-semantics.sh`, `test-sycophancy-gate-e2e.sh`, etc.) validating hook and schema behavior.
- `npm test` → `scripts/test-skills.js` — runs the skill matrix, the validate script, and the cucumber suite.

**What is not measured:** whether a skill that should fire actually fires in a real harness session. The `skill-matrix.js` script (SP-016, Jaccard collision detection) is the closest thing and it measures description-similarity, not activation. The 214-skill audit ([pulser, 73% < 60/100](https://dev.to/thestack_ai/i-audited-214-claude-code-skills-73-were-silently-broken-2m9a)) suggests the activation problem is not theoretical.

### 3.2 The BDD test surface is small but well-engineered

The BDD testing skill family is shipped and works. `bdd-cucumber-js`, `bdd-cucumber-rs`, `bdd-lifecycle-loop`, `bdd-video-proof` are the four shipped skills. The `bdd-video-proof` v2.0 local cert bundle (Mode A) is the right default — the IPFS pinning Mode B is documented but the [BDD-003 IPFS pin sweep](docs/future-work/02-bdd-testing-evolution/STATUS.md) is partially-shipped at best. The BDD future work matrix has 7 of 15 BDD-* items shipped or partially-shipped.

**What is genuinely good in the BDD surface:**
- The immutable-tests rule (`shared/scripts/protect-tests.sh`, `shared/scripts/tests/test-protect-tests.sh`) is enforced and is genuinely load-bearing. New BDD work goes to `tests/features/drafts/`. This is the kind of self-disciplined CI design that [Anthropic's research on agentic coding expertise](https://explainx.ai/blog/anthropic-claude-code-expertise-research-agentic-coding-2026) implicitly endorses: "what separates high-performing users is not their ability to write code themselves but their ability to specify problems clearly" — the immutable-tests rule is the structural enforcement of "specify problems clearly, don't move goalposts."
- The `bdd-lifecycle-loop` flake budget (`scripts/flake-budget.sh`) — shipped in BDD-002 — is the kind of triage automation that the [AI agent debugging playbook](https://jobsbyculture.com/blog/ai-agent-debugging-guide-2026) calls for: "a written failure-mode taxonomy, updated every time a new bug shape appears."

**What is missing from the BDD surface:**
- **BDD for skills themselves.** There is no feature file that asserts "skill X should fire on prompt Y" or "skill X should NOT fire on prompt Z." This is the test surface the Fable 5 spec's M4 introduces. It belongs *alongside* the existing `tests/features/` directory, not inside it — but it should run in the same `npm run cucumber` cycle.
- **End-to-end video evidence for the BDD-* tests that already ship.** `bdd-video-proof` v2.0 produces a cert bundle, but the existing `forge-validate.feature` and `forge-enrich.feature` do not record video because they don't drive a browser. This is correct — the BDD video proof is for UI scenarios. The missing piece is **a CLI-mode video proof** for non-browser scenarios (think: a scenario that drives `prometheus kbd pause` and expects the response to be deterministic), which can use [Playwright's MCP screencast API](https://playwright.dev/mcp/tools/video) or a tmux/screen recorder.
- **BDD-005 testid drift detection is "ready, planned."** This is the highest-leverage remaining BDD-* item per the matrix. The `data-testid` convention is already required by `bdd-cucumber-js`; the drift detection script is the closing-of-the-loop. 1–2 day effort, immediate value.
- **BDD-007 candidate test drafts promotion workflow.** The guard already allows `tests/features/drafts/`; the promotion script (with human sign-off) is missing. Pair with M4 of the spec: a draft skill eval prompt can graduate to a real `evals/activation/<skill>/prompts.yaml` only after human review.

### 3.3 The 236-file SKILL.md install is a documented silent-drop risk

`find . -name "SKILL.md"` returns 236 files. This is the literal install footprint the Fable 5 spec is concerned about. Without C1 budget enforcement, every harness that loads this repo at session-start is at risk of the [15K character / 1% context silent drop](https://www.reddit.com/r/ClaudeAI/comments/1psgr91/claude_code_drops_skills_after_a_15k_description/). The `prometheus-cli` already has `install`, `list`, `search`, `verify`, and `policy` subcommands in `tools/prometheus-cli/crates/prometheus-cli/src/commands/` — adding `skill lint` and `skill budget` is a subcommand addition, not new infrastructure.

**One thing the spec does not address:** **skill priority**. The `skillListingBudgetFraction` documentation describes "lowest-priority full descriptions get dropped until the listing fits." The prometheus-skill-pack does not currently express skill priority in frontmatter (no `priority: high|medium|low` field). The C1 budget tool should suggest a priority assignment based on (a) which skills are in the enforcement-critical set, (b) which skills have the highest measured activation rate, and (c) which skills the user invokes manually most often. A frontmatter field the lint can enforce is a 1-day change; a sensible default assignment is a 1-day follow-up.

### 3.4 The control-plane fixes are in flight; the CI test surface is not consolidated

`scripts/test-kbd-control-plane.sh` exists. It is not invoked by `npm test`. The `scripts/smoke-test.sh` is a separate entry point. `npm test` → `scripts/test-skills.js` runs the validate + matrix + cucumber path. The control plane has its own wrapper. **A single `npm run test:all` that runs the skill surface, the control-plane shell tests, the cucumber suite, and the new activation eval smoke run is the right consolidation**, and it is missing. The Fable 5 spec calls for this under "Create one required control-plane test job"; this audit agrees and would extend it to include the skill lint and budget check.

### 3.5 The skill description collision detection (SP-016) is real and not currently enforced as a CI gate

`scripts/skill-matrix.js` is shipped and runs pairwise Jaccard similarity. CI mode (`--ci`) fails on new collisions not in `scripts/skill-collision-allowlist.json`. The allowlist file is checked in. This is the existing scaffolding on which C1 lint can build — the Fable 5 spec correctly identifies that the C1 description-quality work extends SP-016 from a Jaccard baseline to a multi-criteria scoring framework. The fact that SP-016 exists and is not widely referenced is itself a finding: **the C1 work should be framed as "extending the existing SP-016 infrastructure," not "new lint tool,"** to avoid creating a parallel measurement system that drifts from the production one.

### 3.6 `propose-skill-update.sh` is on the Stop chain and the Fable 5 spec's F-2 budget recommendation is correct

`shared/scripts/propose-skill-update.sh` is the propose-skill-update step on the Stop chain. Its own header comment notes it is called with `|| true` and is non-blocking. It is invoked from `evaluate-session.sh`. The spec's F-2 recommendation to move this to the `scheduled/` mechanism is correct: **the Stop chain is operator-safety critical; non-session-critical work does not belong on it.** Effort 1 day; behavior unchanged for users; the Stop chain's worst-case latency drops materially.

### 3.7 `pk-focus-on-prompt.sh` has the heuristic, and the heuristic has a known-bad shape

The Fable 5 spec's F-1 diagnosis is precise: "top-5 longest words is a length-as-salience proxy." A `pk` BM25 search over the wiki index is the right replacement, and the index exists. The spec's recommendation (move keyword extraction into `pk`, run lexical and semantic paths concurrently with a shared 3s deadline, skip for prompts < 8 words) is correct and bounded — 1–2 day effort. The fast-fail path is already there; the slow-degrade path is what hurts.

The `surreal-memory` curl in the same file (3s cap) and the `pk focus` invocation (5s cap) **stack to 8s of per-prompt latency in slow-degradation states**. For a hook that runs on every prompt in a hook-heavy install, 8s × N turns is real. Concurrent execution with a shared deadline is the right fix.

### 3.8 The `pre-commit` and CI matrix is mostly correct, with one safety gap

`.prettierignore` exists. It does not list `skills/**/SKILL.md` explicitly. A multi-line `description:` field can be reformatted by Prettier (or a future hook), which is a [documented silent-kill pattern](https://dev.to/lizechengnet/why-claude-code-skills-dont-trigger-and-how-to-fix-them-in-2026-o7h). This is the F-5 finding; the fix is one line. The lint (C1) then enforces it permanently.

### 3.9 The `kbd-runtime` crate is the most important file in the repo

`substrate/kbd-runtime` implements the event-sourced runtime, lifecycle, causal-frontier, CRDT-claim, conflict, and integrity contract as production Rust. Current observations:

- **Replay determinism is not guaranteed by serde_json default.** The spec's footnote 1 ("Specify RFC 8785 / JCS for canonical serialization or byte-equivalence will flake") is correct and load-bearing. The current `serde_json::to_vec` output is non-canonical (HashMap iteration order, number formatting). The acceptance test "byte-equivalent projections on repeated replay" will fail today against the current code, even with deterministic event inputs. Fix: introduce a `canonical_json::to_vec` wrapper or switch to `serde_json::to_vec(&serde_json::Value::Object(map))` with a `BTreeMap<String, Value>` serializer. Effort 0.5–1 day; a single test (`fn replay_is_byte_equivalent_across_replays`) gates it.
- **The `LifecycleState::is_suspended` and `is_terminal` predicates are correct** and are already used by the bash hooks via `_wr_is_suspended_status` / `_wr_is_terminal_status` in `waypoint-render.sh`. Good.
- **Claim expiry is represented as signed CRDT state with TTL and monotonic tokens.** Tests cover acquire, renew, release, collision winner selection, loser blocking, reconnect conflicts, and operator adjudication.
- **Singleton mutations validate a causal frontier that dominates the current slot.** Concurrent candidates remain visible and emit an alarm rather than being silently overwritten.
- **Operator pause/cancel remains an explicit signed authority path.** Ordinary harness mutations still require valid replica identity, signature, frontier, and applicable claim state.
- **The five named tests cover the headline properties; the test file goes to line 975 of lib.rs.** Worth pulling them into a separate `tests/` integration test directory as the crate grows, but not blocking.

### 3.10 The `sovereign-sync` KBD domain is now authoritative

The historical gap is closed. `substrate/sovereign-sync` implements `kbd-control:<project-id>` as signed authoritative Loro-delta exchange over iroh. Per-replica journals remain write-ahead ingestion logs; imports are validated before atomic persistence, same-machine replicas converge through the shared document path, and typed subscribers observe appends, claims, conflicts, and singleton violations.

---

## 4. The Multi-Harness Development Story

### 4.1 The control plane's replicated-authority model

The recovered control plane uses a more general model: each replica has a signed write-ahead journal, the grow-only Loro map is project authority, and causal frontiers plus explicit claims/conflicts govern concurrent work. This retains the useful [Temporal "history is the state" framing](https://chrisgavin.dev/blog/temporal-data-management) while supporting offline replicas and deterministic convergence.

The prometheus-skill-pack's choice to put the KBD runtime in a Rust library (`kbd-runtime`) rather than coupling it to any single harness's event loop is the right separation of concerns. The bash hooks (`position-stop-gate.sh`, etc.) and the harness-specific adapters (Claude Code, Kimi, OpenCode, Codex) all consume the same `Runtime` API.

### 4.2 The instruction plane has the opposite problem: it must run on every harness, but no harness sees the same state

A skill in `skills/process/kbd-process-orchestrator/SKILL.md` is a **portable instruction** — the same Markdown is loaded by Claude Code, Kimi, OpenCode, Codex. The harness reads the description, decides whether to load the body, and (if loaded) follows the instructions. The skill does not have a "state" in the KBD sense; it has a description and a body.

This means the **instruction plane is intrinsically simpler than the control plane**: there is no concurrency, no multi-writer, no event journal. There is, however, the description-budget problem (§1.1) and the activation problem (§1.2) and the compaction-loss problem (§1.3). The Fable 5 spec is correct to scope C1, C2, C3 to the instruction plane and not let them touch the control plane.

### 4.3 The seam between planes: prompts and hooks

The two planes meet in two places:
- **The bash hooks** (`shared/scripts/*.sh`) read KBD state (via `current-waypoint.json`, soon via the `kbd-runtime` `replay()` API) and emit advisory or steering behavior. They are control-plane consumers.
- **The skill bodies** (`SKILL.md` Markdown) sometimes instruct the agent to read KBD state, pause a run, or acquire a scoped CRDT claim. The kbd-process-orchestrator skill body is the canonical example. The skill is instruction-plane; what it *tells* the agent to do is control-plane.

The seam is the `prometheus kbd ...` CLI surface (`tools/prometheus-cli/crates/prometheus-cli/src/commands/`). Its operator contract covers status, lifecycle controls, `claim acquire|renew|release`, conflict resolution, registry/adoption, audit, and observation. The skill is declarative ("here's what to do when X happens"), and the CLI is imperative ("do this thing now"). The control plane makes those commands signed, atomic, causally validated, and recoverable.

### 4.4 The multi-harness install story is more mature than the Fable 5 spec acknowledges

The codebase has:
- `.claude/` — Claude Code payloads
- `.opencode/` — OpenCode payloads
- `.codex-plugin/` — Codex plugin
- `.agents/` — agentskills.io format
- `.cursor/` — Cursor payloads
- `.windsurf/` — Windsurf payloads
- `.clinerules/` — Cline payloads
- `tools/disk-space-guardian/.kimi/skills/` — Kimi payloads

Eight harnesses, eight install trees. The `scripts/install-platforms.ts` and `scripts/install-skills-flat.sh` (the "flat" install is the agentskills.io standard format) handle the per-harness adapter logic. The [Fable 5 spec's claim of 140/140 portable installed payloads for every harness](https://www.reddit.com/r/ClaudeAI/comments/1qzspyq/superpowers_exploded_to_210k_github_stars_in_7/) is in line with the superpowers precedent of "the same skill folder works identically across Claude Code, Codex CLI, Codex App, Cursor, Gemini CLI, OpenCode, GitHub Copilot CLI, and Factory Droid" (8 harnesses). The fact that prometheus-skill-pack matches the harness count is good; the spec's focus on Kimi 139 vs Claude 140 and Codex machine-specific absolute paths is the right granularity to verify.

### 4.5 A multi-harness install has a multi-harness bug surface

A skill that fires on Claude Code may not fire on Codex. A `position-stop-gate.sh` patch on Claude Code is invisible to Kimi. The control-plane plan correctly identifies per-harness parity as a P0 issue. **The instruction plane needs the same parity discipline:** the activation eval (§1.3) must run per harness. A skill that activates 100% on Claude Code and 30% on Codex is a real bug, not a measurement artifact. The C3 driver must be parameterized by harness, with the corpus and baseline versioned per harness. The spec says this; this audit confirms the cost is real.

---

## 5. Additional Audit Findings (Beyond the Fable 5 Spec)

### 5.1 No per-skill activation telemetry exists in production

`shared/scripts/hook-log.sh` is invoked by most hooks and writes to an in-process log. The Stop gate writes to `~/.prometheus/position-stop-advisories.txt` and `~/.prometheus/position-stop-advisories.log`. **There is no per-skill "this skill was considered and either fired or didn't" telemetry.** Without it, the activation problem is unobservable in production. A `UserPromptSubmit` hook that logs the `available_skills` list, the per-skill description text, and a post-response "did any skill fire this turn" log is a 1-day hook addition that closes the observability gap. The C3 eval harness consumes the same schema; a single telemetry producer serves both purposes.

### 5.2 The `sycophancy` analyzer is well-built but is not wired into the skill eval loop

`tools/prometheus-cli/crates/prometheus-cli/src/commands/sycophancy.rs` and `shared/scripts/sycophancy-check-*.sh` exist. The analyzer scores agent output against a corpus of sycophancy patterns. The Fable 5 spec recommends running the verifier output through the sycophancy analyzer at strict before acceptance — this audit agrees and adds: **the same analyzer should run on the C3 Tier 2 scenario transcripts as a continuous check, not just on the verifier's verdict.** A scenario where the agent appears to follow the skill but uses sycophantic language to convince the verifier it followed the skill is a worse failure than a scenario where the agent refuses. The analyzer is the only thing that detects it.

### 5.3 The `forge` binary is a strong foundation for a `prometheus skill` subcommand

`tools/forge-rs/` exists alongside `tools/prometheus-cli/`. The forge binary has `validate`, `enrich` subcommands, and a cucumber test surface (the only two features currently in `tests/features/`). The `prometheus-cli` `skill` subcommand is not present in `main.rs` — but the scaffold is there: `validate.rs` already shells out to `scripts/validate-skills.js`. The C1 lint, C1 budget, and C3 evals can land as `prometheus-cli` subcommands (Rust, with shim scripts for harness-headless invocation) without creating a new binary. The `forge` binary's role is to test source code against a constitution; the `prometheus skill` subcommand's role is to test skill metadata against a quality bar. Different responsibilities, same shell-out-to-Node pattern.

### 5.4 The Codex `OpenSpec` change-format is an unused lever

`openspec/` exists at the repo root with `change-001` and `archive/`. `skills/process/openspec-*` skills are vendored under `tools/disk-space-guardian/.kimi/skills/`. The OpenSpec format — "proposed change with tasks, validated per task" — is exactly the kind of structured workflow the Fable 5 spec's `kbd-control:` event journal could integrate with. A `prometheus openspec` subcommand that bridges OpenSpec tasks to KBD `PlanRevised` events is a 2-day addition and would let the existing OpenSpec authoring flow participate in the durable execution model. Lower priority than the eval harness, but worth noting.

### 5.5 The cross-tool handoff protocol document is the right reference; the file is not yet a state machine

Cross-tool coordination documentation is tutorial material; the `kbd-runtime` public API and event schema are canonical. Handoffs are represented by signed lifecycle/claim events and causal frontiers, not by a second document-driven state machine.

### 5.6 Doctor command is the right place to surface instruction-plane health

`tools/prometheus-cli/crates/prometheus-cli/src/commands/doctor.rs` implements a multi-check diagnostic with severity levels, repair actions, and dry-run mode. Control-plane diagnostics cover daemon reachability, journal/document integrity, registry and replica state, projection health, claim/conflict state, synchronization lag, and installed harness parity. This audit adds: the same doctor should report **(a) description-budget utilization per harness with the dropped-skill list if any, (b) skill-collision count and the worst offenders, (c) the last activation-eval baseline date and which skills have regressed, (d) the Prettier/description-format check status.** These are 4 new check methods, all reading existing telemetry — total effort 1–2 days.

### 5.7 The `prometheus-knowledge` MCP server is the right surface for activation telemetry

`tools/prometheus-knowledge/` (visible in the directory listing) is the home of `pk` (the knowledge engine that powers `pk-focus-on-prompt.sh`). It already exposes MCP tools. Adding an `mcp__pk__skill_activations` tool that returns the per-skill activation telemetry (§5.1) is a 1-day MCP addition that lets any harness observe the activation pattern without a separate logging path. This is a cleaner architectural choice than per-harness log scraping.

### 5.8 The `BTreeMap` requirement for canonical JSON (§3.9) is also a Sovereign Sync requirement

If the `kbd-control:` event journal is replicated via Loro, Loro's CRDT merge needs deterministic inputs to converge. Non-canonical JSON serialization produces different CRDT updates for the same logical event. The RFC 8785 / JCS canonicalization is therefore not just a replay-determinism issue — it is a multi-device-replication issue. The Fable 5 spec's footnote 1 is more load-bearing than the spec itself acknowledges; promote it from footnote to top-level requirement.

### 5.9 The `nats`-style pub/sub mental model for hooks is a missed opportunity

The hook chain is: UserPromptSubmit → bash script → emit advisory / context. Each hook is fire-and-forget. Typed AG-UI/SSE events now expose appended events, claim acquisition, claim conflicts, and singleton violations. Additional local hook pub/sub remains optional rather than a control-plane prerequisite.

---

## 6. Recommendations for Additional Changes

This section lists changes that go beyond the Fable 5 spec. Each has a confidence range, a bounded effort estimate, and a priority relative to the spec's M1–M6.

### 6.1 A `prometheus skill trace` debugging command — **HIGH priority, 3–5 day effort**

**Problem.** When a skill doesn't fire in production, the only debugging surface is the JSONL transcript — the user (or agent) must walk the transcript turn-by-turn looking for the divergence moment. The [4-step misbehaving-agent playbook](https://dev.to/yureki_lab/how-i-debug-a-misbehaving-ai-coding-agent-my-4-step-playbook-3m12) is correct that this is the right approach, but it is currently a manual exercise.

**Solution.** A `prometheus skill trace <session_id>` subcommand that:
1. Loads the JSONL transcript for `session_id`.
2. Extracts: every `User` message, every `assistant` message, every `Skill` tool call (or its absence), and the description text of all skills available at session-start.
3. For each `User` message, computes: which skills' descriptions are top-3 by lexical overlap, and whether any of them fired. Surfaces a "considered but not fired" list.
4. Cross-references with the per-skill activation baseline (if a C3 baseline exists) and flags regressions.

**Why it matters.** Without this, M2's human-gated batch review of `--prescribe` drafts is necessary but not sufficient. The review catches skills that don't fire on a curated prompt set; it doesn't catch skills that don't fire on a real user's prompt. The trace command is the bridge between the eval harness (synthetic prompts) and production (real prompts).

**Effort:** 3–5 days. The transcript parsing is straightforward; the description-overlap scoring reuses the existing Jaccard infrastructure in `scripts/skill-matrix.js`; the regression flagging reuses the activation baseline JSON. No new infrastructure.

### 6.2 A BDD-driven skill contract test in the same `npm run cucumber` cycle — **HIGH priority, 2–3 day effort**

**Problem.** The Fable 5 spec's M4 introduces activation evals as a separate subcommand (`prometheus skill evals`). This audit argues they should be in `tests/features/` and run with `npm run cucumber` — the same cycle as the existing forge features. The reasons:

1. The cucumber surface is **already wired to the harness's coverage reporting and report generation**. Adding the activation evals as cucumber features gives them the same CI ergonomics for free.
2. The BDD evidence format (`scenario.embed(mime='text/html', data=...)`) can carry the per-scenario activation rate, false-positive rate, and the `Skill` tool-call log inline. The same `bdd-video-proof` v2.0 cert bundle covers them.
3. The 1% / 99% boundary between "this is a unit test" and "this is an eval" is artificial. A scenario that runs `prometheus skill eval --skill kbd-process-orchestrator --prompt "I need to plan a phase"` and asserts "the Skill tool was called with name kbd-process-orchestrator within 30s" is a BDD scenario. It costs 5–10s per scenario; the 140-skill activation eval is 140 × 5s × N trials = bounded.
4. The draft BDD-007 promotion workflow (§3.2) can graduate a draft skill eval to a real one with the same mechanism as a draft feature graduates to an implemented feature.

**Effort:** 2–3 days. The cucumber step definitions are thin (one for "run eval for skill X" and one for "assert skill X fired in transcript"); the fixtures live in `tests/features/skill-eval/`; the existing `cucumber.mjs` paths glob extends naturally.

### 6.3 Promote the `kbd-control:` Sovereign Sync domain to co-P1 — **MEDIUM-HIGH priority, 3–5 day effort**

**Resolved problem.** The KBD runtime now publishes signed project events through the authoritative `kbd-control:` Loro document. A pause, claim, adjudication, or lifecycle change can converge across machines without treating a compatibility projection or audit export as input.

**Solution.** A `kbd-control` adapter in `substrate/sovereign-sync/src/` that:
1. Exposes a `subscribe(project_id) -> EventStream<RuntimeEvent>` method.
2. Reconciles per-replica write-ahead journals into the grow-only project document and publishes signed deltas.
3. Bootstraps a device from the project document and causal frontier, then replays only journal entries missing from that authority.
4. Provides authoritative signed Loro-delta exchange for each project while retaining per-replica journals as write-ahead ingestion logs.

**Effort:** 3–5 days. The adapter is thin; the existing `kbd-runtime` and `sovereign-sync` crates do the heavy lifting.

### 6.4 A description-budget model that simulates the per-harness settings — **HIGH priority, 1–2 day effort**

**Problem.** The C1 budget tool per the Fable 5 spec is a character count against a fixed budget. This audit recommends it instead model the two settings in [Claude Code v2.1.129+](https://claudefa.st/blog/guide/mechanics/skill-listing-budget): `skillListingBudgetFraction` (fraction of context window) and `skillListingMaxDescChars` (per-skill truncation threshold). The model should report per-harness:
- Total description characters and tokens.
- Per-skill truncation list (descriptions over `skillListingMaxDescChars`).
- Drop list (the N skills the harness would drop to fit the budget, lowest-priority first).
- Effective per-skill budget: chars / harness_budget × priority_weight.

**Why it matters.** A 140-skill install on Claude Code 2.1.129 with `skillListingBudgetFraction=0.01` on a 1M-token Opus window has a 10K-token description budget. The same install on Sonnet 4.5 has a 5K-token budget. The C1 tool should run the simulation per harness and surface the worst case as the report's headline. Effort: 1–2 days. The model is small; the gain is that the report can be cross-checked against the actual harness behavior with `claude --debug` and the `/doctor` command.

### 6.5 Wire the sycophancy analyzer into the C3 Tier 2 verifier loop — **MEDIUM priority, 0.5–1 day effort**

**Problem.** The Fable 5 spec says the C3 Tier 2 verifier output "is passed through the sycophancy-corrected analyzer at strict before acceptance." This audit confirms the spec and adds: the analyzer should also run on the agent's transcript for the scenario, not just on the verifier's output. A scenario where the agent writes a sycophantic compliance message that the verifier is then biased by is a real failure mode. The analyzer already exists; the wiring is the change.

**Effort:** 0.5–1 day. The hook chain already invokes the analyzer; the C3 driver is a new caller.

### 6.6 Move `propose-skill-update.sh` off the Stop chain — **MEDIUM priority, 1 day effort**

**Problem.** The Fable 5 spec's F-2 finding. The script is non-session-critical and is called via `|| true` from the Stop chain. It belongs in the `scheduled/` mechanism alongside the other session-end advisory work.

**Effort:** 1 day. The script moves; the call site in `evaluate-session.sh` is updated; the Stop chain's worst-case latency drops by up to 30s.

### 6.7 Add a priority frontmatter field and enforce it in C1 lint — **MEDIUM priority, 1 day effort**

**Problem.** The `skillListingBudgetFraction` documentation describes skills being dropped "lowest-priority first." The prometheus-skill-pack does not currently express skill priority in frontmatter.

**Solution.** A `priority: high | medium | low` (default `medium`) frontmatter field. The C1 lint enforces that the field is present and valid. The C1 budget tool uses the field to compute the drop list. A `prometheus skill priority --set <skill> <level>` CLI command can re-prioritize after M4's baseline measurement. The enforcement-critical skill list (§1.2) gets `priority: high`; everything else defaults to `medium` unless the activation measurement justifies a downgrade.

**Effort:** 1 day. The field is optional in the agentskills.io spec; the lint can require it for first-party skills.

### 6.8 Add a CI gate that fails on `allowed-tools` over-scoping — **MEDIUM-HIGH priority (security), 1 day effort**

**Problem.** A skill with `allowed-tools: 'Bash(*)'` can execute arbitrary shell commands; a skill with `allowed-tools: 'Read, Grep, Glob'` cannot. The 140-skill install likely contains a mix. The [Claude Code skills security writeup](https://labs.reversec.com/posts/2026/05/skill-issues-compromising-claude-code-with-malicious-skills-agents-part-1) is the right reference.

**Solution.** A C1 lint rule that classifies `allowed-tools` patterns and warns on:
- Wildcard `Bash(*)` or `Bash(.*)` without a documented justification in the skill body.
- A list of tools inconsistent with the skill's stated purpose (e.g., a "documentation lookup" skill that requests `Edit` and `Write`).
- Pre-approval of `mcp__*` tools (a skill should not unilaterally approve an MCP server).

**Effort:** 1 day. The pattern classification is a small set of regexps; the lint output is a per-skill table with the warnings, not a hard fail (warnings only; the user accepts the over-scope by adding to the allowlist).

### 6.9 Surface activation telemetry in the doctor command — **MEDIUM priority, 1 day effort**

**Problem.** §5.1's finding: no per-skill activation telemetry exists. Once §5.1's hook is added, the doctor command should report on it.

**Solution.** Extend `tools/prometheus-cli/crates/prometheus-cli/src/commands/doctor.rs` with a `check_skill_activation_telemetry` method that reads the per-skill activation log, computes the 7-day rolling activation rate per skill, and flags any enforcement-critical skill with rate < 0.5 (well below the [Seleznov 84% baseline](https://medium.com/@ivan.seleznov1/why-claude-code-skills-dont-activate-and-how-to-fix-it-86f679409af1)) as a yellow severity, < 0.2 as red.

**Effort:** 1 day. The hook is §5.1; this is just the reader.

### 6.10 Codify the enforcement-critical skill set in `AGENT_BASE_RULES.md` — **LOW priority, 0.5 day effort**

**Problem.** The C2 re-anchor and the activation eval both depend on a stable list of enforcement-critical skills. The list is currently implicit.

**Solution.** One paragraph in `AGENT_BASE_RULES.md` (a precedence list, like the existing Rule 26 chain) that names the 10 enforcement-critical skills and instructs the LLM to invoke them on 1% chance while requiring explicit description match for everything else.

**Effort:** 0.5 day. The list is already enumerated in §1.2; the paragraph is short.

---

## 7. The Multi-Harness Development Story — Impact Analysis

The Fable 5 spec asks the right question: what is the impact of this project on multi-harness development? The answer, in three parts:

### 7.1 The control plane's impact is durable, federated, cross-tool execution

The `kbd-runtime` + `kbd-control:` replication model is what makes multi-harness development not just portable but *durable*. A task can be started in Claude Code on a MacBook Pro, paused, audited, and resumed in OpenCode on a Mac mini, with the audit log preserved across the device boundary. The [Temporal durable execution model](https://temporal.io/blog/temporal-replaces-state-machines-for-distributed-applications) and the [LangGraph interrupt + thread_id pattern](https://docs.langchain.com/oss/python/langgraph/interrupts) are the proven precedents; the prometheus-skill-pack's Rust implementation matches their semantics. The impact: a multi-harness workflow is no longer "two parallel sessions that may or may not be in sync" — it is "one durable execution with multiple observers."

The catch: the replication is only as good as the canonical serialization (§3.9, §5.8). The audit's call to promote the JCS requirement from footnote to top-level is not pedantic — it is the difference between a working multi-device system and a silently-merging CRDT that occasionally loses events.

### 7.2 The instruction plane's impact is the difference between "the skill exists" and "the skill runs"

The single highest-leverage change in the Fable 5 spec is C1 lint + C1 budget. The 140-skill install is silently dropping skills in the wild; the 1% description-budget is the proximate cause. Without measurement and budget enforcement, every other change — re-anchor, eval harness, rationalization content — operates on a moving target: you are testing and fixing skills that may or may not survive the harness's session-start description collection.

The C3 activation eval is the second-highest. It is the only way to detect a skill that survives the budget but never fires on relevant prompts. The 84% activation rate is the floor; the 100% forced-eval rate is the ceiling. Without an eval harness, the prometheus-skill-pack is operating on the description-writer's optimism, not the harness's measured behavior.

The impact: the instruction plane turns "the skill exists" into "the skill runs 95% of the time when relevant and never when not relevant." That is the production-grade skill quality bar that the [production-grade skills playbook](https://github.com/enuno/claude-command-and-control/blob/main/docs/best-practices/14-Production-Grade-Skills-Development.md) describes.

### 7.3 The combination is the actual deliverable

Neither plane alone is enough. The control plane without a working instruction plane is a perfectly durable execution engine that runs skills half the time. The instruction plane without a working control plane is a set of skills that fire 95% of the time but cannot pause, claim, handoff, or audit across devices.

The Fable 5 spec's two-plane framing is correct. This audit's additions (§5.1, §5.7, §6.1, §6.2, §6.4) tighten the instruction plane; the control-plane plan already tightens the control plane. The combined project is the thing the prometheus-skill-pack needs to be a multi-harness development platform rather than a multi-harness skill collection.

---

## 8. Sequencing and Capacity

The Fable 5 spec's implementation plan (M1–M6, 22–38 days) is honest. This audit's additions (the §6 list) total 13–22 days. Combined, the full plan is 35–60 days. The spec's recommendation — "commit to M1–M3 (9–15 days) and gate M4–M6 on M1's findings" — is the right call.

**Recommended first 15 days** (combining the spec's M1–M3 with this audit's highest-leverage additions):

| Day | Workstream | Source | Effort |
|---|---|---|---|
| 1–7 | C1 lint (read-only report mode) + C1 budget with per-harness settings simulation (§6.4) | Fable 5 M1 + this audit §6.4 | 7d |
| 1–3 | Description-format Prettier guard (F-5) + C1 negative-constraint template | Fable 5 M2 prep + this audit §3.8 | 1d |
| 3–5 | C2 re-anchor with sentinel + enforcement-critical skill set codified in `AGENT_BASE_RULES.md` (§6.10) | Fable 5 M3 + this audit §1.2, §6.10 | 3d |
| 5–7 | Move `propose-skill-update.sh` off Stop chain (§6.6) | This audit §3.6, §6.6 | 1d |
| 7–9 | Priority frontmatter field + C1 lint enforcement (§6.7) | This audit §6.7 | 1d |
| 9–10 | Allowed-tools lint rule (§6.8) | This audit §6.8 | 1d |
| 10–12 | `kbd-control:` adapter (sovereign-sync) MVP (§6.3) | This audit §6.3 | 3d |
| 12–14 | Canonical JSON for `kbd-runtime` (RFC 8785 / JCS) + replay-byte-equivalence test | This audit §3.9, §5.8 | 1d |
| 14–15 | Integration: doctor check for description budget, harness parity, replication lag | This audit §5.6, §6.9 | 1d |

This 15-day first cut delivers:
- Ground truth on the description budget per harness (M1).
- A working re-anchor mechanism (M3).
- The control-plane's replication leg (kbd-control adapter MVP) so the cross-machine test plan is testable.
- Three low-effort safety/quality wins (Prettier, allowed-tools, propose-skill-update move).
- The canonical JSON fix that makes the durability claim testable.
- Doctor command extensions that surface the new health signals.

If the budget is tight, drop §6.3 from the first 15 days and accept that the cross-machine test plan stays as a manual acceptance run for one more cycle.

---

## 9. Risks and Open Questions

1. **Description budget is a moving target.** The `skillListingBudgetFraction` setting in Claude Code is a fraction of the *model's* context window, not a fixed character count. The 1% default on a 1M-token Opus window is 10K tokens, not 15K characters. The C1 budget tool needs to model this dynamically; the empirical ceiling bisection the spec proposes is the right approach, but the measurement should be repeated after any model upgrade.
2. **Activation evals cost tokens.** A 140-skill × 8-prompt × 5-trial activation eval is 5,600 Skill-tool invocations, each consuming prompt tokens and producing model output. At Anthropic Sonnet 4.5 pricing, this is roughly $5–15 per full run. Nightly CI is the right cadence; per-PR CI is not. The spec is correct on this.
3. **The re-anchor's kill-switch (PROMETHEUS_REANCHOR=0) needs a documented escape hatch.** The spec mentions it; this audit adds: the kill-switch should also produce a one-line log entry to a file (not stderr) so the agent can later diagnose "why did the re-anchor not happen." Effort 0.25 day.
4. **The cross-harness parity check is only as good as the harness adapters.** If the Codex fallback prompts still contain machine-specific absolute paths (the Fable 5 P0 finding), the lint and budget tool are running on a corrupted install. The control-plane plan's P0 work on Codex relocatability is a precondition for any instruction-plane measurement on Codex.
5. **The enforcement-critical skill set is a 10-skill list maintained by a person.** A skill added to the repo by a contributor is *not* in the list unless the contributor remembers to update `AGENT_BASE_RULES.md`. A `prometheus skill designate --enforcement-critical <skill>` CLI command is a 0.5-day addition that updates both the markdown and the C2 re-anchor list atomically.
6. **Resolved:** the Sovereign Sync `kbd-control:` adapter now carries signed authoritative Loro deltas. Imports are validated in isolation before atomic persistence; grow-only event-map validation prevents a peer update from deleting or rewriting accepted events.
7. **The BDD-driven skill contract test (§6.2) is a meaningful additional test surface.** Two features today → 140 features tomorrow (one per skill). This is fine, but the existing `cucumber.mjs` glob will pick them all up, and the CI runtime will grow proportionally. The spec's per-PR vs nightly split applies here too: a smoke test of 10 enforcement-critical skills per PR, the full 140 nightly.

---

## 10. Sources

**Skill activation and the instruction plane:**
- [Seleznov, 84% activation with forced-eval hook](https://medium.com/@ivan.seleznov1/why-claude-code-skills-dont-activate-and-how-to-fix-it-86f679409af1) · [Scott Spence, 250-trial 84%→100% experiment](https://www.reddit.com/r/ClaudeCode/comments/1qzjy2h/claude_code_skills_went_from_84_to_100_activation/) · [MCP.Directory, 650-trial 20× description-variant spread](https://mcp.directory/blog/why-your-skill-isnt-activating-2026-fixes) · [claudefa.st, skillListingBudgetFraction setting in Claude Code 2.1.129](https://claudefa.st/blog/guide/mechanics/skill-listing-budget) · [lazyskills, description-volume budget](https://lazyskills.sh/troubleshooting/skills-not-triggering) · [lizecheng, token-budget root cause](https://dev.to/lizechengnet/why-claude-code-skills-dont-trigger-and-how-to-fix-them-in-2026-o7h) · [pulser, 214-skill audit, 73% <60](https://dev.to/thestack_ai/i-audited-214-claude-code-skills-73-were-silently-broken-2m9a) · [production-grade skills playbook](https://github.com/enuno/claude-command-and-control/blob/main/docs/best-practices/14-Production-Grade-Skills-Development.md) · [Claude Code skills security](https://labs.reversec.com/posts/2026/05/skill-issues-compromising-claude-code-with-malicious-skills-agents-part-1) · [vanja.io, complete guide](https://vanja.io/claude-code-skills-guide/) · [hidekazu-konishi, complete guide](https://hidekazu-konishi.com/entry/claude_code_skills_complete_guide.html) · [Reddit thread on 15K character budget](https://www.reddit.com/r/ClaudeAI/comments/1psgr91/claude_code_drops_skills_after_a_15k_description/)

**Superpowers and the methodology landscape:**
- [obra/superpowers](https://github.com/obra/superpowers) · [CLAUDE.md, Drill harness](https://github.com/obra/superpowers/blob/main/CLAUDE.md) · [using-superpowers skill](https://github.com/obra/superpowers/blob/main/skills/using-superpowers/SKILL.md) · [DeepWiki, 1% rule + writing-skills methodology](https://deepwiki.com/obra/superpowers/2-getting-started) · [knightli, superpowers skills framework](https://knightli.com/en/2026/05/15/obra-superpowers-agentic-skills-framework/) · [termdock, skills framework](https://www.termdock.com/en/blog/superpowers-framework-agent-skills) · [one-man-company, 210K stars](https://news.one-man-company.com/news/hot-repo-superpowers) · [Chinese YouTube breakdown, Iron Law + TDD analog for skills](https://www.youtube.com/watch?v=SiabL_tBbzY) · [lzw.me, skill system guide](https://lzw.me/docs/opencodedocs/obra/superpowers/start/using-skills/) · [Context7](https://context7.com/obra/superpowers)

**Durable execution and the control plane:**
- [Temporal, replaces state machines](https://temporal.io/blog/temporal-replaces-state-machines-for-distributed-applications) · [chrisgavin, history is the state](https://chrisgavin.dev/blog/temporal-data-management) · [niteagent, durable AI agents with Temporal](https://niteagent.com/blog/2026-06-29-durable-ai-agents-temporal-guide/) · [Temporal main site](https://temporal.io/) · [devstarsj, durable execution 2026](https://devstarsj.github.io/2026/06/04/durable-execution-temporal-restate-distributed-systems/) · [devstarsj, Temporal for microservices 2026](https://devstarsj.github.io/2026/03/24/temporal-durable-execution-workflows-microservices-guide-2026/)
- [LangGraph interrupts](https://docs.langchain.com/oss/python/langgraph/interrupts) · [LangGraph types/interrupt reference](https://reference.langchain.com/python/langgraph/types/interrupt) · [zenn, LangGraph durable execution 入門](https://zenn.dev/moridev/articles/888bf07cdcb57a) · [zenml, Kitaru + LangGraph](https://www.zenml.io/blog/langgraph-durable-runtime) · [zylos, durable execution for agent runtimes](https://zylos.ai/research/2026-04-24-durable-execution-agent-runtimes/)

**Agent observability and debugging:**
- [athenic, agent observability production](https://getathenic.com/blog/agent-observability-monitoring-production) · [langchain, agent observability](https://www.langchain.com/resources/agent-observability) · [laminar, agent observability tracing and debugging](https://laminar.sh/article/agent-observability) · [neelmishra, agent observability](https://neelmishra.github.io/blog/mlops/llm-agents/agent-observability.html) · [coverge, AI agent observability tracing](https://coverge.ai/blog/ai-agent-observability) · [AgentTrace, structured logging framework for agents (arXiv)](https://arxiv.org/html/2602.10133v1) · [LADYBUG, LLM debugger for data-driven apps](https://openproceedings.org/2025/conf/edbt/paper-313.pdf)
- [tianpan, deterministic replay for non-deterministic agents](https://tianpan.co/blog/2026-04-12-deterministic-replay-debugging-non-deterministic-ai-agents) · [dev.to yureki_lab, 4-step misbehaving agent playbook](https://dev.to/yureki_lab/how-i-debug-a-misbehaving-ai-coding-agent-my-4-step-playbook-3m12) · [jobsbyculture, AI agent debugging playbook 2026](https://jobsbyculture.com/blog/ai-agent-debugging-guide-2026) · [producthunt, Retrace](https://www.producthunt.com/products/retrace-2) · [langchain, debugging and evaluating agents with observability](https://www.langchain.com/blog/agent-observability-powers-agent-evaluation)
- [anthropic, AI assistance impacts coding skills](https://www.anthropic.com/research/AI-assistance-coding-skills) · [anthropic, Claude Code expertise](https://www.anthropic.com/research/claude-code-expertise) · [tigzig, anthropic claude code expertise survey](https://www.tigzig.com/ai/posts/anthropic-claude-code-expertise-survey-jun2026.md) · [explainx, anthropic claude code expertise research 2026](https://explainx.ai/blog/anthropic-claude-code-expertise-research-agentic-coding-2026) · [dev.to behruamm, Claude Code session analyzer](https://dev.to/behruamm/are-you-actually-using-claude-code-well-i-built-a-free-scorer-based-on-anthropics-own-research-4bbj)

**BDD + Playwright video evidence:**
- [vitalets/playwright-bdd](https://github.com/vitalets/playwright-bdd) · [vitalets, playwright-bdd docs](https://vitalets.github.io/playwright-bdd/) · [playwright.dev, videos](https://playwright.dev/docs/videos) · [qaskills, Playwright screencast API + video recording](https://qaskills.sh/blog/playwright-screencast-api-video-recording-guide) · [qaskills, Playwright Cucumber BDD integration 2026](https://qaskills.sh/blog/playwright-cucumber-bdd-integration-guide) · [testdino, Playwright BDD setup](https://testdino.com/blog/playwright-bdd) · [thetestingacademy, Playwright Cucumber BDD](https://app.thetestingacademy.com/blog/playwright-cucumber-bdd) · [browserstack, Playwright Cucumber 2026](https://www.browserstack.com/guide/playwright-cucumber) · [Tallyb/cucumber-playwright starter](https://github.com/Tallyb/cucumber-playwright) · [rjking1/playwright-bdd-rk](https://github.com/rjking1/playwright-bdd-rk) · [justin.abrah.ms, generate demo videos with Playwright](https://justin.abrah.ms/blog/2026-02-12-generating-demo-videos-with-playwright.html) · [playwright.dev/mcp, video recording](https://playwright.dev/mcp/tools/video) · [playwright.dev/agent-cli, video recording](https://playwright.dev/agent-cli/commands/video-recording) · [vitest, Playwright browser provider](https://vitest.dev/config/browser/playwright) · [cucumber.io, reporting](https://cucumber.io/docs/cucumber/reporting/) · [cucumber/cucumber-js #124, embedding media in reports](https://github.com/cucumber/cucumber-js/issues/124) · [WasiqB/multiple-cucumber-html-reporter #99](https://github.com/WasiqB/multiple-cucumber-html-reporter/issues/99) · [jenkinsci/cucumber-reports-plugin #113](https://github.com/jenkinsci/cucumber-reports-plugin/issues/113) · [Stack Overflow, mime type for sauce labs video in Cucumber report](https://stackoverflow.com/questions/55165306/mime-type-for-adding-sauce-labs-video-link-in-cucumber-report)

**Cost / budget / rate-limit context for the instruction plane:**
- [code.claude.com/docs, costs (zh-TW)](https://code.claude.com/docs/zh-TW/costs) · [truefoundry, Claude Code rate limits 2026](https://www.truefoundry.com/blog/claude-code-limits-explained) · [sitepoint, Claude Code rate limits 2026](https://www.sitepoint.com/claude-code-rate-limits-explained/) · [github/spec-kit #1672, excessive token usage](https://github.com/github/spec-kit/discussions/1672) · [Claude Platform, extended thinking](https://platform.claude.com/docs/en/build-with-claude/extended-thinking) · [Claude Docs, skills (id)](https://code.claude.com/docs/id/skills) · [LinkedIn, perevillega, skill budget](https://www.linkedin.com/posts/perevillega_i-discovered-something-that-i-suspect-many-activity-7417838063093071872-OgUJ) · [Pere Villega post, char budget](https://www.linkedin.com/posts/perevillega_i-discovered-something-that-i-suspect-many-activity-7417838063093071872-OgUJ)

**CI/CD and quality gates:**
- [vibehackers, ci-cd-and-automation skill](https://vibehackers.io/claude-code/skills/ci-cd-and-automation-sickn33) · [BerryKuipers/claude-code-toolkit, quality-gate](https://claude-plugins.dev/skills/@BerryKuipers/claude-code-toolkit/quality-gate) · [arxiv 2604.14228, dive into Claude Code skills](https://arxiv.org/html/2604.14228v1) · [blog.swifttools.eu, Claude Code 2026](https://blog.swifttools.eu/posts/claude-code-features-guide-2026) · [FlorianBruniaux/claude-code-ultimate-guide](https://github.com/FlorianBruniaux/claude-code-ultimate-guide) · [YouTube Anthropic Skills 2.0](https://www.youtube.com/watch?v=qXWz-V_XMOc&vl=en) · [YouTube Playwright video recording](https://www.youtube.com/watch?v=pvOE-Su7xuE) · [YouTube Playwright-BDD project](https://www.youtube.com/watch?v=xVIk_X3H7rM)

---

## 11. Final Verdict

**Rev 1.3 correction (2026-08-02):** the next step is no longer the 15-day implementation cut described below; the control-plane replacement has landed. Its final release decision is governed by live installed-service certification, while the instruction-plane readiness claims remain governed by measured discovery budgets, live cross-harness eval traces, elapsed rollout evidence, and external security review.

The Fable 5 Instruction-Plane Improvement Specification is a sound document grounded in real measurement. Its priority ordering, its architectural direction, its cost ranges, and its sequencing decisions are all defensible against the audit evidence in this document. The P0 operator-safety patch is landed and verifiable. The KBD runtime is shipped as a Rust crate with a real test suite. The schema normalization to `changes` as an ordered array is in place. The multi-harness install is real and the per-harness parity work is the right next step.

The audit's additions are bounded and complementary:

- **Six new commands / subcommands** (`prometheus skill trace`, `prometheus skill priority`, `prometheus skill designate --enforcement-critical`, `prometheus skill lint` (Rust, building on the existing `forge`-style shell-out pattern), `prometheus skill budget` (with per-harness settings simulation), and the `kbd-control:` adapter on `sovereign-sync`).
- **Two new CI gates** (the activation-eval smoke test on enforcement-critical skills, the BDD-driven skill contract test in the same `cucumber` cycle as forge-validate).
- **Three new skill frontmatter conventions** (priority, allowed-tools minimum scope, and the codified enforcement-critical list in `AGENT_BASE_RULES.md`).
- **Two new doctor checks** (per-skill activation telemetry health, description-budget health per harness).
- **One canonical serialization fix** (RFC 8785 / JCS for the `kbd-runtime` event store, with the corresponding `replay_is_byte_equivalent_across_replays` test).
- **One critical spec promotion** (JCS canonicalization, currently a footnote, must be a top-level requirement — it gates both replay determinism and multi-device replication).

The combined 15-day first cut delivers ground truth, the re-anchor mechanism, the replication leg, the canonical-JSON fix, the safety/quality wins, and the doctor command extensions. The 22–38 day Fable 5 plan then layers on top. The total budget to a measurable, durable, multi-harness instruction + control plane is 35–60 days.

The impact on multi-harness development is the difference between a portable skill collection and a durable multi-harness execution platform. The two planes are orthogonal; the merged system is the product.
