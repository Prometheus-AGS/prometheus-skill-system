---
id: XC-004
title: prometheus doctor end-to-end loop test
status: planned
priority: P1
estimated_effort: 2-3d
agent_role: skill-pack-maintainer
depends_on: [SP-006, SP-012]
unblocks: []
related: [SP-018]
created_from_conversation_turn: 3-4
---

# XC-004 — `prometheus doctor` end-to-end loop test

## Problem

The skill-pack has many moving pieces — Stop-chain hooks (SP-006), pipeline enforcement (SP-012), Cedar gates (SP-011), reflector sycophancy correction (SP-013), librarian event persistence (SP-019), dual memory store (SP-020), trace capture (SP-007), and codegraph extraction (BDD-008). When something breaks, identifying *which* piece broke requires ad-hoc diagnostics across multiple file paths and processes. There is no single command that says "the loop is healthy" or "here's exactly which piece is broken."

## Evidence

Today, debugging a "the librarian didn't ingest my session" issue means: check `~/.prometheus/hooks.log` (if SP-006 is in), check that the Stop hook fired, check that `forge-reflect-on-stop.sh` exited 0, check that the librarian process saw the event, check that surreal-memory has the entry. Five separate diagnostic steps.

## Why it matters

A health-check command replaces ad-hoc archaeology with a known sequence. The command becomes the canonical onboarding artifact — "did `prometheus doctor` pass?" is the smoke test for "is this environment sane?"

It also functions as the integration test for SP-018 — the smoke test asserts artifacts; doctor asserts the operational loop.

## Proposed fix

A `prometheus doctor` CLI command (in `pk-cli` or a new `prometheus-cli` crate) that runs through a checklist:

**Hook layer:**

- [ ] `~/.prometheus/hooks.log` exists and is current (last entry within 24h if sessions ran).
- [ ] Stop-chain scripts exist and are executable.
- [ ] `hooks.json` files are symlinked (per SP-015).
- [ ] Cedar policies exist and parse.

**Knowledge layer:**

- [ ] surreal-memory is reachable.
- [ ] `kg_*` and `episode_*` namespaces exist (per SP-020).
- [ ] Per-project KB directory exists (per SP-008).
- [ ] LibrarianEvent records have been written in last 7 days (per SP-019).
- [ ] Codegraph for current commit SHA exists (per BDD-008).

**Pipeline layer:**

- [ ] ZeeSpec → PMPO → OpenSpec layers all have at least one artifact in the last 30 days.
- [ ] No reflections rejected for >48h consecutively (per SP-013).
- [ ] Pipeline smoke test (`scripts/test-pipeline-e2e.sh`) passes.

**Cross-source consistency:**

- [ ] STATUS.md and surreal-memory agree (no `done` task in one missing in the other).
- [ ] BUG_FIX_LEDGER.md exists at expected paths.

**Output.** A summary table with red/yellow/green per check. Non-zero exit if any red. JSON output mode (`--json`) for machine consumption (CI gate).

## Trade-offs and risks

- **Risk: doctor itself becomes a point of failure.** Mitigation: keep checks minimal and shell out to existing tools rather than reimplementing. If a check is broken, the user sees specifically which one.
- **Risk: false positives (claims a piece is broken when it's actually fine).** Mitigation: each check has clear evidence; the report shows what was looked for and what was found.
- **Cost: build time.** Bounded — doctor is a maintenance investment that pays back on every diagnostic.

## Acceptance criteria

- [ ] `prometheus doctor` command exists and runs in <30 seconds.
- [ ] All checks above implemented and produce a clear pass/fail status.
- [ ] `--json` output mode for CI.
- [ ] Non-zero exit if any check fails.
- [ ] CI workflow runs `prometheus doctor --json` on schedule (e.g. nightly) and surfaces in a dashboard.
- [ ] Documentation: each check is listed with "what to do if this fails."

## Implementation steps

1. Define check registry: a list of `Check { name, run, severity }`.
2. Implement each check as a small async fn or shell-out.
3. Implement the runner that executes all checks and aggregates results.
4. Add output formatting (table, JSON).
5. Wire into CI as a nightly check.
6. Document.

## Dependencies

SP-006 (hook log to read), SP-012 (pipeline checkable), and ideally SP-013, SP-018, SP-019, SP-020, BDD-008 for full coverage. Doctor can ship with partial coverage and grow.

## Open questions

- Should doctor have a `--auto-fix` mode for common issues? Probably not initially — confused signals about "did doctor break it or fix it" are harder to debug. Read-only first.
- Should it run inside Claude Code as a slash command? Yes, as `/prometheus-doctor`. Same logic.
