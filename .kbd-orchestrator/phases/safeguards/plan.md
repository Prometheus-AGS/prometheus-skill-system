# Plan — safeguards

Backend: native-kbd. Ordered: close the false-claim first (smallest, highest
integrity value), then scope guard, then sycophancy generalization. Each change
carries `scope:` — and from this phase on, that `scope:` is enforced by the
guard the phase ships.

| # | Change | Gaps | Summary |
|---|--------|------|---------|
| 1 | change-001-protect-tests | H1 | `shared/scripts/protect-tests.sh` (PreToolUse Write\|Edit\|MultiEdit): block edits to existing `tests/steps/*.steps.ts`, `tests/support/*.ts`, `tests/features/*.feature`; allow new files + `tests/features/drafts/**`; `PROMETHEUS_ALLOW_TEST_EDITS=1` override; no-op when no `tests/features/`. Wire into hooks.json. Test. Removes the false CLAUDE.md claim by making it true. |
| 2 | change-002-scope-guard | H2 | waypoint `scoped_paths`/`scope_overrides`; `shared/scripts/scope-guard.sh` (PreToolUse, `PROMETHEUS_SCOPE_ENFORCE=off\|warn\|ask` default **warn**): out-of-scope edit → warn notice (warn mode) / ask-JSON (ask mode); `.kbd-orchestrator/**`, `SCRATCHPAD.md` always allowed; `shared/scripts/scope-record.sh` (PostToolUse) records approved expansions. Wire both. Tests. |
| 3 | change-003-sycophancy-artifact-gate | H3 | extract `shared/scripts/lib/sycophancy.sh` (MCP invocation + scoring + counter) from sycophancy-check-reflection.sh (which becomes a thin wrapper, behavior unchanged); new `shared/scripts/sycophancy-check-artifact.sh` (PostToolUse, path-filtered to `**/reflection.md`/`**/assessment.md`): exit-2 feedback + set `reflect_gate:"rejected"` in progress.json; modify pipeline-enforce.sh to block `kbd-new-phase\|kbd-next-phase` while `reflect_gate` rejected. Tests. |

Completion per change: change.md tasks checked, tests green, commit. Phase end:
`npm run validate:strict`, `npm run build`, `validate:signals`, full shell-test
sweep including the new `test-protect-tests.sh`, `test-scope-guard.sh`,
`test-sycophancy-artifact.sh`. Reflection gated.
