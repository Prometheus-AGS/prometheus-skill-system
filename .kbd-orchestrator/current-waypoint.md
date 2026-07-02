# Current Waypoint

- **Phase**: phase-okf-llm-wiki-adoption
- **Status**: execute_ready
- **Backend**: native-kbd (via kbd-apply), self-executing (Claude Code CLI)
- **Changes**: 7 of 8 complete
- **Active change**: change-okf-008-integration-verification (last change; 007 done by parallel session)
- **Next command**: `kbd-apply begin-task change-okf-008-integration-verification <task-id>`

## Round order

1. Round 1 (parallel): change-okf-001-vendor-specs, change-okf-002-pk-workspace-baseline
2. Round 2 (parallel): change-okf-003-permissive-okf-parser, change-okf-007-llm-wiki-skills (drafting)
3. Round 3: change-okf-004-okf-writer-and-id-mapping
4. Round 4 (parallel): change-okf-005-index-log-and-body-links, change-okf-006-okf-lint
5. Round 5: change-okf-008-integration-verification

Cross-repo note: changes 003-006 modify prometheus-knowledge-rs (sibling
checkout created by change-okf-002). Pushing/PR against that remote requires
user confirmation (approval gate in execution.md).

Execution contract: `.kbd-orchestrator/phases/phase-okf-llm-wiki-adoption/execution.md`
