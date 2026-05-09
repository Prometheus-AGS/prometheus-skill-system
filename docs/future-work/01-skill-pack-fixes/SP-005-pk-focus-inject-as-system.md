---
id: SP-005
title: pk focus --inject-as system-context flag
status: planned
priority: P2
estimated_effort: 0.5d
agent_role: rust-codegraph
depends_on: [SP-004]
unblocks: []
related: []
created_from_conversation_turn: 3-4
---

# SP-005 — pk focus --inject-as system-context flag

## Problem

`pk focus` currently emits its retrieval as a user-visible response. This is helpful for debugging but means the retrieval consumes user-facing tokens, conflicts with assistant output for terminal real estate, and signals to the user that "the librarian fetched stuff" which is operational noise rather than substantive content.

## Evidence

Run `pk focus` on a sample query. Observe that the output goes to stdout and renders as part of the assistant turn.

## Why it matters

For the librarian to truly act as a context-augmentation layer, its output should arrive *as system context* — not visible to the user, but available to the model. Claude Code supports system-context injection via specific hook output protocols (the `additionalContext` field in the hook response JSON).

## Proposed fix

Add a `--inject-as` flag to the `pk` CLI (a `pk-cli` crate change). Two valid values:

- `user-visible` (default; current behavior).
- `system-context` (emit a JSON object with the hook protocol's `additionalContext` shape, suppress stdout otherwise).

Update `shared/scripts/pk-focus-on-prompt.sh` to pass `--inject-as system-context` when running inside a `UserPromptSubmit` hook, since that's the natural injection point.

## Trade-offs and risks

- **Risk: silently injecting content the user can't see** can be surprising when the agent acts on it. Mitigation: log the injection to `~/.prometheus/hooks.log` (per SP-006) so the user can audit after the fact.
- **Risk: hook protocol changes** if Claude Code revises its system-context API. Mitigation: keep the JSON-shaping in one function; revise that one if the API changes.

## Acceptance criteria

- [ ] `pk focus --inject-as system-context "..." ` produces the correct JSON shape and no other stdout.
- [ ] `pk focus --inject-as user-visible "..." ` (and the no-flag default) produces the human-readable output unchanged.
- [ ] `pk-focus-on-prompt.sh` passes the flag when invoked inside a hook.
- [ ] An end-to-end smoke test: a UserPromptSubmit hook firing `pk focus --inject-as system-context` results in a follow-up assistant turn that references the retrieved knowledge without the user having seen the retrieval.

## Implementation steps

1. Modify `pk-cli/src/commands/focus.rs` to parse `--inject-as`.
2. Add a `system-context` rendering path that emits the hook-protocol JSON.
3. Update `pk-focus-on-prompt.sh` to use the flag when `${CLAUDE_HOOK_INVOCATION:-}` is non-empty.
4. Add an integration test in `pk-cli/tests/`.

## Dependencies

SP-004 (the extractor produces richer output that benefits from system-context injection).

## Open questions

- What's the exact JSON shape Claude Code expects for `additionalContext`? Verify against the Claude Code documentation; the protocol may have evolved.
- Should this flag also apply to `pk ingest` and other librarian commands? Out of scope here; track separately if useful.
