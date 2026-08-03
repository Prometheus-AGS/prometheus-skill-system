# 15 · Hooks & Lifecycle

The loops and skills are visible. The hooks are not — and they are where most of the system's discipline actually lives. Hooks are scripts that fire at lifecycle events: a session starting, a prompt being submitted, a tool about to write a file, a subagent stopping, a session ending. Every guarantee in this guide that sounds automatic — context priming, scope enforcement, the sycophancy gate, the immutable-tests rule, memory write-back — is a hook. This page documents every event and every script that runs on it.

Claude Code's installed hook chain is declared in `hooks/hooks.json`.
Cross-harness lifecycle mappings are declared once in
`shared/harnesses/capabilities.json` and generate the Claude Code, Codex,
OpenCode, and Kimi adapters under `shared/harnesses/generated/`.

## The lifecycle at a glance

```mermaid
sequenceDiagram
    participant Session
    participant Prompt as Each prompt
    participant Tool as Each tool call
    participant Sub as Each subagent
    participant Stop as Session end

    Session->>Session: SessionStart — bounded snapshots + KB health
    Prompt->>Prompt: UserPromptSubmit — immutable bounded context
    Tool->>Tool: PreToolUse — immutable-tests guard (writes only)
    Tool->>Tool: PostToolUse — validate and record local evidence
    Sub->>Sub: SubagentStop[role] — checkpoint + dispatch (reflector → sycophancy gate)
    Stop->>Stop: Stop — atomic local enqueue; never forces continuation
```

## SessionStart

Runs once when a session begins. Sets the stage.

| Order | Script | Purpose |
|---|---|---|
| 1 | `detect-project-context.sh` | Resolve project identity and snapshot scope without network work. |
| 2 | snapshot reader | Load bounded project/shared/global immutable generations. |
| 3 | `memory-outbox-flush.sh` | Hand local uncertain delivery records to durable reconciliation. |
| 4 | `pk-health.sh` | Run read-only `pk lint` once per 24h and preserve failure/empty-output status. |

## UserPromptSubmit

The generated adapter receives the prompt event and queues noncritical work in
the project runtime's `deferred-hooks/` outbox. Prompt and Stop events do not
perform network-heavy memory or learning work inline.

## PreCompact

`kbd-harness-adapter.sh pre_compact claude-code` records a bounded deferred
event. Claude's `SessionStart:compact` path and native post-compact events on
other harnesses render the same canonical re-anchor after compaction.

## PreToolUse — the one remaining guard

A single blocking gate remains. It runs *before* a tool executes and can refuse
it (exit 2).

**Matcher `Bash`: none.** Shell commands are not gated.

**Matcher `Write|Edit|MultiEdit`:**

| Script | What it enforces |
|---|---|
| `protect-tests.sh` | The **BDD Immutable-Tests Rule** — blocks edits to existing `tests/steps/*`, `tests/support/*`, `tests/features/*.feature`; allows new files under `tests/features/drafts/` |

### What was removed, and why

The pre-mutation fence (`kbd-harness-adapter.sh pre_mutation`) previously gated
`Bash`, `Write`, `Edit`, and `MultiEdit` on KBD project identity, control-plane
reachability, and lifecycle state. It was removed, along with
`pipeline-enforce.sh`, `scope-guard.sh`, `check-child-scope.sh`,
`guard-direct-deploy.sh`, and `cedar-skill-gate.sh`.

The fence assumed several agents on several devices contending for one
repository. A single operator does not have that contention, and the cost was severe: every gate
failed closed, so a stopped daemon, an uninitialized runtime, or a phase that
had simply *finished* removed the operator's ability to run `ls`, `git status`,
or `cargo test` — including the very diagnostics each denial recommended. The
scope guards compounded it by flagging edits to a submodule or a sibling project
as out-of-scope, which is ordinary work when a change spans a dependency.

The KBD adapter still runs on `SessionStart`, `UserPromptSubmit`, `Stop`, and
`PreCompact`. Those events only read state and print the re-anchor block; they
never intercept a tool call. Phases, `progress.json`, waypoints, and reflections
are unchanged — they record position, and recording was always the part that
earned its keep.

The immutable-tests rule deserves emphasis. A code-generation agent under pressure to make a failing test pass has an obvious shortcut: edit the test. That shortcut destroys the test's value. `protect-tests.sh` removes the shortcut structurally — the agent *cannot* edit an existing step definition or feature file; it can only add new draft scenarios for human review. This is the same principle as the sycophancy gate, applied to tests: prevent the agent from grading its own homework.

## PostToolUse

Runs after a `Write|Edit|MultiEdit` succeeds. This records and validates local evidence only; durable learning is deferred to Stop enqueue and the worker.

| Order | Script | Purpose |
|---|---|---|
| 1 | `validate-state.sh` (evolver) | Validate evolver state after a write |
| 2 | `validate-gitops-write.sh` (10s) | Confirm written files conform to `TJ-CICD-001` |
| 3 | `scope-record.sh` | Record approved out-of-scope writes to the waypoint so they are not re-flagged |
| 4 | `write-position-reminder.sh` | Refresh `.kbd-orchestrator/position-reminder.txt` |
| 5 | `sycophancy-check-artifact.sh` (35s) | Gate `**/reflection.md` and `**/assessment.md` — exit 2 with Delta/Root-Cause/Corrective-Actions feedback, set `reflect_gate=rejected`; two-rejection soft cap |

## SubagentStop — per-role

Each KBD role has its own matcher. Every role runs a checkpoint and a workflow dispatch; two roles run additional gates.

| Matcher | Scripts (in order) |
|---|---|
| `assessor` | `state-checkpoint(assess)` → `workflow-dispatch(assess)` |
| `analyst` | `state-checkpoint(analyze)` → `workflow-dispatch(analyze)` |
| `planner` | `state-checkpoint(plan)` → `workflow-dispatch(plan)` |
| `executor` | `validate-state.sh` → `state-checkpoint(execute)` → **`evaluate-session.sh` (30s)** → `workflow-dispatch(execute)` |
| `reflector` | **`sycophancy-check-reflection.sh` (35s)** → `log-reflection.sh` → `state-checkpoint(reflect)` → `workflow-dispatch(reflect)` |
| *(fallback, no matcher)* | `subagent-checkpoint-fallback.sh` |

Executor and reflector gates produce local evidence, but they do not acknowledge a memory write. Durable learning is owned by the queue worker, which reconciles the v2 operation receipt before publishing snapshots. The fallback matcher guarantees that even an unrecognized subagent gets a checkpoint — no role falls through silently.

## Stop — atomic enqueue by design

The installed Stop path resolves `karpathy-hook-dispatch.sh` through the stable
plugin directory. `enqueue-learning-job.py` writes a private temporary record,
fsyncs it, and atomically renames it into `pending`; the hook then exits. It
does not call a model, Memory, or a service manager. Duplicate Stop events are
deduplicated by stable record identity. Durable continuity comes from the queue,
receipts, checkpoints, and immutable snapshots—not from forcing an assistant
to keep talking.

## Progress signaling

A lifecycle concern that is not a hook but is mandatory in every orchestration turn: the progress-signal protocol. The first tool call of a KBD/loop turn reads `.kbd-orchestrator/position-reminder.txt` (falling back to `current-waypoint.json` and the phase `progress.json`). Every phase and task then emits start/completion signals with accurate counts read from `progress.json` — never estimated. The `validate-progress-signals.js` script is a merge gate requiring every process skill to declare a `## Progress Signals` section, with a ratchet baseline that can only shrink. This is what keeps long, multi-session work scannable and resumable. The mechanics are also covered in [Loop Architecture](03-loop-architecture.md).

## Strictness and degradation

`PROMETHEUS_REFLECT_STRICTNESS` (loose / standard / strict / adversarial,
default strict) sets the sycophancy gate's sensitivity. It governs a *content*
gate on reflection artifacts, not a tool gate, so it cannot block a command.

Memory, summary, and learning work is noncritical and deferred to the local queue.
No hook can now deny a shell command, and
`PROMETHEUS_SCOPE_ENFORCE` no longer has an effect — the scope guards it
configured were removed.

---

*Previous: [← 14 · The Rust Toolchain & Dynamic Generation](14-rust-toolchain.md) · Next: [16 · CLI & Scripts Reference →](16-cli-and-scripts.md)*
