# learn — Feynman-Spine Learning Domain

The `learn` domain provides Feynman-Spine learning infrastructure for the skill pack: a structured pipeline that takes a learner from goal declaration through diagnostic placement, adaptive curriculum planning, iterative Feynman explanation cycles with sycophancy-corrected grading, spaced repetition, deliberate practice, and verifiable credential issuance. A meta-learning entry point (`learn-about-system`) and harness orientation layer (`learn-harness`) make the system self-describing and portable across AI coding tools.

## Skills

| Skill | Description | Depends On |
|---|---|---|
| `ui-surface` | Cross-harness UI rendering layer | (none) |
| `learn-goal` | Entry point: goal declaration + feasibility gate | ui-surface |
| `learn-survey` | Diagnostic placement + learner model seeding | learn-goal, learner-model |
| `learn-plan` | Adaptive curriculum planner | learn-survey |
| `feynman-loop` | Core Feynman explain/grade/gap/relearn cycle | learn-grade, learner-model |
| `learn-grade` | External sycophancy-corrected grader | content-grounding |
| `learn-retain` | FSRS spaced repetition reviews | learn-grade, learner-model |
| `learn-practice` | Deliberate practice (derivation/implementation/transfer) | learn-grade |
| `learn-certify` | OB 3.0 / W3C VC credential issuance | learn-grade, learner-model |
| `learn-kb` | Custom knowledge base management | content-grounding-kb |
| `learn-about-system` | Meta-learning adoption entry point | learn-goal |
| `learn-harness` | Per-harness capability orientation | ui-surface |

## Installation

Install all learn domain skills alongside the rest of the skill pack:

```bash
bash scripts/install-skills-flat.sh
```

This installs to all detected platforms (Claude Code, Kimi Code, MiniMax, OpenCode, Codex, Cursor, Windsurf, Gemini CLI).

## Substrate

Learner state is implemented by [learner-model](../../substrate/learner-model/README.md).
Its current arithmetic uses the recorded FSRS-5 dependency; FSRS-6 terminology in
older design prose does not select a different scheduler. The
[surface bridge](../../substrate/surface-bridge/README.md) has a separate service
and UI-completion boundary. Installing skills is not proof that either runtime
is started or that a credential has been issued.

Storage and connected peer controls have their own component ownership; the
optional [Companion seam](../../docs/integration-contract.md) does not become a
prerequisite for standalone learning. Read each skill's manifest and references
for provider, content-grounding and operator-input requirements.
