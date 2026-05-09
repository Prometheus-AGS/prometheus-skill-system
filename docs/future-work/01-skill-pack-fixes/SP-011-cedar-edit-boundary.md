---
id: SP-011
title: Cedar gate at PostToolUse for SKILL.md edits
status: ready
priority: P1
estimated_effort: 1d
agent_role: hooks-engineer
depends_on: []
unblocks: []
related: [SP-001, SP-016]
created_from_conversation_turn: 3-4
---

# SP-011 — Cedar gate at PostToolUse for SKILL.md edits

## Problem

`policies/skill-mutation.cedar` gates programmatic skill mutations (e.g. `skill.mutate` operations) at the PEP layer. But Claude's `Edit`, `Write`, and `MultiEdit` tools can directly modify a `SKILL.md` file without going through the PEP, bypassing Cedar entirely.

Concretely: a session can edit `skills/foo/SKILL.md` to change the skill's behavior with no policy check. The Cedar policy is a Maginot Line.

## Evidence

1. Read `policies/skill-mutation.cedar` — it gates a specific operation type.
2. Read `.claude-plugin/hooks/hooks.json` — note the PostToolUse matcher includes Edit/Write/MultiEdit but does not invoke Cedar.
3. Confirm: `grep -r 'skill-mutation' .claude-plugin/ hooks/ shared/scripts/` returns nothing operational.

## Why it matters

The whole point of having Cedar policies is that skill modifications go through review. Bypassing them means any agent (or any user with sufficient typo-fingers) can mutate a skill's behavior with no audit trail and no environment-specific gating.

In production environments, this is unacceptable. In dev it's a silent footgun.

## Proposed fix

Add a `PostToolUse` hook that runs after `Edit`/`Write`/`MultiEdit` operations targeting any `*/SKILL.md` path (or any `policies/*.cedar`, or any `hooks/*` file). The hook:

1. Captures the diff (the tool already provides this in its context).
2. Constructs a Cedar request: `principal: <session>, action: skill.mutate, resource: <skill-id>, context: {environment, file_path, diff_summary}`.
3. Runs Cedar evaluation against the policy set.
4. On *deny*: emits a warning to stderr, logs to `~/.prometheus/hooks.log` (per SP-006), and (in `prod` environment) reverts the change by writing the pre-edit content back. In `dev`, just warns.
5. On *allow*: logs the allowed mutation for audit.

The reversion in `prod` is the meaningful enforcement. In `dev`, the audit log is the value.

## Trade-offs and risks

- **Risk: revert produces inconsistent state.** If the editing tool's caller retries, you get a churn loop. Mitigation: revert is one-shot per session per file; subsequent edits in the same session bypass revert and just log. Loud warnings only.
- **Risk: Cedar engine adds latency to every Edit.** Mitigation: Cedar evaluation is microsecond-scale; not a real concern.
- **Risk: legitimate edits are blocked in prod.** Mitigation: Cedar policy itself defines what's allowed; the policy author is responsible for getting it right. The hook is the enforcement, not the policy.

## Acceptance criteria

- [ ] `PostToolUse` hook fires on Edit/Write/MultiEdit targeting `*/SKILL.md`, `policies/*.cedar`, and `hooks/*`.
- [ ] Cedar evaluation runs and logs the result.
- [ ] In `prod` environment, denied mutations are reverted; loud stderr warning.
- [ ] In `dev` environment, denied mutations are warned about but not reverted.
- [ ] All evaluations log to `~/.prometheus/hooks.log` per SP-006.
- [ ] Test: deny policy revert works in prod-mode test harness.

## Implementation steps

1. Add `shared/scripts/cedar-skill-mutation-gate.sh` invoked by PostToolUse hook.
2. Implement the Cedar request construction.
3. Wire to the Cedar runtime (likely a `cedar` CLI binary or a tiny Rust shim — check what's already in policies/).
4. Implement the revert path in prod-mode using the editing tool's pre-image when available, or via `git checkout HEAD~ -- <file>` as fallback (assumes the file was committed before the edit).
5. Test against synthetic deny/allow policies.
6. Document the policy authoring conventions in `policies/README.md`.

## Dependencies

None functional. Recommended after SP-006 so logging is visible.

## Open questions

- What's the right environment detector? Likely `PROMETHEUS_ENV=prod|dev|staging` env var or detection-by-hostname. Document the choice.
- Should the hook also gate `hooks.json` edits? Yes — included above.
- Does the revert via `git checkout` create commits or just modify the working tree? Working tree only; the revert is a "this edit didn't happen" signal.
