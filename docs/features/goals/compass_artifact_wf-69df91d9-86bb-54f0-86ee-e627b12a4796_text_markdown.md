# Harness-Agnostic Goal Abstraction for the Prometheus Skill Pack (PSP)
### Architecture, Functional Specification, Implementation Plan & Strategic Framing
Prepared for Travis James, Founder/CTO, Prometheus AGS — July 31, 2026

---

## 1. Executive Summary

**Direct answer:** Build the goal abstraction as a **hybrid** — one portable *Goal Contract* (data) plus per-harness *drivers* (code), with a thin in-harness *compliance skill* that detects and delegates to native goal primitives where they exist. Do **not** build "one skill that adapts per-harness." Prompting alone cannot equalize harness behavior, so the enforcement mechanism must live outside the model context, in a UAR-native goal supervisor that treats every harness as a dispatch target through the `knowme:harness` adapters and BossFang.

**The recommended frame shift:** Travis's stated objective — "a single common goal implementation" delivered as a skill — conflates two separable things: the *contract* (what "done" means, verifiable exit criteria, evidence, scope, failure paths) and the *enforcement* (what keeps the agent running until the contract is satisfied and survives context overflow). The contract is genuinely portable and should be a versioned artifact in PSP. The enforcement is **not** portable via prompting: Codex is RL-post-trained for persistence, Claude Code is trained toward user alignment/consent, and no SKILL.md can rewrite either disposition. The right architecture separates the two: the portable contract rides in every harness; enforcement is delivered natively where a harness has it (Codex `/goal`, Claude Code `/goal`+Stop hook) and by an **outer Rust supervisor** (Ralph-style fresh-session-per-phase) where it does not — and, critically, the outer supervisor is the *only* mechanism that is context-overflow-proof for all harnesses including the ones with native primitives.

**Three findings that should drive the design:**
1. Both Claude Code and Codex now ship a native `/goal` primitive. OpenAI's Codex introduced `/goal` roughly two weeks before Anthropic shipped the same primitive in Claude Code (Codex Goal Mode shipped in Codex CLI v0.128.0 on April 30, 2026, graduated to GA and became on-by-default in v0.133.0 on May 21, 2026; Claude Code `/goal` requires v2.1.139 or later, released May 11, 2026). Neither solves context overflow; both suffer post-compaction goal dilution. This validates the outer-loop approach as the durable substrate, not the native primitives.
2. The Anthropic Agent Skills spec (SKILL.md), published by Anthropic on December 18, 2025, is now a genuine cross-harness standard adopted by 32 tools as of March 2026 per agentskills.io — named adopters include OpenAI Codex, Google Gemini CLI, JetBrains Junie, AWS Kiro, Block Goose, OpenCode, Cursor, and VS Code ("Within 48 hours of publication, both OpenAI and Microsoft had integrated support"). MiniMax's `mmx` also ships a SKILL.md that symlinks into agent skill directories. This means the *contract* and the *compliance layer* can genuinely be written once as portable SKILL.md files. Enforcement cannot.
3. Travis's empirical observation (Codex finishes autonomously and produces provable codebases; Claude Code stops early) is **real but its mechanism is often mis-attributed**. The best available controlled evidence (BUILD-AND-FIND, arXiv 2605.06136) does *not* show Codex "finishing" while Claude "stops early"; it shows Claude Opus achieving 100% implementation coverage but producing artifacts that cost materially more downstream inspection effort, while Codex/GPT-5.5 produced terser, more "findable" artifacts using far fewer build tokens. The stopping-early behavior Travis sees is a real product-philosophy divergence (nilenso), but PSP must not over-claim that an outer loop makes Claude "as good as Codex" — it makes Claude *finish*, which is a necessary but not sufficient condition for provable output.

---

## 2. Harness Capability Matrix

| Capability | Claude Code | Codex CLI | OpenCode | Kimi Code (Moonshot) | MMX / MiniMax (see note) | KnowMe/UAR |
|---|---|---|---|---|---|---|
| **Native goal primitive** | `/goal` (v2.1.139+, May 11 2026): sets completion condition; a small fast model (Haiku default) evaluates after every turn; wraps a session-scoped prompt-based Stop hook | Goal Mode `/goal` (v0.128.0, Apr 30 2026; GA/default v0.133.0, May 21 2026): runtime-managed plan→act→test→review loop with per-goal token budget, pause/resume/clear | None native; build via outer loop over `opencode serve` HTTP/SDK | None native; "Ralph loops" documented as external pattern | None native (coding); `minimax` TUI has Duo "player-coach iterative validation" mode | To be built: UAR goal engine |
| **Non-interactive / headless** | `claude -p` / `--print`; `--output-format stream-json --verbose` | `codex exec` (prints final msg to stdout, streams to stderr); `--json`, `-o`, `--output-schema`; `resume` | `opencode run`; `opencode serve` (OpenAPI 3.1 + JS/TS SDK) | `kimi -p`; JSONL event stream; session continuation | `minimax -p`; `mmx --non-interactive --output json` (multimodal gen CLI) | Axum service |
| **Hooks / lifecycle** | ~30 events; Stop/SubagentStop/PreCompact can **block** (exit 2 forces continuation); prompt-based hooks | PreToolUse hooks; execpolicy/sandbox gates; no rich Stop-hook parity | Plugin system (`opencode` field: commands/tools/hooks); session events | Hooks referenced in K2.7 docs; less mature | config.toml modes (Normal/Plan/Agent/YOLO/RLM/Duo); `/set approval_mode` | Cedar policy + ractor supervision |
| **Skills (SKILL.md)** | Native origin of spec; progressive disclosure; **budget overflow** (see note) | Adopted (Dec 2025); reads standard locations | Native + `.claude/skills`, `.agents/skills`; compaction-resilient reinjection plugins | K2.7 CLI advertises "skills" | Reads SKILL.md via symlink into `~/.claude/skills` etc. | Native skill system ships with UAR |
| **State persistence / resume** | `--resume`/`--continue` restores active goal (timer/tokens reset); session dirs | `~/.codex/sessions` rollout files; `codex exec resume --last`/`<id>`; `--ephemeral` to skip | `opencode session list`, export/import JSON, `--continue`/`--session` | `~/.kimi-code` sessions; persistent multi-turn | `~/.minimax/sessions`, `--resume latest`/`<id>` | SurrealDB + provenance |
| **Autonomy disposition** | **Prompt-steered toward consent**: AskUserQuestion tool, proactiveness restraint, "just stop" after a file (nilenso, verified from system prompt) | **RL-trained for persistence**: "keep going until fully resolved," "persist… persevere even when function calls fail"; codex models drop the autonomy prose entirely (baked into post-training) | Model-dependent (BYO model); harness is neutral | Model-dependent; K2.6/K2.7 open weights | Model-dependent (M2/M3); M2.7 RL'd on "OpenClaw" harness for self-evolution | Policy-governed; disposition set by dispatch target |
| **Config/instruction file** | CLAUDE.md; `.claude/` | AGENTS.md (layered, nearest-wins, 32 KiB default cap, AGENTS.override.md) | AGENTS.md (falls back to CLAUDE.md); opencode.json | AGENTS.md-style | AGENTS.md (`minimax init`) | DID identity + Cedar |
| **IDE embedding** | ACP-capable | ACP-capable | ACP-capable | ACP (Zed/JetBrains) | — | ACP/A2A/MCP/AG-UI |

**Note — the "MMX" ambiguity (must be stated explicitly):** "MMX" does **not** resolve to a single coding harness. Three candidates exist, and PSP should treat this as an open identification:
- **`mmx` / MMX-CLI (MiniMax official, `mmx-cli` on npm):** a *multimodal generation* CLI (text/image/video/speech/music/vision/search), **not** a coding agent harness. It ships a SKILL.md that other agents (Claude Code, OpenCode, Cursor) call as a *tool*. It has clean non-interactive flags but no autonomous code-editing loop or goal primitive. This is the most literal match for "MMX."
- **Unofficial `minimax` / MiniMax-CLI (Rust, community):** a genuine coding TUI with Normal/Plan/Agent/YOLO/RLM/**Duo** modes, AGENTS.md, skills, subagents, compaction config, `-p` non-interactive, `--resume`. Duo mode ("player-coach autocoding with iterative validation") is the closest thing to a native goal loop. This is the most plausible match if Travis means "a MiniMax coding harness."
- **Xiaomi MiMo Code:** a distinct open-source agentic coding harness reported to target ultra-long (200+ step) tasks; note MiMo's API is Claude-Code-harness-compatible and Codex-incompatible (per BUILD-AND-FIND), so in practice MiMo models are often driven *through* the Claude Code harness.

**Recommendation:** PSP should implement the MMX driver against the **unofficial Rust `minimax` coding CLI** (Duo mode + `-p` + `--resume`) as the primary target, treat **MMX-CLI (`mmx`)** as a *tool/skill* dependency rather than a harness, and keep a **MiMo Code** stub. Flag the ambiguity in the spec so a later decision is a config change, not a rewrite. Confidence in this identification: low-medium.

**Note — Claude Code skill budget (refines C1/C3):** Skill descriptions load into a budget governed by `skillListingBudgetFraction` (default 1% of context window). Practitioner reports (claudefa.st) put the practical ceiling around 75–125 skills before the budget is exhausted at the default 1% setting, and describe a behavior change at v2.1.129 from silent per-description truncation to dropping entire descriptions for low-use skills (ranked by recency/frequency) with a startup warning naming which went dark. I was unable to confirm the exact token ceiling and version-by-version truncation caps against a first-party Anthropic source, so treat the specific numbers as practitioner-reported rather than verified. The *direction* is well-established and is the empirical basis for PSP components C1 (trigger reliability) and C3 (tiered activation): Travis's ~140 skills exceed the default ceiling, so on the default setting a meaningful fraction are silently dark.

---

## 3. Architecture Recommendation

### 3.1 Recommended architecture: Hybrid (Option C), realized as a UAR-native goal engine (Option D) at the top

The goal abstraction has **three planes**:

**Plane 1 — Portable Goal Contract (data).** A versioned `goal-contract.json` (+ human-readable `goal.md`) that travels with the work, expressed in Travis's OpenSpec/PMPO vocabulary. It contains the goal statement, machine-verifiable exit criteria, scope constraints, failure paths, phase decomposition, and evidence requirements. It is harness-agnostic because it is inert data.

**Plane 2 — In-harness compliance skill (thin, portable SKILL.md).** One SKILL.md — `psp-goal-runner` — installed into every harness via the standard skill locations. Its job is *not* to make the model loop by willpower. Its job is: (a) read the Goal Contract and `STATUS.md`; (b) detect whether a native goal primitive exists (`/goal` in Claude Code/Codex) and, if so, hand the completion condition to it; (c) enforce the single-writer and evidence-emission discipline so the outer supervisor can verify; (d) emit a structured completion/failure sentinel (`GOAL_COMPLETE`/`GOAL_FAILED` + evidence manifest). It is the C1/C2 anchor: short, always-triggering, re-anchored after compaction.

**Plane 3 — Out-of-harness Goal Supervisor (Rust, authoritative).** A UAR-native engine that owns the loop that actually matters. For each phase in the contract it spawns a **fresh harness session** (Ralph-style, context-overflow-proof), passes the contract + STATUS.md, waits for the sentinel, runs the **verification gate** (command + expected exit code / file predicates) itself — never trusting the agent's self-report — updates STATUS.md, and either advances, retries (up to max attempts), reverts, or escalates. This plane is where Cedar policy, provenance, DID identity, and the sycophancy-correction gate live. It dispatches through `knowme:harness` adapters; BossFang is the remote-dispatch surface.

The convergence question Travis raised — in-harness steering hooks vs out-of-harness invocation adapters — resolves as: **out-of-harness invocation is the primary control surface; in-harness hooks are a per-driver optimization, not the contract.** The supervisor treats a native `/goal` or a Stop-hook as *one implementation of a phase runner*, interchangeable with a plain `exec` call wrapped in the outer loop. This is directly supported by the Ralph community's own consensus: "If you're implementing Ralph as part of the agent harness via skill/command/etc you are missing the point" — the fresh context must come from *outside*.

### 3.2 Alternatives considered and rejected

**(a) Pure skill (SKILL.md instructs the model to loop).** *Rejected as the primary mechanism.* Portable and zero-infrastructure, but it cannot override RL-trained stopping (Claude) or force stopping (Codex over-runs), and it dies with the context window — the exact failure mode Travis already hit with Claude Code. It survives only as Plane 2, the compliance layer, never as the enforcement layer.

**(b) Pure out-of-harness Rust orchestrator (no native delegation).** *Rejected as needlessly wasteful.* It ignores that Codex Goal Mode and Claude `/goal` genuinely reduce per-phase orchestration cost for phases that fit in one context window. Refusing to delegate to them means paying fresh-session startup cost on every micro-iteration. Keep them as fast paths inside the phase runner.

**(c) UAR-native goal engine treating every harness as a dispatch target.** *Adopted — but as the top of the hybrid, not as a replacement for native primitives.* This is the correct home for governance, identity, and audit. The nuance: it must dispatch to native primitives when they help and fall back to fresh-session phasing when they don't, rather than reimplementing a monolithic loop that never uses them.

### 3.3 Capability-inversion boundary (Cargo level)

The Goal Supervisor crate **must not depend on any write-actuator crate.** It depends only on: the contract schema crate, the verification-gate crate (read-only: runs commands, reads predicates, returns pass/fail), the harness-driver *trait* crate (dispatch interface), and the Cedar policy crate. Write actuators (the harness drivers that actually spawn `codex exec`/`claude -p` and let the model edit files) are injected as trait objects. This keeps the supervisor unable to mutate the workspace directly — it can only *ask a driver to* and then *verify the result* — which is the structural guarantee behind M1-first measurement gating and single-writer discipline.

---

## 4. Functional Specification

### 4.1 The Goal Contract schema (`goal-contract.json`)

```json
{
  "contract_version": "PSP-GOAL-1.0",
  "goal_id": "did:knowme:goal:...",
  "statement": "Migrate gen_ui_core FFI surface to the new error enum until all call sites compile and tests pass.",
  "scope": {
    "allow_paths": ["crates/gen_ui_core/**"],
    "deny_paths": ["**/secrets/**", "infra/**"],
    "must_not_change": ["public API of gen_ui_core::render()"]
  },
  "phases": [
    {
      "id": "P1",
      "name": "spec",
      "artifact": "openspec/proposal.md",
      "exit_criteria": [{"type": "file_exists", "path": "openspec/proposal.md"}]
    },
    {
      "id": "P2",
      "name": "execute",
      "exit_criteria": [
        {"type": "command", "cmd": "cargo test -p gen_ui_core", "expect_exit": 0},
        {"type": "command", "cmd": "cargo clippy -- -D warnings", "expect_exit": 0}
      ]
    }
  ],
  "exit_criteria_global": [
    {"type": "command", "cmd": "cargo build --workspace", "expect_exit": 0}
  ],
  "evidence": {
    "require": ["command_stdout", "git_diff", "test_report"],
    "sink": "surrealdb://provenance/goal/{goal_id}"
  },
  "failure_policy": {
    "max_attempts_per_phase": 3,
    "on_exhaust": "revert_and_escalate",
    "revert": "git",
    "escalation_target": "trusted-host-approval-gate"
  },
  "governance": {
    "cedar_policy_id": "psp.goal.migrate",
    "identity": "did:knowme:agent:...",
    "human_gate": "P2:pre-merge"
  }
}
```

`STATUS.md` is the human/agent-readable projection the fresh session reads each phase (current phase, last evaluator reason, checklist, last evidence pointers). `goal-state.json` is the machine mirror the supervisor owns (single-writer: only the supervisor writes it).

This maps directly to Travis's OpenSpec conventions: `proposal.md`/`tasks.md`/`design.md`/`spec-delta.md` become phase artifacts with `file_exists`/`command` exit criteria; PMPO v2's Task Loop (Spec→Plan→Execute→Reflect) is the default phase template, and the Evolution Loop (Compile→Evaluate→Optimize→Promote) becomes a meta-contract the standing loop runs over accumulated goal telemetry.

### 4.2 Per-harness driver behaviors

Each driver implements a `PhaseRunner` trait: `run_phase(contract, phase, status) -> PhaseOutcome`.

- **Claude Code driver.** Fast path: `claude -p "/goal <exit-condition>"` with `--output-format stream-json --verbose`, paired with **auto mode** (goal alone does not grant permissions) and a **prompt-based Stop hook** as belt-and-suspenders (exit 2 forces continuation on a real failing check). Per the Claude Code docs, `/goal` "is a wrapper around a session-scoped prompt-based Stop hook" whose evaluator "does not call tools, so it can only judge what Claude has already surfaced in the conversation" — meaning the completion condition must be written so Claude's own transcript output demonstrates it. Overflow path: fresh `claude -p` per phase; do not rely on `/goal` surviving compaction (Anthropic docs confirm `/goal` does not solve context overflow, and the compaction-re-attachment limits Travis documented apply). C2 re-anchor skill is loaded so the compliance instructions survive compaction.
- **Codex driver.** Fast path: `codex exec --full-auto` (or `-a never -s workspace-write` for unattended workspace writes without approvals) — Codex's RL persistence means a single `exec` often runs the phase to completion. Optionally Goal Mode where a ChatGPT-auth session is available. Resume via `codex exec resume`. Known risk: post-compaction goal dilution (continuation prompt ~462 tokens/turn; a re-attachment fix has been proposed but treat as unreliable). AGENTS.md carries the compliance instructions natively (layered, nearest-wins, 32 KiB default cap).
- **OpenCode driver.** No native goal. Start `opencode serve` once; drive phases via the OpenAPI/`@opencode-ai/sdk` `session.create`/`session.prompt`, or `opencode run --attach`. Skills load from `.claude/skills`/`.agents/skills`, so the compliance skill is portable unchanged. Outer loop is mandatory (this is the reference implementation of the emulation layer).
- **Kimi Code driver.** No native goal. `kimi -p` with JSONL event stream; parse `<promise>`-style completion sentinel; session continuation for resume. Ralph loop is the documented pattern; wrap it in the supervisor.
- **MMX driver.** Target unofficial `minimax` Rust CLI: `minimax -p` in Agent/Duo mode, `--resume`, AGENTS.md. Duo mode's iterative validation is used as an in-harness fast path; the outer loop remains authoritative. Treat MMX-CLI (`mmx`) as a tool dependency, MiMo Code as a stub.
- **UAR driver.** Native: the supervisor calls the UAR goal engine in-process; no subprocess. This is the highest-observability path (ractor actors, direct SurrealDB provenance).

### 4.3 Verification gates, failure/escalation

Gates run **in the supervisor, read-only, after the sentinel**. A phase is complete only when every `exit_criteria` predicate passes on evidence the supervisor collected itself (command exit codes, file predicates, git diff), never on the agent's assertion. On failure: retry with the evaluator's reason injected into the next fresh session (bounded by `max_attempts_per_phase`); on exhaust, `git revert` the phase and escalate to the trusted-host human approval gate. Human approval gates exist **only in the trusted host layer** — never inside a harness driver — consistent with Travis's stated discipline.

### 4.4 Cedar / audit integration points

Every dispatch is a Cedar-authorized action keyed on DID identity: `(principal=agent-did, action=run_phase, resource=goal_id+scope)`. Scope `allow_paths`/`deny_paths` are enforced twice — as harness sandbox flags (`-s workspace-write`, OpenCode permissions, deny-globs) *and* as a post-hoc Cedar check on the git diff, so a harness that ignores its sandbox still fails the gate. All evidence and every phase transition are written to SurrealDB provenance under the goal DID. BossFang remote dispatch inherits the same identity/Cedar/audit plane.

### 4.5 Sycophancy-correction gate placement

Place the S-01–S-08 critic **between phase completion and human/merge gate, and it receives only the produced artifact + the Goal Contract exit criteria — never the generation transcript or the agent's self-reported reasoning.** This is the same isolation Travis already specified: the critic judges the diff/tests/artifact against the contract, not the story the builder told. Because the supervisor already holds the artifact and evidence out-of-harness, this gate is naturally a supervisor stage, not a skill. It is especially important on the Codex path (terse, confident output) and on any path where the evaluator model saw the generation history.

---

## 5. Implementation Plan (M1-first, Rust-first)

**Phase M0 — Measure before building (no new engine yet).** Before writing the supervisor, instrument the thing Travis already trusts. On `prometheus-skill-pack`: (a) audit which of the ~140 skills are silently dark under the current `skillListingBudgetFraction`; raise the fraction or tier per C3 and record the delta. (b) Run the *same* representative goal through Codex `exec` + a trivial bash Ralph loop vs Claude `-p` + `/goal` vs Claude `-p` + Stop-hook, capturing: completion (did it finish), verification-gate pass rate, token cost, wall-clock, and post-hoc BUILD-AND-FIND-style "findability" of the artifact. **This is the M1 gate:** you are measuring whether an outer loop closes the Claude-finishes gap and by how much, *before* committing to the full engine. Threshold to proceed: outer loop must bring Claude Code phase-completion to parity with Codex (≥ the Codex completion rate) on the pilot goal.

**Phase M1 — Contract + compliance skill.** Ship `goal-contract.json` schema crate (`psp-goal-contract`) and the portable `psp-goal-runner` SKILL.md + `STATUS.md` convention into `prometheus-skill-pack`. No orchestrator yet — validate the contract is authored correctly and the sentinel/evidence discipline holds when a human drives phases manually across all six harnesses.

**Phase M2 — Goal Supervisor (Rust) with two drivers.** In `universal-agent-runtime`, build:
- `psp-goal-supervisor` (crate; owns the loop, STATUS.md/goal-state.json single-writer, no write-actuator deps)
- `psp-goal-verify` (crate; read-only gate runner)
- `psp-harness-driver` (trait crate) + `driver-codex`, `driver-claude` (first two, since they're Travis's baseline and the RL-persistence contrast).

Wire Cedar + SurrealDB provenance. Exit criterion: reproduce the M0 pilot goal end-to-end through the supervisor with equal-or-better completion and full audit trail.

**Phase M3 — Remaining drivers + BossFang dispatch.** Add `driver-opencode` (reference emulation via `serve`+SDK), `driver-kimi`, `driver-mmx`, `driver-uar`. Expose the supervisor through `knowme:harness` adapters under one identity/Cedar/audit plane; BossFang (librefang fork) becomes the remote-dispatch surface. Integrate the S-01–S-08 sycophancy critic as a supervisor stage.

**Phase M4 — KnowMe integration + Evolution Loop.** Surface goal runs in `know-me-system` (Tauri 2 + React 19 desktop first, via `gen_ui_core`; Flutter mobile as read/monitor). Feed goal telemetry into PMPO v2's Evolution Loop so the standing loop can Compile→Evaluate→Optimize→Promote contract templates and driver parameters.

**Stack discipline:** Rust-first (Axum 0.8, Tauri 2) for supervisor/drivers/gates; TypeScript (Mastra) only for the OpenCode SDK glue and any agent-side orchestration that must run in-process with a JS harness. ACP is the embedding path for IDE surfaces; A2A/MCP for inter-agent and tool planes.

---

## 6. Framing & Recommended Goal Adjustments

**What cannot be equalized across harnesses (be honest in PSP docs):**
- **Autonomy disposition.** Codex is RL-post-trained to persist ("keep going until fully resolved… persevere even when function calls fail"); Claude Code is trained/prompted toward consent (AskUserQuestion, restraint, "just stop"). A skill cannot flip either. The outer loop can make Claude *finish* (by re-invoking it), but it cannot make Claude *want* to finish within a turn, and it cannot stop Codex from over-reaching on an existing codebase.
- **Artifact character.** The controlled evidence (BUILD-AND-FIND) shows the two produce *differently-shaped* outputs (Codex terser/flatter and cheaper in tokens; Claude Opus higher coverage but heavier prose/inspection cost) — not a simple better/worse. An outer loop does not converge these; it only guarantees the exit criteria are met.
- **Context-overflow economics.** Every harness degrades as context fills; only fresh-session phasing escapes it. This is a law, not a tuning parameter.

**What PSP should claim:** "One Goal Contract, verifiable identically everywhere; equivalent *completion and verification* behavior across harnesses; native primitives used where present." That is deliverable and defensible.

**What PSP must NOT claim:** behavioral parity ("the same goal behaves identically on every harness") or quality parity ("Claude via PSP == Codex"). The scenario that hurts Prometheus most is shipping a "harness-agnostic goal" that implies parity, then a customer running the same contract on Claude Code and Codex and getting materially different artifacts — turning PSP's central promise into its most visible failure. Claiming *completion + verification* parity (true) instead of *behavioral* parity (false) inoculates against this.

**Strategic positioning against the commoditizing harness layer:** The harnesses are converging (all adding `/goal`, all reading SKILL.md, all speaking ACP). That convergence is *good* for Prometheus: it commoditizes exactly the layer PSP does not want to own, and it makes the durable value the **governance, verification, provenance, and cross-harness contract** — the UAR/Cedar/BossFang plane — which no single harness vendor will build because it is deliberately vendor-neutral. Position PSP's goal abstraction as the **portable verification-and-governance contract that outlives any harness**, explicitly not as a better harness.

---

## 7. Risks & Open Questions

1. **The parity trap (highest business risk).** Named above. Mitigation: contract-level claims only; ship a "harness behavior profile" alongside every goal run so differences are visible, not hidden.
2. **MMX identity risk.** If Travis means MiMo Code or the official `mmx` rather than the unofficial `minimax` CLI, the MMX driver retargets. Mitigation: driver-trait isolation makes this a config/driver swap, not an architecture change. **Open question for Travis: which "MMX" do you mean?**
3. **Goal Mode auth / surface constraint.** Codex Goal Mode reached GA across the Codex desktop app, IDE extension, and CLI, with support in ACP-compatible clients including Zed. My earlier read that Goal Mode requires ChatGPT-subscription auth (and is therefore unavailable to API-key/CI runs) is practitioner-reported and I could **not** re-confirm it against the current release — the v0.140 CLI even added `/import` to pull config out of Claude Code, suggesting the surface is still moving. **Verify the auth constraint against the current Codex release before relying on it.** Regardless, the safe default for unattended BossFang/CI runs is `codex exec` + outer loop; Goal Mode is a local-interactive fast path only. (Confidence: medium.)
4. **Native-primitive drift.** `/goal` semantics in both harnesses are young (Claude v2.1.139+, Codex v0.128–0.133+) and changing monthly; post-compaction dilution is an acknowledged, only-partially-fixed bug in both. Mitigation: treat native primitives as optional fast paths; the outer loop is the contract-of-record so drift degrades performance, not correctness.
5. **Skill-budget interaction.** With ~140 skills, the compliance skill itself could be evicted/dark. Mitigation: C1/C2/C3 — pin `psp-goal-runner` as high-priority, keep it tiny, re-anchor on PreCompact, and (on the outer-loop path) inject it fresh per phase so it never depends on surviving compaction.
6. **Verification expressiveness.** Contracts whose "done" is subjective ("improve the design") cannot be gated; both `/goal` docs warn on this. Mitigation: require every contract phase to carry at least one machine-checkable predicate; route subjective judgment to the sycophancy critic + human gate, never to the loop exit condition.
7. **Cost/latency of fresh-session phasing.** Fresh sessions re-pay context-loading cost each phase. Mitigation: phase granularity is a tuning knob; use native `/goal` within a phase that fits one window, fresh sessions only at phase boundaries. Measure in M0.
8. **Single-writer enforcement under parallelism.** If future work runs phases in parallel worktrees (per the agent-teams + Ralph hybrid pattern), goal-state.json single-writer must become a supervisor-mediated queue. Out of scope for M2; flagged for M3+.

**Confidence ranges.** Native `/goal` mechanics (Claude, Codex) — high (primary docs). Codex RL-persistence attribution — medium-high (nilenso inference from system prompts, explicitly interpretive). BUILD-AND-FIND findings — medium (single-author preprint, small Rust-only panel; directionally credible, not a leaderboard, and the specific effort multipliers/artifact counts could not be re-verified from the source PDF). MMX identification — low-medium (genuinely ambiguous; three live candidates). The claim that an outer loop closes Travis's Claude-vs-Codex gap — medium, and explicitly the thing M0/M1 must measure before the full build.