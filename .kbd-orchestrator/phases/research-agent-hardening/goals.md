# Goals

- G1 Execution: prometheus-research daemon and run-research.sh actually execute every pipeline stage and fire the existing hooks; the --daemon-job spawn flag, the 1970 timestamp helper, one output root, and one manifest.json schema are fixed
- G2 Learn coherence: feynman-loop, learn-grade, learn-retain, and learn-certify share one artifact path and one corpus schema (key_points, misconceptions), and learner-model gains add_gap and add_session write paths plus certified_at
- G3 Provenance: deep-research and feynman-loop emit the four-label vocabulary (verified, unverified, blocked, inferred), a <slug>.provenance.md sidecar, a plan file that acts as task ledger and verification log, and --resume reads the stage checkpoint
- G4 Duties: the four research agents carry tool allowlists, verifier completes before reviewer, a scale gate keeps narrow questions out of multi-agent mode, and the final report is judged by adversarial-review so the producer never reviews itself
- G5 Scoring: source credibility uses available-signal weight renormalization with a rank sensitivity artifact, and the graph builder emits content-addressed claim IDs and contradicts edges

## Context

> Phase: `research-agent-hardening`
> Created: 2026-09-03 by `/kbd-new-phase` (runtime revision 613)
> Predecessor in the waypoint: `control-plane-to-companion`, parked at
> `/kbd-apply change-cpc-006-discovery-and-pairing` (16 of 22 project-wide).
> Next command: `/kbd-assess research-agent-hardening`

Source analysis: "Two Feynmans", published 2026-09-03 at
https://claude.ai/code/artifact/874b2865-dae8-45c0-8e68-3734d3b520e0
Reference: `/Users/gqadonis/Projects/references/feynman` (Feynman CLI 0.3.47,
companion-inc research agent on the Pi coding agent). It is a research agent,
not a Feynman-technique learning tool.

Defects verified by command on 2026-09-03 that this phase must close:

- `substrate/prometheus-research/src/job/spawn.rs:48` passes `--daemon-job`;
  the clap `Cli` in `main.rs` does not define it. No daemon job ever runs.
- `substrate/prometheus-research/src/job/checkpoint.rs:60` formats every
  timestamp as `1970-01-01T00:00:SSZ`.
- `skills/research/deep-research/scripts/run-research.sh` emits started and
  completed per stage without executing anything and reports `complete`.
- feynman-loop writes `artifacts/<artifact-id>.json`; learn-retain globs
  `artifacts/<concept-id>-*.json`; learn-certify reads `artifacts/<concept-id>/`.
- `content-grounding-kb.sh` emits `is_misconception` and `content_summary`;
  learn-grade expects `key_points[]` and `misconceptions[]`.
- `skills/research/deep-research/references/` contains no `inferred` or
  `blocked` verification label.

Survey-only findings to confirm during assess: learner-model has no write path
for `gaps` or `sessions`; four incompatible `manifest.json` shapes; two output
roots (`~/.prometheus/research/` vs `~/.research-jobs/`); `post-stage.sh`
checkpoint is never read; adversarial-review is not wired into research or learn.

Goal order is a dependency chain: G1 before G3 and G4, G2 before G3.
