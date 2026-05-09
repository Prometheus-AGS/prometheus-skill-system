# Parallel Work Streams

How to allocate concurrent Claude Code sessions across the pack. This document operationalizes `00-meta/parallel-agent-routing.md` for actual execution.

## Concurrency limits per role

The roles have different file-modification footprints. The practical concurrency limits are:

| Role | Concurrent sessions before contention |
|------|---------------------------------------|
| `skill-pack-maintainer` | 1 (most tasks touch CLAUDE.md or hooks.json) |
| `hooks-engineer` | 2 (different scripts in `shared/scripts/`) |
| `rust-codegraph` | 2 (different crates or different parts of one crate) |
| `bdd-engineer` | 2 (different scripts or different features) |
| `docs-writer` | 2 (different markdown files) |

Total practical concurrency: **9 sessions**, though most teams won't run more than 3-4 simultaneously.

## Stream allocations by phase

### Phase 1 — Quick wins

If you have **5 sessions** to spend:

| Session | Role | Tasks |
|---------|------|-------|
| A | skill-pack-maintainer | SP-015 → SP-006 (sequential) |
| B | hooks-engineer | SP-013 |
| C | hooks-engineer | (waits for A on SP-006; meanwhile pulls SP-009 once A frees) |
| D | bdd-engineer | BDD-001 → BDD-002 (sequential) |
| E | docs-writer | BDD-006 |

### Phase 2 — Boundary conditions

| Session | Role | Tasks |
|---------|------|-------|
| A | rust-codegraph | SP-008 |
| B | bdd-engineer | BDD-005 |
| C | bdd-engineer | BDD-007 |
| D | skill-pack-maintainer | SP-016 → SP-001 |

### Phase 3 — Foundational architecture

| Session | Role | Tasks |
|---------|------|-------|
| A | rust-codegraph | BDD-008 (long-running, 1-2 weeks) |
| B | rust-codegraph | SP-019 (parallel; different crate) |
| C | hooks-engineer | SP-007 |
| (D-E unused; phase-3 work is heavy and benefits from focus) | | |

### Phase 4 — Selective execution payoff

| Session | Role | Tasks |
|---------|------|-------|
| A | rust-codegraph | BDD-009 |
| B | bdd-engineer | BDD-010 |
| C | bdd-engineer | BDD-011 → BDD-012 |
| D | rust-codegraph | SP-020 |

### Phase 5 — Loop closure

| Session | Role | Tasks |
|---------|------|-------|
| A | docs-writer | BDD-013 |
| B | bdd-engineer | BDD-014 |
| C | bdd-engineer | BDD-015 |
| D | hooks-engineer | SP-002 → SP-004 |
| E | rust-codegraph | SP-010 |

### Phase 6 — Operational hardening

Highly parallel; many small independent tasks.

| Session | Role | Tasks |
|---------|------|-------|
| A | hooks-engineer | SP-011 → SP-012 → SP-014 → SP-018 |
| B | hooks-engineer | SP-021 |
| C | skill-pack-maintainer | XC-004 → XC-005 → SP-017 |
| D | bdd-engineer | BDD-004 |
| E | docs-writer | XC-001 → XC-002 → XC-003 |
| F | rust-codegraph | SP-005 |

## Cross-stream coordination

When one stream depends on another's output, coordinate explicitly:

- A waits for B's `done` status in STATUS.md before starting.
- A's first action when picking up the task is to confirm B's work is in fact merged on `main` (not just done in a branch).
- A documents in its session-end notes how B's output was consumed.

## Stream recovery

If a session is interrupted (context exhaustion, process crash, agent gives up):

1. The task remains `in-progress` in STATUS.md.
2. A new session checks STATUS.md, sees the task is `in-progress` with `assigned_to: <previous-session>`.
3. The new session decides: resume or restart.
4. **Resume**: read the task doc, the partial branch, and any commits the previous session made. Continue from there. Update `assigned_to` to current session.
5. **Restart**: discard the partial branch (`git branch -D future-work/<task-id>`), reset the task to `ready` in STATUS.md, then pick it up fresh.

The decision is judgment. If the previous session wrote substantive code that compiles and is partially tested, resume. If the partial work is incomplete enough that re-doing is faster, restart.

## Anti-patterns

- **Long-running tasks left running across many sessions of intermittent work.** A 1-2 week task (e.g. BDD-008) should have *one primary owner*. Hand-offs are expensive; preserve continuity where possible.
- **Multiple tasks pulled into a single session.** Already prohibited by `00-meta/execution-protocol.md`. Repeated here because it's the most common drift in practice.
- **Promotion of `planned` to `ready` without verifying dependencies.** The `unblocks` and `depends_on` lists in STATUS.md are the truth source. If a session starts a `planned` task because "it looks ready," it may discover mid-work that a dependency wasn't actually satisfied.

## When to slow down

If three sessions in the same role start contending for the same files repeatedly, slow down. Two concurrent sessions in `bdd-engineer` is fine; three is fragile. The contention shows up as merge conflicts during PR review. When it does, the right response is to serialize, not to add more sessions.

## When to speed up

If multiple sessions are completing tasks faster than reviews can keep up, the bottleneck is review, not generation. Add reviewer capacity (or merge less aggressively) before adding more agent sessions.
