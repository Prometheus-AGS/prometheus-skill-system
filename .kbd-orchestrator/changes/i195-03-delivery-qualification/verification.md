# Verification — i195-03-delivery-qualification

Requirements, not results. Nothing below has run. Status vocabulary: PASS / FAIL / BLOCKED / NOT_EXERCISED. Exit 2 is always BLOCKED.

## Behavior: value-scenario-verify drives the real repair-and-resume operation and its oracle is sensitive

| Field | Required declaration |
|---|---|
| Canonical identity | issue-195-closeout / i195-03-delivery-qualification / task 1; scenario VER-001 |
| Source identity | prometheus-skill-pack main at the completed-production candidate tree; tracked-diff and untracked SHA-256 recorded by the driver |
| Production entry point | /value-scenario-verify invoked by a scratch Claude session against the IDR-002/IDR-003 repair-and-resume operation as runner |
| Real collaborators | Claude Code CLI with generated payload, installed prometheus CLI (change repair-identity, kbd-apply begin/end-task), scratch journal |
| Boundary exercised | process, signed journal, stream-json observation |
| Observable result | verdict PASS with the observed resumed task state; the session reports authored, measured and demonstrated value separately |
| Negative control | a deliberately wrong expected value yields FAIL; a removed required assertion yields NOT_EXERCISED, never PASS |
| Isolation | Owned mkdtemp project; scratch PROMETHEUS_DATA_DIR, PROMETHEUS_PLUGIN_ROOT, XDG_CONFIG_HOME, XDG_DATA_HOME, TMPDIR, device key and hook roots; KBD_MEMORY_MCP_URL/UAR_MEMORY_MCP_URL at http://127.0.0.1:1; no write to the real journal, ~/.prometheus/plugins, ~/.claude, ~/.codex or ~/.local/share/opencode. Cleanup owner: the integration driver World. CLAUDE_CONFIG_DIR=~/.prometheus-scratch/kbd194/claude (operator login). |
| Prerequisites | operator-logged-in scratch Claude config; value-scenario-verify in the generated payload |
| Final local gate | node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issue-195-closeout/adopted-contract.json --prometheus "$(command -v prometheus)" --evidence-root .kbd-orchestrator/phases/issue-195-closeout/evidence (repository root, after every production task is done) |
| Evidence | .kbd-orchestrator/phases/issue-195-closeout/evidence/<attempt>/VER-001/ — pending; nothing has run |
| Limitations | n=1 sample. |

## Behavior: Canonical closure keeps six claims distinct and starts no successor

| Field | Required declaration |
|---|---|
| Canonical identity | issue-195-closeout / i195-03-delivery-qualification / task 1; scenario VER-003 |
| Source identity | prometheus-skill-pack main at the completed-production candidate tree; tracked-diff and untracked SHA-256 recorded by the driver |
| Production entry point | scratch session drives the canonical phase closure writer through value-scenario-verify (closure adapter fixture/step/disposition) |
| Real collaborators | Claude Code CLI, installed prometheus CLI, scratch journal |
| Boundary exercised | process, signed journal |
| Observable result | typed disposition persists; authored, measured, demonstrated, gaps, release eligibility and release authority reported separately; eventCounts show no new phase, task or publication events |
| Negative control | a request to "also start the next phase" produces no phase/task events |
| Isolation | Owned mkdtemp project; scratch PROMETHEUS_DATA_DIR, PROMETHEUS_PLUGIN_ROOT, XDG_CONFIG_HOME, XDG_DATA_HOME, TMPDIR, device key and hook roots; KBD_MEMORY_MCP_URL/UAR_MEMORY_MCP_URL at http://127.0.0.1:1; no write to the real journal, ~/.prometheus/plugins, ~/.claude, ~/.codex or ~/.local/share/opencode. Cleanup owner: the integration driver World. CLAUDE_CONFIG_DIR=~/.prometheus-scratch/kbd194/claude (operator login). |
| Prerequisites | operator-logged-in scratch Claude config |
| Final local gate | node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issue-195-closeout/adopted-contract.json --prometheus "$(command -v prometheus)" --evidence-root .kbd-orchestrator/phases/issue-195-closeout/evidence (repository root, after every production task is done) |
| Evidence | .kbd-orchestrator/phases/issue-195-closeout/evidence/<attempt>/VER-003/ — pending; nothing has run |
| Limitations | n=1 sample. |

## Behavior: A Codex session recovers the selected method and reports the conflict

| Field | Required declaration |
|---|---|
| Canonical identity | issue-195-closeout / i195-03-delivery-qualification / task 1; scenario SRF-001 |
| Source identity | prometheus-skill-pack main at the completed-production candidate tree; tracked-diff and untracked SHA-256 recorded by the driver |
| Production entry point | codex exec --json -s read-only --ephemeral with the installed Codex payload under scratch CODEX_HOME and PROMETHEUS_PLUGIN_ROOT |
| Real collaborators | codex-cli, installed Codex payload, method-context.mjs, scratch project |
| Boundary exercised | process, JSONL event observation |
| Observable result | reply names the selected method, contract digest and the CONFLICT; no attempted generated-code check |
| Negative control | sensitivity: a second scratch project whose signed KBD-METHOD selects a different method and digest must be reported as that method and digest, not the expected one. Not logged in under scratch CODEX_HOME: NOT_EXERCISED, never run against ~/.codex |
| Isolation | Owned mkdtemp project; scratch PROMETHEUS_DATA_DIR, PROMETHEUS_PLUGIN_ROOT, XDG_CONFIG_HOME, XDG_DATA_HOME, TMPDIR, device key and hook roots; KBD_MEMORY_MCP_URL/UAR_MEMORY_MCP_URL at http://127.0.0.1:1; no write to the real journal, ~/.prometheus/plugins, ~/.claude, ~/.codex or ~/.local/share/opencode. Cleanup owner: the integration driver World. CODEX_HOME=~/.prometheus-scratch/kbd194/codex (operator login). |
| Prerequisites | operator-logged-in scratch Codex home (optional) |
| Final local gate | node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issue-195-closeout/adopted-contract.json --prometheus "$(command -v prometheus)" --evidence-root .kbd-orchestrator/phases/issue-195-closeout/evidence (repository root, after every production task is done) |
| Evidence | .kbd-orchestrator/phases/issue-195-closeout/evidence/<attempt>/SRF-001/ — pending; nothing has run |
| Limitations | n=1; NOT_EXERCISED is an allowed outcome. |

## Behavior: An OpenCode session under verified isolation recovers the selected method and reports the conflict

| Field | Required declaration |
|---|---|
| Canonical identity | issue-195-closeout / i195-03-delivery-qualification / task 1; scenario SRF-002 |
| Source identity | prometheus-skill-pack main at the completed-production candidate tree; tracked-diff and untracked SHA-256 recorded by the driver |
| Production entry point | opencode run --format json --standalone with scratch XDG roots and OPENCODE_DISABLE_CLAUDE_CODE=1 |
| Real collaborators | opencode 2.2.0, pack skills made available inside the scratch config root, method-context.mjs |
| Boundary exercised | process, JSON event observation |
| Observable result | isolation preflight passes; reply names the selected method and the CONFLICT |
| Negative control | sensitivity: a second scratch project with a different signed method and digest must be reported as such. Isolation preflight shows config or auth resolving outside the scratch roots: NOT_EXERCISED (unsupported isolation), no session run |
| Isolation | Owned mkdtemp project; scratch PROMETHEUS_DATA_DIR, PROMETHEUS_PLUGIN_ROOT, XDG_CONFIG_HOME, XDG_DATA_HOME, TMPDIR, device key and hook roots; KBD_MEMORY_MCP_URL/UAR_MEMORY_MCP_URL at http://127.0.0.1:1; no write to the real journal, ~/.prometheus/plugins, ~/.claude, ~/.codex or ~/.local/share/opencode. Cleanup owner: the integration driver World. XDG_CONFIG_HOME/XDG_DATA_HOME under ~/.prometheus-scratch/kbd194/opencode; OPENCODE_DISABLE_CLAUDE_CODE=1; OPENCODE_DISABLE_AUTOUPDATE=1. |
| Prerequisites | operator login inside the scratch OpenCode data root (optional) |
| Final local gate | node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issue-195-closeout/adopted-contract.json --prometheus "$(command -v prometheus)" --evidence-root .kbd-orchestrator/phases/issue-195-closeout/evidence (repository root, after every production task is done) |
| Evidence | .kbd-orchestrator/phases/issue-195-closeout/evidence/<attempt>/SRF-002/ — pending; nothing has run |
| Limitations | XDG honoring on macOS is UNVERIFIED until the preflight runs; n=1. |
