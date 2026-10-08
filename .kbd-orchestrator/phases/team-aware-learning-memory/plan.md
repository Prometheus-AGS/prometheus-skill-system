PLAN: team-aware-learning-memory
Project: prometheus-skill-pack
Date: 2026-10-04
OpenSpec available: YES (openspec/ exists) — this phase pins `specBackend: native-kbd` in `.kbd-orchestrator/project.json`, matching every native change in this repository's recent phases
Changes to implement: 4

CHANGE LIST (ordered)
1. change-tlm-004-hook-timeout-seconds: correct hook timeout units to seconds for both harnesses and remove the PR #121 millisecond floor
   - Scope: hook contract, generators, generated hook/dist outputs, CLAUDE.md, docs/codex-plugin.md
   - Depends on: NONE
   - Recommended agent: claude-code
   - Est. complexity: S
   - Complexity score: Low
   - Model class: medium
   - Customer value: MEDIUM (stops 17-minute to 10-hour hook timeouts; removes wrong guidance this session introduced)
   - Details: Convert contract timeouts from milliseconds to seconds, add a 600-second generator ceiling, delete `CODEX_MIN_HOOK_TIMEOUT_MS`, correct the docs, regenerate (C-01/C-03/C-04).
2. change-tlm-001-subagent-identity-probe: confirm per-subagent identity and context injection at runtime in Claude Code and Codex
   - Scope: one probe script + evidence file
   - Depends on: NONE
   - Recommended agent: claude-code
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH (every later delivery decision depends on these verdicts)
   - Details: A scratch project with a project-local `tlm-probe-role` agent and SubagentStart/PreToolUse/SubagentStop hooks injecting a nonce. Runs `claude -p` and `codex exec`. Records CONFIRMED/REFUTED/UNVERIFIABLE for six behaviours, keyed per harness. Isolation: Claude `--setting-sources project,local --strict-mcp-config --no-session-persistence`; Codex `--ephemeral --ignore-user-config`, with scratch HOME. Live state is re-hashed against pre-probe hashes.
3. change-tlm-002-team-aware-learning-design: the design document and learning-envelope schema
   - Scope: docs/design + shared/schemas
   - Depends on: change-tlm-001-subagent-identity-probe
   - Recommended agent: claude-code
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Ten sections: identity, envelope, visibility/promotion, per-agent recall and budgets, delivery (file tiers in both harnesses), attributed writes, cross-agent awareness, upstream prerequisites, mini scope, measurement. Plus a JSON Schema with valid and invalid examples.
4. change-tlm-003-revised-implementation-plan: replace parent-plan PRs 4–14 with a role-aware PR sequence and per-agent integration gates
   - Scope: docs/plans + session plan pointer
   - Depends on: change-tlm-002-team-aware-learning-design, change-tlm-004-hook-timeout-seconds
   - Recommended agent: claude-code
   - Est. complexity: M
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH
   - Details: Upstream pk/surreal-memory fixes and pins first, then identity, envelope, recall, delivery, file-tier reduction, awareness, promotion, gaps, discovery, teams and mini. Each PR has an integration gate; end-to-end two-role gates run in both harnesses.

TASK MODEL ASSIGNMENTS
| Phase path | Change ID | Backend task ID | Requirements | Provider/model | Reasoning effort | Rationale and dated evidence | Harness and route | Worker launch and handoff | Native alternative | Availability and verification | Prerequisites |
|---|---|---|---|---|---|---|---|---|---|---|---|
| team-aware-learning-memory | change-tlm-004-hook-timeout-seconds | 1 | low uncertainty; mechanical contract edit + generator guard + regeneration; repo tools only | anthropic/claude-opus-5-5 | medium | mechanical contract and generator fix; medium class suffices; run in-session on the current model because no other model route is verified here (observed 2026-10-04) | Claude Code 2.1.289, native | current agent executes the task in this session; scope = change spec Scope; cwd = repo root or scratch dir; result = files in Scope + execution.md evidence | same | configured and exercised in this session | none |
| team-aware-learning-memory | change-tlm-004-hook-timeout-seconds | 2 | low uncertainty; mechanical contract edit + generator guard + regeneration; repo tools only | anthropic/claude-opus-5-5 | medium | mechanical contract and generator fix; medium class suffices; run in-session on the current model because no other model route is verified here (observed 2026-10-04) | Claude Code 2.1.289, native | current agent executes the task in this session; scope = change spec Scope; cwd = repo root or scratch dir; result = files in Scope + execution.md evidence | same | configured and exercised in this session | none |
| team-aware-learning-memory | change-tlm-004-hook-timeout-seconds | 3 | low uncertainty; mechanical contract edit + generator guard + regeneration; repo tools only | anthropic/claude-opus-5-5 | medium | mechanical contract and generator fix; medium class suffices; run in-session on the current model because no other model route is verified here (observed 2026-10-04) | Claude Code 2.1.289, native | current agent executes the task in this session; scope = change spec Scope; cwd = repo root or scratch dir; result = files in Scope + execution.md evidence | same | configured and exercised in this session | none |
| team-aware-learning-memory | change-tlm-001-subagent-identity-probe | 1 | medium uncertainty; runs claude -p and codex exec in a scratch project; needs authenticated harness CLIs; independent of other changes | anthropic/claude-opus-5-5 | medium | scripted probe and evidence capture; medium class suffices; current model used because it is the only verified route and it can drive claude/codex CLIs in this session (2026-10-04) | Claude Code 2.1.289, native | current agent executes the task in this session; scope = change spec Scope; cwd = repo root or scratch dir; result = files in Scope + execution.md evidence | same | configured and exercised in this session | authenticated claude and codex CLIs (present) |
| team-aware-learning-memory | change-tlm-001-subagent-identity-probe | 2 | medium uncertainty; runs claude -p and codex exec in a scratch project; needs authenticated harness CLIs; independent of other changes | anthropic/claude-opus-5-5 | medium | scripted probe and evidence capture; medium class suffices; current model used because it is the only verified route and it can drive claude/codex CLIs in this session (2026-10-04) | Claude Code 2.1.289, native | current agent executes the task in this session; scope = change spec Scope; cwd = repo root or scratch dir; result = files in Scope + execution.md evidence | same | configured and exercised in this session | authenticated claude and codex CLIs (present) |
| team-aware-learning-memory | change-tlm-001-subagent-identity-probe | 3 | medium uncertainty; runs claude -p and codex exec in a scratch project; needs authenticated harness CLIs; independent of other changes | anthropic/claude-opus-5-5 | medium | scripted probe and evidence capture; medium class suffices; current model used because it is the only verified route and it can drive claude/codex CLIs in this session (2026-10-04) | Claude Code 2.1.289, native | current agent executes the task in this session; scope = change spec Scope; cwd = repo root or scratch dir; result = files in Scope + execution.md evidence | same | configured and exercised in this session | authenticated claude and codex CLIs (present) |
| team-aware-learning-memory | change-tlm-001-subagent-identity-probe | 4 | medium uncertainty; runs claude -p and codex exec in a scratch project; needs authenticated harness CLIs; independent of other changes | anthropic/claude-opus-5-5 | medium | scripted probe and evidence capture; medium class suffices; current model used because it is the only verified route and it can drive claude/codex CLIs in this session (2026-10-04) | Claude Code 2.1.289, native | current agent executes the task in this session; scope = change spec Scope; cwd = repo root or scratch dir; result = files in Scope + execution.md evidence | same | configured and exercised in this session | authenticated claude and codex CLIs (present) |
| team-aware-learning-memory | change-tlm-002-team-aware-learning-design | 1 | high reasoning; cross-repo synthesis of assessment, analysis and probe verdicts; long-context writing | anthropic/claude-opus-5-5 | high | frontier-class cross-source design synthesis required by kbd-plan for design tasks; current session model (2026-10-04) | Claude Code 2.1.289, native | current agent executes the task in this session; scope = change spec Scope; cwd = repo root or scratch dir; result = files in Scope + execution.md evidence | same | configured and exercised in this session | none |
| team-aware-learning-memory | change-tlm-002-team-aware-learning-design | 2 | high reasoning; cross-repo synthesis of assessment, analysis and probe verdicts; long-context writing | anthropic/claude-opus-5-5 | high | frontier-class cross-source design synthesis required by kbd-plan for design tasks; current session model (2026-10-04) | Claude Code 2.1.289, native | current agent executes the task in this session; scope = change spec Scope; cwd = repo root or scratch dir; result = files in Scope + execution.md evidence | same | configured and exercised in this session | none |
| team-aware-learning-memory | change-tlm-002-team-aware-learning-design | 3 | high reasoning; cross-repo synthesis of assessment, analysis and probe verdicts; long-context writing | anthropic/claude-opus-5-5 | high | frontier-class cross-source design synthesis required by kbd-plan for design tasks; current session model (2026-10-04) | Claude Code 2.1.289, native | current agent executes the task in this session; scope = change spec Scope; cwd = repo root or scratch dir; result = files in Scope + execution.md evidence | same | configured and exercised in this session | none |
| team-aware-learning-memory | change-tlm-003-revised-implementation-plan | 1 | high reasoning; dependency ordering across four repos; integration-gate design | anthropic/claude-opus-5-5 | high | frontier-class dependency ordering across four repos and gate design; current session model (2026-10-04) | Claude Code 2.1.289, native | current agent executes the task in this session; scope = change spec Scope; cwd = repo root or scratch dir; result = files in Scope + execution.md evidence | same | configured and exercised in this session | none |
| team-aware-learning-memory | change-tlm-003-revised-implementation-plan | 2 | high reasoning; dependency ordering across four repos; integration-gate design | anthropic/claude-opus-5-5 | high | frontier-class dependency ordering across four repos and gate design; current session model (2026-10-04) | Claude Code 2.1.289, native | current agent executes the task in this session; scope = change spec Scope; cwd = repo root or scratch dir; result = files in Scope + execution.md evidence | same | configured and exercised in this session | none |
| team-aware-learning-memory | change-tlm-003-revised-implementation-plan | 3 | high reasoning; dependency ordering across four repos; integration-gate design | anthropic/claude-opus-5-5 | high | frontier-class dependency ordering across four repos and gate design; current session model (2026-10-04) | Claude Code 2.1.289, native | current agent executes the task in this session; scope = change spec Scope; cwd = repo root or scratch dir; result = files in Scope + execution.md evidence | same | configured and exercised in this session | none |

EXECUTION ROUND ORDER
Round 1 (independent; executed sequentially by the single in-session agent, no second worker): change-tlm-004-hook-timeout-seconds, change-tlm-001-subagent-identity-probe
Round 2: change-tlm-002-team-aware-learning-design
Round 3: change-tlm-003-revised-implementation-plan

BASE BRANCHES
- change-tlm-004: worktree `/Users/gqadonis/Projects/prometheus/worktrees/tlm-004`, branch `fix/hook-timeout-seconds`, off `origin/main` 20d97f2.
- change-tlm-001, 002 and 003 (repository files): worktree `/Users/gqadonis/Projects/prometheus/worktrees/tlm-design`, off `origin/main` at or after 20d97f2. Phase evidence stays in this checkout's `.kbd-orchestrator/phases/team-aware-learning-memory/`.
- Each change's acceptance gate is `.kbd-orchestrator/changes/<id>/verify.sh`, wired as the last task's `verify` string, so `kbd-apply verify` executes it. It asserts the base contains 20d97f2.

COMMANDS TO RUN
(native-kbd; the change directories already exist from the spec stage)
/kbd-apply change-tlm-004-hook-timeout-seconds
/kbd-apply change-tlm-001-subagent-identity-probe
/kbd-apply change-tlm-002-team-aware-learning-design
/kbd-apply change-tlm-003-revised-implementation-plan

GOAL-TO-CHANGE MAPPING
- Goal 1 (inventory of read/write paths and identity): delivered by `assessment.md` sections A–N (assess stage, two review rounds). The runtime-unverified identity behaviours are closed by change-tlm-001.
- Goal 2 (learning envelope): change-tlm-002 (design sections 2 and 3, plus the schema).
- Goal 3 (per-agent recall with measurement versus today): change-tlm-002 (sections 4, 5 and 10). The baseline is measured in `assessment.md`. **Deferred:** the post-change measurement is a gate in the revised plan, because no recall code ships in this phase.
- Goal 4 (attributed writes and cross-agent awareness): change-tlm-002 (sections 6 and 7).
- Goal 5 (revised plan with integration gates proving per-agent targeting in both harnesses): change-tlm-003. **Deferred:** executing those gates is implementation work. Reflect must score goal 5 on whether the gates are specified and executable, not as proven.

TRADE-OFFS AND SCOPE CUTS
- This phase designs; it does not implement role-aware memory. Implementation is the revised plan's PR sequence (change-tlm-003). Shipping a design and plan rather than code is a deliberate cut, so that PRs 4–14 are not built on unconfirmed harness behaviour.
- Matcher namespacing and the mini timeout fix move into the revised plan instead of being done here.
- Cross-model review is unavailable (JUDGE_MODEL_COLLISION). Every review in this phase is harness-native, same family, and that weaker guarantee is recorded, not hidden.
- change-tlm-001 makes real, authenticated model calls (claude -p, codex exec) in a scratch project. It costs tokens but changes no user configuration.

PLAN COMPLETE
