# change-ldd-16-agent-team-handbook

Title: Publish complete agent-team and model-operation documentation in both sites
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: full+mini
Gaps: G18
scope:
  - full:docs/guide/24-agent-teams.md
  - full:docs/agent-teams.md
  - full:site/docs/agent-teams/**
  - full:site/docs/kbd/task-model-assignments.md
  - mini:docs/agent-teams.md
  - mini:site/docs/agent-teams/**
  - mini:site/docs/kbd/task-model-assignments.md
  - full+mini:owned team/model skill references and examples

## Requirements

1. Build a task-oriented handbook with executable examples: prerequisites; discover/create/import teams; select a project team; roles and path ownership; native harness bindings; manifests/schema versions; start/assign/accept/update/cancel/complete work; evidence and recovery. Explain canonical task IDs, revisions, claims and permission boundaries.

2. Explain same-team communication, context-bearing handoff creation/acceptance, stale/conflicting revisions, cross-harness transfer, direct coordinator task acceptance and durable records. Include a complete two-role example from team creation to accepted handoff and verified completion.

3. Explain cross-project coordination: team cards/discovery, issue-backed requests where implemented, repository identity, project/role scoping, approval to message/write another project, routing to owner, correlation/receipt and failure recovery. Do not describe a request as an accepted task or conflate same-repository handoff with cross-repository ownership transfer.

4. Document model discovery, configured access versus actual successful inference, explicit provider/model IDs, team→role→ordered skill→task precedence, accumulated capabilities versus scalar overrides, reasoning settings, context/tool/vision needs, budget and unknown/stale pricing, fallback rules and independent review. Show how to persist the selected ID and inspect effective policy. liter-llm supplies inference; a tool-enabled worker supplies execution.

5. Document Codex/Claude/OpenCode/Kimi/DeepSeek capability differences from their supported native contracts. No invented flags or universal per-role switching claims. Include examples of unsupported controls and how to report an unresolved route. Cover role-private/project/team/user/global memory, digest versus lesson text, learning writeback, promotion, Cortex optional mirror and privacy boundaries.

6. Each full/mini page states supported features, native/service prerequisites and differences. Link canonical guide sources rather than maintaining conflicting duplicates. Keep diagrams and JSON examples pinned to actual schemas and new direct-task-acceptance behavior.

## Binding constraints

Source work uses isolated worktrees under ../worktrees with declared base/provenance; protect deploy-main/deploy/main and other agents' changes. All tests, builds and reviewers wait for completed production and run only locally under change 12. Use scratch HOME/CODEX_HOME/CORTEX_DATA_DIR/plugin/learning roots; never test against live :23001. One Cargo/rustc process machine-wide; pgrep first. User merges every PR and explicitly approves versions.toml edits, tags and real Codex/Claude changes. No approval is inferred from planning. See the phase plan for ordering, exact task model assignments and final gates.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-16-agent-team-handbook` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.
