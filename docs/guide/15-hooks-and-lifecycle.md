# 15 · Hooks & lifecycle

Hooks adapt harness events into bounded context, checkpoints and deferred work.
The source contract is `shared/harnesses/hook-contract.json`; lifecycle capabilities
are declared in `shared/harnesses/capabilities.json`. The adapter generator emits
hook configuration and dispatchers from those inputs. Generated hook files are
release artifacts, not an independent place to change behavior.

## Declared event paths

| Event | Source behavior |
|---|---|
| SessionStart | Canonical KBD context, scoped learning delivery, project/snapshot detection, outbox reconciliation and bounded knowledge health. |
| UserPromptSubmit | Bounded `pk` context and knowledge-gap handling through `karpathy-hook-dispatch.sh`; actual prompt work is not universally network-free. |
| PostToolUse | Record and inspect successful Write/Edit/MultiEdit results, screen reflection artifacts, and enqueue attributed reflection lessons. |
| SubagentStart | Deliver the selected role's scoped lesson view when the harness supplies the event and role. |
| SubagentStop | Role-specific checkpoint/dispatch, attributed lessons, and a compatibility fallback checkpoint. |
| Stop | Atomically enqueue a learning job and return; no model call or synchronous memory acknowledgement. |
| PreCompact / post-compact | Preserve a bounded event and re-anchor from the canonical revision. |
| TaskCompleted | Check consistency with a canonical KBD task's completion receipt; reject inconsistent completion without guarding mutation tools. |

Claude and Codex have explicit scoped-learning hook entries. Other harnesses use
supported lifecycle mappings and manual/skill paths; a declared adapter does not
prove the current harness exposes every native event. Read the selected installed
configuration and current session capabilities.

## Role context and learning

Claude's main thread receives the lead view and team digest. Codex's main thread
receives only local digest metadata because its children can inherit parent context.
Role-specific SubagentStart delivery is bounded and fenced as untrusted data.
Missing team, role or store results in a silent advisory path. See
[Memory tiers](memory-tiers.md) and [Agent Teams](24-agent-teams.md).

Direct learning wrappers and the generated dispatcher export
`PYTHONDONTWRITEBYTECODE=1` before child commands. The writer also suppresses local
bytecode and propagates the environment to descendants. Packaged entry points
matter: suppressing only one helper does not cover a new interpreter process.
These source changes require final generation and installed-path acceptance.

`memory-writeback.sh` extracts accepted reflection sections and routes them through
`learning_write.py`, with per-phase and per-scope deduplication. It can enqueue
memory operations, append the file tier and optionally start bounded pk/Cortex
work. Do not describe every hook as inference-free or network-free merely because
the Stop path is local. A queued write or started mirror is not confirmed storage.

## Completion and mutation boundaries

Bash, Python, Write, Edit and MultiEdit remain unrestricted. There is no Prometheus
PreToolUse mutation fence. A PostToolUse diagnostic cannot undo already-written
bytes, and a canonical TaskCompleted receipt check does not accept a missing task
or authorize another role's path.

Protected BDD integrity is evaluated from committed Git state at final local
certification. Intentional protected changes require the signed approval manifest.
Complete all phase production before tests, validators, compiler checks and review;
a declared legacy hook is not permission to run an earlier gate.

## Recovery

Stop and interrupt are different events. Stop never forces continuation. Honor an
explicit pause before scheduling work and preserve the queued checkpoint/receipt.
Reconcile uncertain writes with their stable operation identity and hash instead of
retry-count inference. Keep failed or unknown outcomes explicit.

Hooks, dispatchers, Python descendants and service workers have separate installed
boundaries. Verify the selected signed generation and actual target projection at
the final gate, then restart sessions whose hook configuration was loaded earlier.
See [Service operations](26-service-operations.md) for log and service ownership.

*Previous: [Rust Toolchain](14-rust-toolchain.md) · Next: [CLI & Scripts](16-cli-and-scripts.md)*
