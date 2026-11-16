# VLBV audit of loop skills (CR-17, conflict CF-17)

Date: 2026-11-15. Scope: skills the VLBV change proposal had not read. Method: grep
for test/build/compiler execution and "test-driven" wording, then read each hit.
Rule applied: generated code executes only at B4/B5
(`skills/process/kbd-process-orchestrator/references/boundaries.md`).

| Skill / file | Hit | Disposition |
|---|---|---|
| `iterative-evolver/SKILL.md` | example trigger `on_cycle_complete → cargo test` | **Changed**: example is a notification; note that triggers never execute generated code under KBD/VLBV |
| `iterative-evolver/prompts/execute.md` | "Build verification / Test execution → terminal commands" | **Changed**: under KBD/VLBV these are recorded as boundary-packet candidates, not run |
| `iterative-evolver/references/workflow-integration.md` | `run-tests-after-execute` trigger | **Changed**: replaced by a notification trigger |
| `iterative-evolver/prompts/assess.md`, `agents/assessor.md`, `references/domain/software.md` | Assess runs build tools/test suites to measure current state | **Documented deviation (aligned in practice)**: Assess measures an existing, already-delivered state outside a value phase's generation window; when run inside a KBD value phase it reads `boundary-results.json` instead. No edit: the evolver is domain-agnostic and the SKILL.md note now states the rule |
| `pmpo-outer-loop/references/loop-schema.md` | feedback source `npm test` | **Changed**: annotated to read the phase's boundary results under KBD/VLBV rather than rerun per iteration |
| `kbd-loop/SKILL.md` | "At its boundary BUILD then RUN … Fix any build/launch/function errors" | **Aligned, wording changed**: build/run are packet entries; fixes are an authorized B5 repair |
| `kbd-goal/templates/codex/continuation.md` | per-task "Run the tests / Run the linter" | **Changed**: no per-task execution; notes only |
| `kbd-goal/references/skill-discovery.md` | keyword routing `tdd → tdd-guide` | **Aligned**: discovery table only; routing does not authorize execution. Follow-up: route `test`/`bdd` keywords to `vlbv` first |
| `kbd-init/SKILL.md` step 3 | detects `cargo check/test/clippy` commands | **Aligned**: discovery of commands for the boundary packet, not execution |
| `kbd-init/references/constraints.md` | `on_iteration_complete → build_health_command`, `on_change_complete → test_command` | **Changed**: removed; only a notification trigger remains |
| `skills/imported/artifact-refiner/references/domain/code.md` | "test-driven evaluation", `cargo test` validation column | **Not edited (submodule)**: follow-up in the artifact-refiner repository; the pack's KBD integration (`references/integrations/artifact-refiner.md`) already limits the refiner to one static pass with no build |
