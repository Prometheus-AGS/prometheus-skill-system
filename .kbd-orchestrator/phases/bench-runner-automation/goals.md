# Goals — bench-runner-automation

- G1 Stage runner: a `RESEARCH_STAGE_RUNNER` command (bash) that drives each pipeline stage 01–10 via the liter-llm gateway (gpt-5.5 via the openai-proxy :8181 already wired as the KBD judge role) — feeds each stage's SKILL.md as prompt + package state to the gateway model and captures the artifacts per run-research.sh validate_stage contracts. Single command per task; stages run without agent invocation.
- G2 Evidence labeller: a Python pass that, after stage 04 (collect), adds `verified`/`unverified`/`partial` labels to each claim in `sources/registry.json` based on credibility scores + chunk-text presence — closes the drt-006 verified_claim_ratio=0.0 gap so bench scores truthfully.
- G3 Multi-task orchestrator: a driver that runs all 10 bench tasks (51,58,66,71,75,79,81,83,85,87) end-to-end using G1, writing packages to `/Users/gqadonis/.prometheus/research/bench-g5/<task-id>/<slug>-<id>/`, with retry/backoff on gateway rate limits and stage-timeout watchdog.
- G4 Bench runbook: fixes the nested→flat layout run-bench expects (symlink `<bench-g5>/task-<id>` → the package), invokes `run-bench.sh` per run + `score-race.sh` + `score-fact.py` per task, and writes each task's row to `BENCH-RESULTS.md`.
- G5 Complete drt-006 task 4 (the bench RACE run on the 10-task subset) — the deliverable that closes the parent's G5 metric and moves it to MET.
- G6 Update parent reflection (G5 MET, completion 100%) and re-gate + re-close.

## Context

Parent phase: `deep-research-onyx-parity`. This child closes the parent's G5 (RACE/falsifiable measurement) so the parent can move from 90% to 100%. Pilot task-51 was built agent-driven and scored RACE 64.0 but verified_claim_ratio=0.0; nine tasks remain and the labels gap must close for a real score. The pipeline scripts (`run-research.sh` + 10 stage skills at `skills/research/deep-research/skills/stage-NN-*/SKILL.md`) are designed for autonomous execution via `RESEARCH_STAGE_RUNNER` (env var at run-research.sh:17) but that path is currently unused.

## Constraints (carried from parent)

- Implementation-first, integration-only evidence (no mocks).
- Local-only validation; no hosted CI.
- bash 3.2 compatible (C-05); set -euo pipefail.
- KBD_PRODUCER_MODEL = glm-5.3 throughout the orchestrator.
- Use the existing KBD-wired liter-llm proxy at :4000 + the openai-proxy at :8181; no new services.
- No secrets in tracked files (C-02).

## Open questions

- Does the runner need to handle the gateway's intermittent timeouts (we saw 3-attempt ladder to 3600s in earlier rounds) with explicit backoff and resume from the last-completed stage? Yes — the runner is the natural place for this.
