---
type: SessionRecord
id: karpathy-session-481c4c801ede8902
title: Karpathy session 481c4c801ede
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T09:13:49.911657+00:00
created_at: 2026-11-17T09:13:49.911657+00:00
updated_at: 2026-11-17T09:13:49.911657+00:00
revision: 0
---

## Delta

The plan is written: [2026-11-17-skill-pack-certification-plan.md](docs/plans/2026-11-17-skill-pack-certification-plan.md). It is a proposal; I did not install or run any tool in it, and no code changed.

**On agentskills.io certification:** no official programme exists. The spec ships a reference validator (`skills-ref`) for `SKILL.md` files, and there is no published conformance test for harnesses. So the plan defines a certificate this project issues itself, backed by `skills-ref`, `skill-validator` and `avocetta` alongside the repo's own strict validator.

**What the plan contains:**

- **Five levels:** spec conformance, static quality and security, trigger accuracy, outcome (with the skill versus without), and portability across harnesses. Each names its tools and a proposed pass rule.
- **Harness matrix** for all nine: documented skill paths, headless runner, and eval tooling. Harbor is the recommended common runner; its agent list covers Claude Code, Codex, OpenCode, Kimi, Qwen, MiniMax (`mcode`) and MiMo. Zed and DeepSeek Harness need a manual protocol for now.
- **Native code track:** integration evidence per crate, the FFI round trip, Wasm component validation and the installer failure case. Mutation testing, fuzzing and Miri are listed as diagnostics only, because the repo's policy accepts integration evidence alone.
- **Feynman track:** scheduler parity against the reference FSRS implementation, seeded-misconception tests for the grader, sycophancy benchmarks (SycEval, ELEPHANT, SPINE, Petri), and tutoring benchmarks (MathTutorBench, EduClaw-Bench).
- **Karpathy track:** split into the three things that name covers here — the LLM wiki, the learning loop and progress memory, and the tokenizer skill.
- **Certificate record, tiering, six phases, and six decisions for you.**

**Discrepancies the harness research turned up** (none checked against an installed binary):

- `CLAUDE.md` lists `~/.opencode/skills/`; OpenCode's docs give `~/.config/opencode/skills`.
- `CLAUDE.md` lists `~/.kimi-code/skills/`; the Kimi CLI docs list `~/.kimi/skills`.
- OpenCode does not look inside a skill directory, so nested bundles like the `deep-research` stages would expose only the parent there.
- The DeepSeek Harness bridge skips nested YAML frontmatter, which would drop the pack's `metadata.tags`.

**Limits:**

- Harness details for MiniMax Code, Zed, MiMo Code and DeepSeek Harness come from documentation and announcements only.
- The native-code tools (cargo and Wasm tooling) are named from general knowledge and were not re-checked in this research.
- Two searches (FSRS reference vectors, Codex skill docs) were rate-limited and not repeated; the Codex row relies on the repo's own notes.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T09:13:40.444845Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
