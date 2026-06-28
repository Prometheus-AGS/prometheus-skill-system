# Analysis — goal-loop-support

**Phase:** goal-loop-support
**Analyzed:** 2026-06-27
**Pipeline tiers used:** Tier 1 (GitHub), Tier 4 (web search)
**Research focus:** Platform parity for `/goal` across Claude Code, OpenCode, Codex CLI, Kimi Code, and Zed

---

## Research Summary

The question is: **how does each target platform implement (or can implement) a goal-driven autonomous loop, and what does KBD need to provide to achieve parity across all of them?**

---

## Platform-by-Platform Findings

### 1. Claude Code (native `/goal`)

| Property | Detail |
|----------|--------|
| **Command** | `/goal [--tokens N] [--worktree] <stopping condition>` |
| **Loop driver** | After each turn, a separate Haiku instance evaluates stopping condition → yes/no + reason |
| **Skill invocation** | Skills are `SKILL.md` files in `~/.claude/skills/`; invoked via `/skill-name` or auto-triggered by agent |
| **Stopping** | Condition satisfied · token ceiling (`--tokens`) · turn limit in condition · Ctrl+C |
| **Worktree isolation** | `--worktree` flag spawns a clean git checkout |
| **Parity gap** | None — this is the reference implementation. KBD must AUGMENT (add Ideation/Spec/multi-phase), not replace. |
| **KBD integration** | For multi-phase goals, KBD orchestrates above `/goal`; single-phase Creation tasks can delegate to native `/goal` |

### 2. Codex CLI (Ralph Loop / `/goal`)

| Property | Detail |
|----------|--------|
| **Version added** | 0.128.0 (30 April 2026) |
| **Activation** | One flag in `~/.codex/config.toml` + restart |
| **Command** | `/goal <objective>` · `/goal pause` · `/goal resume` · `/goal clear` |
| **Loop driver** | Runtime continuation via two prompt files: `goals/continuation.md` (re-invokes) and `goals/budget_limit.md` (exit judgment); maker ≠ evaluator |
| **Stopping** | Goal achieved · token budget hit · turn failure limit (added v0.138.0) · no-progress turns |
| **Custom skills** | Via `AGENTS.md` — instructions loaded before work; no native SKILL.md support; skills go in AGENTS.md or project config |
| **Slash commands** | `/goal` is built-in; custom commands via `AGENTS.md` sections |
| **Parity gap** | No multi-phase support natively. KBD can wrap by: writing `AGENTS.md` with KBD goal context + running `/goal` per phase |
| **KBD integration** | KBD writes a session-specific `AGENTS.md` with goal context, then invokes `codex /goal "<phase-condition>"` per phase |

### 3. OpenCode (plugin ecosystem)

| Property | Detail |
|----------|--------|
| **Native goal** | No built-in `/goal` — but the `opencode-goal-plugin` (prevalentWare, also watzon/opencode-goal) provides it |
| **Plugin mechanism** | `.opencode/commands/` markdown slash command files; `opencode.json` or `config.toml` plugin declarations |
| **Goal plugin features** | `/goal`, `/goal history`, `/goal edit`, `/goal pause`, `/goal resume`; persistent per-session state; `auto_continue`, `max_auto_turns`, `no_progress_token_threshold` config |
| **Agent tools from plugin** | `get_goal`, `create_goal`, `set_goal`, `update_goal`, `clear_goal` — agent can manipulate goal state |
| **Continuation** | Driven by `session.idle` and `session.status` OpenCode events |
| **Skills** | OpenCode has skills via `~/.opencode/skills/` (KBD already installs here) |
| **Parity gap** | Goal plugin must be installed. KBD's `install-skills-flat.sh` should auto-install the goal plugin when setting up OpenCode. |
| **KBD integration** | KBD installs goal plugin + writes goal state via agent tool calls (`create_goal`). Per-phase goal loop driven by `/loop-tick`. |

### 4. Kimi Code

| Property | Detail |
|----------|--------|
| **Native goal** | `/goal next <objective>` built-in; sequential goal queue — when current completes, picks up next automatically |
| **YOLO mode** | `/yolo` skips approval; asks confirmation before starting a goal, suggests switching to Auto for unattended work |
| **Skills** | Full `SKILL.md` support in `~/.kimi-code/skills/` (KBD already installs here via `install-skills-flat.sh`); skill discovery auto-injected into system prompt; `extra_skill_dirs` in `config.toml` |
| **Slash commands** | `/skill:<name>` — reads SKILL.md content and sends to agent. Skills appear in TUI slash command panel. |
| **Stopping** | Goal marked complete with verified evidence; blocked with concrete blocker; turn budget |
| **Gap** | Kimi's `/goal next` is a queue, not a condition-based loop. KBD must provide the stopping condition evaluator as a skill. |
| **KBD integration** | KBD ships `kbd-goal` as a SKILL.md skill; Kimi auto-discovers it. KBD evaluator agent runs as a sub-skill call checking condition after each turn. `/goal next` queues each phase. |

### 5. Zed (ACP + Skills)

| Property | Detail |
|----------|--------|
| **Protocol** | Agent Client Protocol (ACP) — JSON-RPC 2.0 over stdio; external agents (Claude Code, Codex, OpenCode, Gemini CLI) connect as native providers |
| **Skills** | Full `SKILL.md` support at `zed.dev/docs/ai/skills`; same format as agentskills.io; installed to `~/.zed/skills/` |
| **Invocation** | `/skill-name` slash command in message editor; OR agent auto-invokes via skill tool |
| **Autonomous control** | `disable-model-invocation: true` in frontmatter prevents auto-invoke; skill still available as slash command |
| **Native goal loop** | No built-in `/goal` native to Zed itself — but connected agents (Claude Code, Codex via ACP) bring their goal loops |
| **Loop pattern** | For Zed sessions running Claude Code or Codex as ACP agent, `/goal` is available from that agent. For Zed running its own loop: custom skill must implement loop via repeated session/prompt calls |
| **Gap** | Zed has no standalone goal loop when used with its own LLM (not a connected external agent). KBD must provide it as a skill that drives a loop using Zed's `session/prompt` flow. |
| **KBD integration** | KBD ships `kbd-goal` SKILL.md to `~/.zed/skills/`. When Zed is connected to Claude Code/Codex via ACP, native `/goal` is usable. When Zed is standalone, KBD skill emulates the loop via repeated prompting with condition evaluation. |

---

## Cross-Platform Parity Matrix

| Feature | Claude Code | Codex CLI | OpenCode | Kimi Code | Zed |
|---------|------------|-----------|----------|-----------|-----|
| Native `/goal` | ✅ Full | ✅ Full (v0.128+) | ⚠️ Plugin | ✅ Queue-based | ❌ Via ACP agent |
| Condition evaluator (maker≠evaluator) | ✅ Haiku | ✅ budget_limit.md | ⚠️ Plugin logic | ❌ Missing | ❌ Missing |
| Skill loading (SKILL.md) | ✅ | ❌ AGENTS.md only | ✅ | ✅ | ✅ |
| Multi-phase orchestration | ❌ | ❌ | ❌ | ⚠️ Queue only | ❌ |
| Worktree isolation | ✅ `--worktree` | ⚠️ Manual | ⚠️ Manual | ❌ | ❌ |
| Session state persistence | ✅ | ✅ config.toml | ✅ Plugin state | ✅ | ❌ |
| Human escalation gates | ⚠️ Ctrl+C only | ⚠️ Ctrl+C only | ✅ Plugin blocker | ✅ YOLO gate | ❌ |
| KBD skills installed | ✅ | ❌ | ✅ | ✅ | ✅ |

**Critical gaps requiring KBD to implement:**
- **Condition evaluator** for Kimi Code and Zed (standalone)
- **Multi-phase orchestration** for ALL platforms (none support it natively)
- **AGENTS.md integration** for Codex (skills reach Codex via AGENTS.md, not SKILL.md)
- **OpenCode goal plugin auto-install** in `install-skills-flat.sh`

---

## Implementation Approach Per Platform

### The Shared Abstraction: `kbd-goal` Skill + Evaluator Agent

The key insight from research: every platform that has `/goal` uses the same architecture:
1. **Writer/Builder** — the main agent context doing work
2. **Evaluator** — a separate, lighter model checking "is condition X met?"
3. **Continuation prompt** — injected when condition not met, guiding next turn
4. **Budget/turn limit** — hard ceiling preventing infinite spin

KBD must implement this as a **platform-detection shim** in the `kbd-goal` skill:

```
/kbd-goal "build weekly standup generator" --phases ideation,spec,creation --stop "TASKS.md complete, all tests pass"
```

The skill detects which tool is active (from `project.json → tool` field) and routes:

| Tool | Strategy |
|------|---------|
| `claude-code` | For single-phase: delegate to native `/goal`. For multi-phase: KBD orchestrates; delegates per-phase to `/goal`. |
| `codex` | Write `goals/continuation.md` + `goals/budget_limit.md` snippets; invoke `codex /goal "<phase-condition>"` per phase |
| `opencode` | Ensure goal plugin is installed; use `create_goal` agent tool; KBD manages phase transitions |
| `kimi` | Use `/goal next` to queue phases; KBD evaluator skill checks condition after each turn |
| `zed` | If ACP agent is Claude Code/Codex: delegate to their native `/goal`. If standalone: KBD skill drives loop via repeated `session/prompt` |

### The Evaluator Agent Pattern

For platforms without a native separate evaluator (Kimi standalone, Zed standalone), KBD will ship:

**`agents/kbd-goal-evaluator.md`** — a subagent definition with:
- System prompt: "You are a goal condition evaluator. Given a stopping condition and a transcript summary, return PASS or FAIL and a one-sentence reason. Be strict. Do not accept partial completion."
- Model: small/fast (Haiku or equivalent)
- Tools: file read only (to read test output, STATE.md, etc.)
- No write access — pure evaluation

This agent is invoked by the KBD loop tick script after each execution turn, before deciding to continue or stop.

### Codex AGENTS.md Integration

Codex doesn't support SKILL.md — skills reach it via `AGENTS.md`. KBD will:
1. Auto-generate a session `AGENTS.md` prefix when `tool == "codex"` containing:
   - KBD phase context (current phase, goal, stopping condition)
   - Key rules (from relevant skills)
   - Continuation pattern (`goals/continuation.md` template)
2. Ship `goals/continuation.md` and `goals/budget_limit.md` template files to `~/.codex/goals/` during install

### OpenCode Plugin Auto-Install

`install-skills-flat.sh` will gain an OpenCode section that:
1. Checks if `opencode-goal-plugin` is installed (`opencode plugins list`)
2. If not: `npx @prevalentware/opencode-goal-plugin install` (or `opencode plugins add`)
3. Writes plugin config to `.opencode/config.toml` with KBD-tuned defaults (`max_auto_turns: 20`, `no_progress_token_threshold: 5000`)

### Kimi Code Evaluator Skill

Since Kimi's `/goal next` is a queue (not condition-based), KBD ships a `kbd-goal-check` skill that:
- Is invoked after each turn by the agent (auto-discovered)
- Reads `STATE.md` / test output / defined measurable criteria
- Returns PASS (with evidence) or CONTINUE (with next action hint)
- On PASS: calls Kimi's goal completion mechanism

### Zed ACP Strategy

Zed's strategy is two-track:
1. **ACP-connected agent** (Claude Code/Codex as ACP backend): The connected agent's native `/goal` is available. KBD skill simply delegates to it.
2. **Zed standalone** (e.g., using Zed's own built-in model): KBD `kbd-goal` skill implements the loop by:
   - Writing `STATE.md` after each turn
   - Invoking `kbd-goal-evaluator` subagent via Zed's skill tool mechanism
   - Reading PASS/FAIL
   - If FAIL: injecting continuation guidance as next user message

---

## Multi-Phase Goal Decomposition (All Platforms)

This is uniformly missing from all platforms. KBD's value-add. The architecture:

```
/kbd-goal "build weekly standup generator" --phases ideation,spec,creation
```

Internally creates:
```
.kbd-orchestrator/goals/<goal-slug>/
  goal.json              # goal definition, phases, stopping conditions per phase
  IDEAS.md               # Ideation phase output
  SPEC.md                # Specification phase output
  TASKS.md               # Creation phase decomposition
  STATE.md               # Execution state per task
  loop.json              # Outer loop definition (reuses pmpo-outer-loop schema)
```

Each phase is a **child KBD phase** under the goal phase. The outer loop (`/loop-tick`) advances through phases, gating on:
- **Ideation → Spec**: human approval of `IDEAS.md` (or auto if `--auto-gates` flag)
- **Spec → Creation**: human approval of `SPEC.md`
- **Creation → Deployment**: explicit deploy parameters required

---

## Build vs. Adopt Decisions

| Component | Decision | Rationale |
|-----------|---------|-----------|
| Evaluator agent | **BUILD** — `agents/kbd-goal-evaluator.md` | All platforms except Claude Code/Codex lack one; existing `sycophancy-correction` MCP is wrong abstraction (grades prose, not goal conditions) |
| OpenCode goal loop | **ADOPT** — `@prevalentware/opencode-goal-plugin` | Mature, covers all config knobs; KBD auto-installs and configures |
| Codex goal loop | **ADOPT** — native `codex /goal` | Already shipped; KBD generates supporting `continuation.md`/`budget_limit.md` |
| Multi-phase orchestration | **BUILD** — new `kbd-goal` skill + `goal.json` schema | No platform has this; builds on existing KBD child phases + pmpo-outer-loop |
| Kimi evaluator | **BUILD** — `kbd-goal-check` SKILL.md | Kimi's queue model needs a condition-check skill |
| Zed standalone loop | **BUILD** — logic in `kbd-goal` skill detecting Zed+standalone | Zed ACP with external agent delegates; standalone needs KBD to emulate |
| Platform detection | **BUILD** — extend existing `project.json → tool` mechanism | Already exists; needs `--tool` override and auto-detect expansion |

---

## Open Questions for Plan Phase

1. **`sycophancy-correction` as evaluator?** The MCP's `detect_sycophancy` tool grades prose quality, not goal completion. NOT suitable as the goal evaluator. Confirmed: build a separate `kbd-goal-evaluator` agent.

2. **`loop.json` schema extension**: The existing schema needs a `phases[]` array field and per-phase `stopping_condition`. Minor additive change — backward compatible.

3. **Codex `AGENTS.md` generation**: Should KBD generate a project-level `AGENTS.md` (risks conflicting with user's existing file) or a session-scoped prefix injection? Prefer: write to `.codex-kbd-context.md` and document that users `@include` it from their `AGENTS.md`. Avoids clobbering.

4. **Zed ACP detection**: How to detect if Zed is running with an ACP-connected agent vs. standalone? Check for `ZED_ACP_AGENT` env var or presence of `~/.zed/acp-agents.json`. Needs verification in plan phase.

5. **Inner-loop promotion threshold**: What makes a task "too complex" and auto-promotes to a child phase? Proposal: fail count ≥ 3 on same task OR agent explicitly flags `NEEDS_CHILD_PHASE: true` in `STATE.md`.

---

## Recommended Change List (confirmed from assessment, refined by research)

| Priority | Change ID | Title | Platform Impact |
|----------|-----------|-------|----------------|
| 1 | `goal-001` | `agents/kbd-goal-evaluator.md` — separated evaluator subagent | All platforms lacking native evaluator |
| 2 | `goal-002` | `skills/process/kbd-goal/SKILL.md` — unified entry point with platform detection | All platforms |
| 3 | `goal-003` | `goal.json` schema + `.kbd-orchestrator/goals/` directory structure | All platforms |
| 4 | `goal-004` | Ideation child-phase template (discovery + critic agents, `IDEAS.md` convergence) | All platforms |
| 5 | `goal-005` | Specification child-phase template (writer + reviewer loop, `SPEC.md`) | All platforms |
| 6 | `goal-006` | Creation loop enhancement (TASKS.md, STATE.md, verifier per task) | All platforms |
| 7 | `goal-007` | Claude Code bridge: per-phase `/goal` delegation + multi-phase wrapper | Claude Code |
| 8 | `goal-008` | Codex bridge: `continuation.md` + `budget_limit.md` templates + AGENTS.md prefix | Codex CLI |
| 9 | `goal-009` | OpenCode plugin auto-install in `install-skills-flat.sh` + goal plugin config | OpenCode |
| 10 | `goal-010` | Kimi `kbd-goal-check` evaluator skill + `/goal next` phase queueing | Kimi Code |
| 11 | `goal-011` | Zed dual-track: ACP delegation vs. standalone loop emulation | Zed |
| 12 | `goal-012` | Inner-loop auto-promotion (fail count threshold → `kbd-new-child`) | All platforms |
| 13 | `goal-013` | Goal-time skill/MCP discovery (parse goal → recommend skills + servers) | All platforms |
| 14 | `goal-014` | `loop.json` schema extension: `phases[]` + per-phase `stopping_condition` | All platforms |

14 changes. Suggest grouping into two sub-phases:
- **Sub-phase A (core, changes 1–6)**: Platform-agnostic engine — evaluator, entry point, schema, three phase templates
- **Sub-phase B (integration, changes 7–14)**: Per-platform bridges, inner-loop, discovery, schema extension
