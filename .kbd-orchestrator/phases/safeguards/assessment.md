# Assessment — safeguards

Phase 3 of the approved framework-evolution plan. Builds on Phases 1–2.

## Ground truth (verified by reading the scripts)

| Fact | Location | Implication |
|------|----------|-------------|
| protect-tests.sh referenced by CLAUDE.md but ABSENT | CLAUDE.md "BDD Immutable-Tests Rule" | docs make a false claim today — highest-priority fix |
| sycophancy MCP invocation logic is self-contained | sycophancy-check-reflection.sh:40-204 | extractable into a lib; the SubagentStop wrapper becomes thin |
| Gate only fires for the `reflector` SubagentStop | hooks.json SubagentStop matcher | main-loop kbd-reflect output (a Write, not a subagent) is ungated |
| pipeline-enforce keys off Bash input mentioning kbd-execute/kbd-reflect | pipeline-enforce.sh:38 | add a kbd-new-phase/kbd-next-phase rule keyed on reflect_gate |
| PreToolUse Write\|Edit group has cedar-skill-gate only | hooks.json:42-50 | protect-tests + scope-guard append here (widen matcher to add MultiEdit) |
| PostToolUse Write\|Edit\|MultiEdit has validate-state + gitops | hooks.json:52-66 | sycophancy-check-artifact + scope-record append here |
| No `scoped_paths` in waypoint; no scope concept anywhere | waypoint schema | scope guard introduces it; every Phase 1-2 change already carries `scope:` frontmatter to enforce retroactively |

## Gaps this phase closes

| ID | Gap | From plan |
|----|-----|-----------|
| H1 | protect-tests.sh missing — false CLAUDE.md claim; nothing blocks edits to existing BDD test files. | Phase 3.1 |
| H2 | No change-set scope guard — an execution loop can edit any file regardless of the active change's declared `scope:`. | Phase 3.2 |
| H3 | Sycophancy gate covers only the reflector subagent; kbd-reflect/assessment main-loop artifacts are ungated. | Phase 3.3 |

## Constraints

- Hooks snapshot at session start — these won't take effect until reload (same
  as Phase 1 hooks; documented, not blocking).
- Scope guard ships in **warn** mode (user decision) — logs + non-blocking
  notice first release; flip to ask in Phase 6.
- PostToolUse cannot un-write; the sycophancy artifact gate gets teeth via a
  progress.json `reflect_gate` flag + pipeline-enforce block at the next
  lifecycle boundary.
- All hooks degrade gracefully (binary/state absent → exit 0) and log JSONL.

## Verdict

GO. H1 is a single new script closing a documentation lie. H2/H3 extend
existing hook groups and reuse proven MCP-invocation code. No schema-breaking
changes; scope fields are additive.
