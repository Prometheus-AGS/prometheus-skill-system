# Verification — i195-02-single-method-reference

Requirements, not results. Nothing below has run. Status vocabulary: PASS / FAIL / BLOCKED / NOT_EXERCISED. Exit 2 is always BLOCKED.

## Behavior: All stage skills carry the identical core in source, generated payloads and an installer-dispatched scratch generation

| Field | Required declaration |
|---|---|
| Canonical identity | issue-195-closeout / i195-02-single-method-reference / task 2; scenario REF-001 |
| Source identity | prometheus-skill-pack main at the completed-production candidate tree; tracked-diff and untracked SHA-256 recorded by the driver |
| Production entry point | node scripts/check-selected-method-core.mjs; then, from the integration driver without any CLI session, node scripts/install-plugin-generation.js --source-root <candidate> --home <scratch> --plugin-root <scratch>/plugins after the World has created the owned scratch roots |
| Real collaborators | distribution generator, installer generation writer, scratch plugin root |
| Boundary exercised | filesystem (source, dist/, scratch installed generation) |
| Observable result | the section bytes are identical across the 11 source skills, both generated payloads and the scratch-installed generation; check exits 0 |
| Negative control | a scratch copy with one stage section altered by one character makes the check exit non-zero naming that file |
| Isolation | Owned mkdtemp project; scratch PROMETHEUS_DATA_DIR, PROMETHEUS_PLUGIN_ROOT, XDG_CONFIG_HOME, XDG_DATA_HOME, TMPDIR, device key and hook roots; KBD_MEMORY_MCP_URL/UAR_MEMORY_MCP_URL at http://127.0.0.1:1; no write to the real journal, ~/.prometheus/plugins, ~/.claude, ~/.codex or ~/.local/share/opencode. Cleanup owner: the integration driver World. |
| Prerequisites | completed production; generator outputs regenerated |
| Final local gate | node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issue-195-closeout/adopted-contract.json --prometheus "$(command -v prometheus)" --evidence-root .kbd-orchestrator/phases/issue-195-closeout/evidence (repository root, after every production task is done) |
| Evidence | .kbd-orchestrator/phases/issue-195-closeout/evidence/<attempt>/REF-001/ — pending; nothing has run |
| Limitations | Proves byte identity, not that a model reads the section. |

## Behavior: Fresh, resumed and delegated sessions recover the same method, contract, value and stage authority from the rewritten core (rerun)

| Field | Required declaration |
|---|---|
| Canonical identity | issue-195-closeout / i195-02-single-method-reference / task 1; scenario POL-001 |
| Source identity | prometheus-skill-pack main at the completed-production candidate tree; tracked-diff and untracked SHA-256 recorded by the driver |
| Production entry point | existing POL-001 session implementation in scripts/tests/lib/kbd-194-196-sessions.mjs with the regenerated Claude payload |
| Real collaborators | Claude Code CLI, generated payload, method-context.mjs, scratch project |
| Boundary exercised | process, stream-json observation |
| Observable result | all observations report identical method, contract digest, included value/exclusions and stage authority |
| Negative control | existing POL-001 negative cases |
| Isolation | Owned mkdtemp project; scratch PROMETHEUS_DATA_DIR, PROMETHEUS_PLUGIN_ROOT, XDG_CONFIG_HOME, XDG_DATA_HOME, TMPDIR, device key and hook roots; KBD_MEMORY_MCP_URL/UAR_MEMORY_MCP_URL at http://127.0.0.1:1; no write to the real journal, ~/.prometheus/plugins, ~/.claude, ~/.codex or ~/.local/share/opencode. Cleanup owner: the integration driver World. CLAUDE_CONFIG_DIR=~/.prometheus-scratch/kbd194/claude (operator login). |
| Prerequisites | operator-logged-in scratch Claude config |
| Final local gate | node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issue-195-closeout/adopted-contract.json --prometheus "$(command -v prometheus)" --evidence-root .kbd-orchestrator/phases/issue-195-closeout/evidence (repository root, after every production task is done) |
| Evidence | .kbd-orchestrator/phases/issue-195-closeout/evidence/<attempt>/POL-001/ — pending; nothing has run |
| Limitations | n=1 sample. |

## Behavior: Resume after supersession keeps budgets and uncertain effects with the rewritten core (rerun)

| Field | Required declaration |
|---|---|
| Canonical identity | issue-195-closeout / i195-02-single-method-reference / task 1; scenario POL-003 |
| Source identity | prometheus-skill-pack main at the completed-production candidate tree; tracked-diff and untracked SHA-256 recorded by the driver |
| Production entry point | existing POL-003 session implementation in scripts/tests/lib/kbd-194-196-sessions.mjs with the regenerated Claude payload |
| Real collaborators | Claude Code CLI, generated payload, installed prometheus CLI, scratch journal |
| Boundary exercised | process, signed journal, stream-json observation |
| Observable result | latest signed selection wins; retired workflow not resurrected; consumed budget retained; uncertain effect not retried |
| Negative control | existing POL-003 stale handoff |
| Isolation | Owned mkdtemp project; scratch PROMETHEUS_DATA_DIR, PROMETHEUS_PLUGIN_ROOT, XDG_CONFIG_HOME, XDG_DATA_HOME, TMPDIR, device key and hook roots; KBD_MEMORY_MCP_URL/UAR_MEMORY_MCP_URL at http://127.0.0.1:1; no write to the real journal, ~/.prometheus/plugins, ~/.claude, ~/.codex or ~/.local/share/opencode. Cleanup owner: the integration driver World. CLAUDE_CONFIG_DIR=~/.prometheus-scratch/kbd194/claude (operator login). |
| Prerequisites | operator-logged-in scratch Claude config |
| Final local gate | node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issue-195-closeout/adopted-contract.json --prometheus "$(command -v prometheus)" --evidence-root .kbd-orchestrator/phases/issue-195-closeout/evidence (repository root, after every production task is done) |
| Evidence | .kbd-orchestrator/phases/issue-195-closeout/evidence/<attempt>/POL-003/ — pending; nothing has run |
| Limitations | n=1 sample. |

## Behavior: Finite source/generated/scratch-installed parity (rerun of the prior phase scenario)

| Field | Required declaration |
|---|---|
| Canonical identity | issue-195-closeout / i195-02-single-method-reference / task 2; scenario PKG-001 |
| Source identity | prometheus-skill-pack main at the completed-production candidate tree; tracked-diff and untracked SHA-256 recorded by the driver |
| Production entry point | existing PKG-001 implementation in scripts/tests/lib/kbd-194-196-package.mjs |
| Real collaborators | generator (two runs), installer dispatch, scratch plugin root |
| Boundary exercised | filesystem, installer process |
| Observable result | two generator runs byte-identical; actual installer dispatch used; method-context --write works through a symlinked skill dir |
| Negative control | existing PKG-001 negative cases |
| Isolation | Owned mkdtemp project; scratch PROMETHEUS_DATA_DIR, PROMETHEUS_PLUGIN_ROOT, XDG_CONFIG_HOME, XDG_DATA_HOME, TMPDIR, device key and hook roots; KBD_MEMORY_MCP_URL/UAR_MEMORY_MCP_URL at http://127.0.0.1:1; no write to the real journal, ~/.prometheus/plugins, ~/.claude, ~/.codex or ~/.local/share/opencode. Cleanup owner: the integration driver World. |
| Prerequisites | completed production |
| Final local gate | node scripts/tests/kbd-194-196.integration.mjs --contract .kbd-orchestrator/phases/issue-195-closeout/adopted-contract.json --prometheus "$(command -v prometheus)" --evidence-root .kbd-orchestrator/phases/issue-195-closeout/evidence (repository root, after every production task is done) |
| Evidence | .kbd-orchestrator/phases/issue-195-closeout/evidence/<attempt>/PKG-001/ — pending; nothing has run |
| Limitations | macOS only. |
