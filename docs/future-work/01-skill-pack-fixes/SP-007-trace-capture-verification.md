---
id: SP-007
title: Trace capture file existence verification + implementation
status: ready
priority: P1
estimated_effort: 2d
agent_role: hooks-engineer
depends_on: []
unblocks: [SP-019]
related: [SP-006]
created_from_conversation_turn: 3-4
---

# SP-007 — Trace capture file existence verification

## Problem

The self-learning architecture document (`docs/plans/2026-04-29-change-006-karpathy-loop-hooks.md`) describes trace capture writing to `.prometheus/traces/`. Reading the actual hook layer, no script writes there. Either:

- The documentation describes a planned-but-unbuilt feature, or
- The capture happens elsewhere (e.g. inside the librarian process) and the path is misdocumented.

Either way, the discrepancy means agents that consult the architecture doc will form incorrect mental models.

## Evidence

1. Read `docs/plans/2026-04-29-change-006-karpathy-loop-hooks.md` — note the `.prometheus/traces/` references.
2. Run a session, then `find . -path '*.prometheus/traces*'` from the project root and from `$HOME`. Note whether anything was written.
3. Grep `shared/scripts/*.sh` for `traces` or `trace`. Count writes.

## Why it matters

The trace capture is foundational for SP-019 (LibrarianEvent persistence): if events aren't being captured at all, persisting them is moot. Verify the actual state before designing the persistence layer.

A secondary effect: agents that read the architecture doc and find references to traces will assume they exist and may try to consume them. They won't find them and will work around the absence in non-systematic ways.

## Proposed fix

Two-phase task. Phase 1 verifies state. Phase 2 implements or documents the truth.

**Phase 1: Verification.**

Run the find command. Inspect the librarian process for in-process trace writes. Confirm whether traces are written *anywhere*, and if so, where.

Document findings in this task's STATUS.md `notes:` field. Then choose phase 2A or 2B based on outcome.

**Phase 2A: If traces are written but to a different location.**

Update the architecture doc to point at the actual location. Add a small abstraction (`PROMETHEUS_TRACE_DIR` env var) so the location is configurable.

**Phase 2B: If traces are not written anywhere.**

Implement trace capture as a small `shared/scripts/lib/trace-capture.sh` that hooks SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, and Stop. Each event appends to `${PROMETHEUS_TRACE_DIR:-.prometheus/traces}/<session-id>/<seq>.jsonl`.

The trace events are: timestamp, hook name, tool name (if applicable), tool args (redacted), tool result (redacted size only).

## Trade-offs and risks

- **Volume.** A trace file per session per event grows fast in long sessions. Mitigation: rotate by session, gzip on session end (Stop hook), keep 7 days locally.
- **Sensitive content in tool args.** Redact aggressively. Strip anything that matches common-secret patterns (passwords, tokens, API keys). Reuse `feedback-project-adapter`'s redaction patterns.
- **Coupling to SP-006.** This task's logging discipline is the same as SP-006's hook log. Consider unifying: hook log records events; trace capture is a richer subset for selected hooks. They share the JSONL format and the rotation policy.

## Acceptance criteria

- [ ] STATUS.md notes for this task document the verification finding (where, if anywhere, traces are currently written).
- [ ] Either: (a) architecture doc updated to point at the real location and `PROMETHEUS_TRACE_DIR` env var added; or (b) trace capture implemented at the documented location.
- [ ] A test session produces a non-empty trace directory at the documented location.
- [ ] Sensitive content in trace events is redacted (test with a synthetic prompt containing a fake API key).

## Implementation steps

1. Verify (Phase 1): grep, find, and read enough of the codebase to know whether traces exist.
2. Decide which phase 2 path applies.
3. Implement (Phase 2A or 2B).
4. Add a test that produces traces and validates redaction.
5. Update the architecture doc.

## Dependencies

None — this is a verification-and-act task.

## Open questions

- Should traces feed directly into surreal-memory as `LibrarianEvent` records (per SP-019), or remain on-disk JSONL with the librarian ingesting later? Recommend on-disk first, librarian-ingested later. Decouples capture from persistence.
- What's the redaction list? Reuse SSR feedback-project-adapter's patterns plus any extras specific to skill-pack-internal traces.
