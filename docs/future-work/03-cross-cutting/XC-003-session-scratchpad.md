---
id: XC-003
title: Per-session SCRATCHPAD.md pattern
status: ready
priority: P2
estimated_effort: 0.5d
agent_role: docs-writer
depends_on: []
unblocks: []
related: []
created_from_conversation_turn: 1-2
---

# XC-003 — Per-session SCRATCHPAD.md pattern

## Problem

Long Claude Code sessions accumulate working state — half-completed plans, intermediate findings, "remember to do X" reminders — that has nowhere to live. The agent's context window holds them but eventually rolls them out. Once they're out, recovery requires re-reading the session transcript, which is expensive.

Sessions that span multiple agent runs (e.g. resuming after Claude Code restarts) lose this state entirely.

## Evidence

Observe a long session. Notice when the agent forgets earlier decisions. Notice when it would benefit from "I already considered approach X and rejected it" without having to re-derive.

## Why it matters

This is the Karpathy "Write" operation in operational form: short-term, session-scoped writes that the agent can re-read. Cheaper than full episodic memory; more durable than context.

## Proposed fix

A simple convention: every Claude Code session has a `.prometheus/scratchpad.md` in the project root that the agent maintains. The file is gitignored. Sections:

```markdown
# Session scratchpad — <session-id>

## Decisions made
- 2026-05-09 10:34 — Chose pk-codegraph in Rust workspace, Node extractor; rejected pure-Rust extraction because of TS ecosystem.

## Open questions
- How to handle dynamic testid construction in BDD-005?

## Plans in progress
- [ ] BDD-008 step 3: ts-morph extractor.
- [x] BDD-008 step 1: Crate scaffold.

## Findings
- `prometheus-knowledge`'s schema doesn't have an `event` table yet.

## Reminders
- After SP-013, verify the SubagentStop matcher fires on the reflector.
```

The agent appends to it as the session progresses. At session start, the agent reads it (if it exists) to recover state.

## Trade-offs and risks

- **Risk: scratchpad becomes a parallel ledger that drifts from the truth.** Mitigation: it's gitignored, ephemeral, scoped per-session. When the session ends, it's discarded (or archived to `.prometheus/scratchpad-archive/` if useful).
- **Risk: secrets or sensitive context leak into scratchpad.** Mitigation: same redaction convention as the trace capture (per SP-007).
- **Cost: agent must remember to update it.** Mitigation: include "did you update the scratchpad?" as a soft-prompt in the SubagentStop hook.

## Acceptance criteria

- [ ] `.prometheus/scratchpad.md` convention documented in skill-pack `CLAUDE.md`.
- [ ] `.prometheus/` is in default project `.gitignore` template (per SP-008).
- [ ] Skill-pack ships a `scratchpad-template.md` that agents copy on session start if no scratchpad exists.
- [ ] At session end, scratchpad either discarded or archived to `.prometheus/scratchpad-archive/<timestamp>.md` based on a `--archive-scratchpad` flag on session-end.
- [ ] Documentation in CLAUDE.md tells the agent: read scratchpad on start, update during session.

## Implementation steps

1. Author the template.
2. Add the convention to skill-pack `CLAUDE.md`.
3. Update `.gitignore` template.
4. Add session-start logic (per SP-006 hook log shim, fire scratchpad-init on SessionStart).
5. Add session-end logic (archive option).
6. Test by running a session and verifying scratchpad behavior.

## Dependencies

None functional.

## Open questions

- Should the scratchpad be auto-summarized at session end and the summary fed into surreal-memory? Possibly — promotes scratchpad notes to episodic memory. Out of scope here; track separately.
- Should multiple parallel sessions in the same project share a scratchpad or have separate ones? Separate, per session ID, to avoid cross-session context bleed.
