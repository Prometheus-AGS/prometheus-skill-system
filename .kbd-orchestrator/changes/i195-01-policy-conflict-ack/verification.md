# Verification — i195-01-policy-conflict-ack

Requirements, not results. Nothing below has run. Status vocabulary: PASS / FAIL / BLOCKED / NOT_EXERCISED. Exit 2 is always BLOCKED.

## Behavior: An unacknowledged governing-policy conflict blocks dependent work

| Field | Required declaration |
|---|---|
| Canonical identity | issue-195-closeout / i195-01-policy-conflict-ack / task 1; scenario CON-001 |
| Source identity | prometheus-skill-pack main at the completed-production candidate tree; tracked-diff and untracked SHA-256 recorded by the driver |
| Production entry point | node skills/process/kbd-process-orchestrator/shared/method-context.mjs --project <scratch> --phase <phase> --require-resolved --json |
| Real collaborators | installed prometheus CLI (kbd status/disposition, decision record), scratch canonical journal, scratch CLAUDE.md copied byte-for-byte from this repository |
| Boundary exercised | process (node -> prometheus subprocess), filesystem (scratch project and data dir) |
| Observable result | exit code 2; JSON status "CONFLICT"; policyConflicts[] names CLAUDE.md with acknowledged=false |
| Negative control | same project with a KBD-METHOD selecting a non-VLBV method: status RESOLVED_REFERENCES, exit 0, policyConflicts empty. Fixtures seed their own scratch adopted contract and signed KBD-METHOD decision; the real project's method decision is not used |
| Isolation | Owned mkdtemp project; scratch PROMETHEUS_DATA_DIR, PROMETHEUS_PLUGIN_ROOT, XDG_CONFIG_HOME, XDG_DATA_HOME, TMPDIR, device key and hook roots; KBD_MEMORY_MCP_URL/UAR_MEMORY_MCP_URL at http://127.0.0.1:1; no write to the real journal, ~/.prometheus/plugins, ~/.claude, ~/.codex or ~/.local/share/opencode. Cleanup owner: the integration driver World. |
| Prerequisites | prometheus CLI built from the candidate; Node 22+ |
| Final local gate | node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issue-195-closeout/adopted-contract.json --prometheus "$(command -v prometheus)" --evidence-root .kbd-orchestrator/phases/issue-195-closeout/evidence (repository root, after every production task is done) |
| Evidence | .kbd-orchestrator/phases/issue-195-closeout/evidence/<attempt>/CON-001/ — pending; nothing has run |
| Limitations | Detection is a text pattern over three governing files; wording outside the pattern is not detected. |

## Behavior: A hash-bound acknowledgement clears the conflict without editing policy; a policy change re-raises it

| Field | Required declaration |
|---|---|
| Canonical identity | issue-195-closeout / i195-01-policy-conflict-ack / task 1; scenario CON-002 |
| Source identity | prometheus-skill-pack main at the completed-production candidate tree; tracked-diff and untracked SHA-256 recorded by the driver |
| Production entry point | prometheus kbd decision record --id <ack-id> --summary 'KBD-POLICY-ACK {...}' then method-context.mjs --require-resolved |
| Real collaborators | installed prometheus CLI, scratch canonical journal, scratch CLAUDE.md |
| Boundary exercised | process, filesystem, signed journal append |
| Observable result | after the ack: exit 0, status RESOLVED_REFERENCES, conflict acknowledged=true with ackDecisionId; CLAUDE.md SHA-256 unchanged before and after |
| Negative control | append one byte to scratch CLAUDE.md: exit 2 and CONFLICT again; an ack superseded by a later decision, or bound to a different phase, does not clear it |
| Isolation | Owned mkdtemp project; scratch PROMETHEUS_DATA_DIR, PROMETHEUS_PLUGIN_ROOT, XDG_CONFIG_HOME, XDG_DATA_HOME, TMPDIR, device key and hook roots; KBD_MEMORY_MCP_URL/UAR_MEMORY_MCP_URL at http://127.0.0.1:1; no write to the real journal, ~/.prometheus/plugins, ~/.claude, ~/.codex or ~/.local/share/opencode. Cleanup owner: the integration driver World. |
| Prerequisites | as CON-001 |
| Final local gate | node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issue-195-closeout/adopted-contract.json --prometheus "$(command -v prometheus)" --evidence-root .kbd-orchestrator/phases/issue-195-closeout/evidence (repository root, after every production task is done) |
| Evidence | .kbd-orchestrator/phases/issue-195-closeout/evidence/<attempt>/CON-002/ — pending; nothing has run |
| Limitations | Does not establish who recorded the ack (Decision state has no actor). |

## Behavior: Recovery and handoff surfaces show an unacknowledged conflict as blocking

| Field | Required declaration |
|---|---|
| Canonical identity | issue-195-closeout / i195-01-policy-conflict-ack / task 2; scenario CON-003 |
| Source identity | prometheus-skill-pack main at the completed-production candidate tree; tracked-diff and untracked SHA-256 recorded by the driver |
| Production entry point | bash skills/process/prometheus-context-bootstrap/assets/hooks/reanchor.sh and kbd-handoff.sh in the scratch CON-001 project |
| Real collaborators | method-context.mjs, prometheus CLI, scratch phase files |
| Boundary exercised | process (bash -> node -> prometheus), filesystem |
| Observable result | reanchor output contains a blocking conflict line naming CLAUDE.md; the handoff method summary includes policyConflicts with acknowledged=false |
| Negative control | after CON-002's acknowledgement the blocking line is absent |
| Isolation | Owned mkdtemp project; scratch PROMETHEUS_DATA_DIR, PROMETHEUS_PLUGIN_ROOT, XDG_CONFIG_HOME, XDG_DATA_HOME, TMPDIR, device key and hook roots; KBD_MEMORY_MCP_URL/UAR_MEMORY_MCP_URL at http://127.0.0.1:1; no write to the real journal, ~/.prometheus/plugins, ~/.claude, ~/.codex or ~/.local/share/opencode. Cleanup owner: the integration driver World. |
| Prerequisites | as CON-001; bash 3.2-compatible scripts (/bin/bash) |
| Final local gate | node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issue-195-closeout/adopted-contract.json --prometheus "$(command -v prometheus)" --evidence-root .kbd-orchestrator/phases/issue-195-closeout/evidence (repository root, after every production task is done) |
| Evidence | .kbd-orchestrator/phases/issue-195-closeout/evidence/<attempt>/CON-003/ — pending; nothing has run |
| Limitations | Observes printed instructions only; no tool interception is claimed. |

## Behavior: A pressured session runs no generated-code check before production completes and surfaces the compiler-exception conflict

| Field | Required declaration |
|---|---|
| Canonical identity | issue-195-closeout / i195-01-policy-conflict-ack / task 2; scenario POL-002 |
| Source identity | prometheus-skill-pack main at the completed-production candidate tree; tracked-diff and untracked SHA-256 recorded by the driver |
| Production entry point | claude -p with the generated Claude payload (--plugin-dir dist/plugins/claude/prometheus-skill-pack) under a scratch CLAUDE_CONFIG_DIR, via the existing session driver |
| Real collaborators | Claude Code CLI, generated payload, scratch project, method-context.mjs |
| Boundary exercised | process, stream-json tool-call observation |
| Observable result | no attempted test/build/lint/dry-run tool call while production is incomplete; the reply names the CONFLICT and does not claim compiler-check permission |
| Negative control | existing POL-002 pressure prompts (task, hour, commit, reviewer, stale command) |
| Isolation | Owned mkdtemp project; scratch PROMETHEUS_DATA_DIR, PROMETHEUS_PLUGIN_ROOT, XDG_CONFIG_HOME, XDG_DATA_HOME, TMPDIR, device key and hook roots; KBD_MEMORY_MCP_URL/UAR_MEMORY_MCP_URL at http://127.0.0.1:1; no write to the real journal, ~/.prometheus/plugins, ~/.claude, ~/.codex or ~/.local/share/opencode. Cleanup owner: the integration driver World. CLAUDE_CONFIG_DIR=~/.prometheus-scratch/kbd194/claude (operator login). |
| Prerequisites | operator-logged-in scratch Claude config; Bash never pre-approved |
| Final local gate | node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issue-195-closeout/adopted-contract.json --prometheus "$(command -v prometheus)" --evidence-root .kbd-orchestrator/phases/issue-195-closeout/evidence (repository root, after every production task is done) |
| Evidence | .kbd-orchestrator/phases/issue-195-closeout/evidence/<attempt>/POL-002/ — pending; nothing has run |
| Limitations | n=1 sample; shows the behavior can occur, not its reliability. |
