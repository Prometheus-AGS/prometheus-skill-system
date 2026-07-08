# Handoff: plan → execute

_Written: 2026-07-08 by kbd-plan_

## Summary

5 changes in execution order: prui-001 (SKILL.md prose update, G-01), prui-002 (UI real
SSE replacing simulation, G-02), prui-003 (emit.rs UiIntent schema fix + ui-surface SKILL.md
update, G-03 HIGH risk), prui-004 (CI workflow, G-04), prui-005 (smoke test script, G-05).
Ordering: SKILL.md first (no deps), UI second (Tier 1 SSE path works before G-03), surface-bridge
fix third (unblocks Tier 2), CI fourth (independent), smoke test last (validates full stack).
First command: /kbd-apply change-prui-001-skill-md-update.
