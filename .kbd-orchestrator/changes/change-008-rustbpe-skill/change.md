---
id: change-008-rustbpe-skill
title: skills/rust/karpathy-tokenizer/ teaching rustbpe
phase: phase-compliance-and-power-multiplier
gaps: [E2]
priority: P2
effort: S
agent: rust-skills:rust-skill-creator
evolver_item_id: null
status: proposed
---

# change-008 — Karpathy Tokenizer Skill

## Context

Karpathy released [`rustbpe`](https://github.com/karpathy/rustbpe) — "the missing
tiktoken training code" — explicitly because the Python `minbpe` was too slow
and HuggingFace tokenizers too bloated. `nanochat` uses it for tokenizer
training. The pack should have a skill that teaches the LLM (and the
forge-rs enricher) how to use `rustbpe` correctly: training BPE, exporting to
tiktoken format for fast inference, and integrating with the
`agent-tokenizer` crate added in change-004.

## Scope

In:

- New skill `skills/rust/karpathy-tokenizer/`:
  - `SKILL.md` — frontmatter + ≤500-line body covering `rustbpe` training,
    common pitfalls (special-tokens encoding, regex pre-tokenization), export
    to tiktoken format, integration patterns.
  - `references/rustbpe-vs-tiktoken.md` — when to train vs. when to load a
    pretrained tokenizer.
  - `references/nanochat-walkthrough.md` — annotated tour of how nanochat uses
    rustbpe in its training pipeline.
  - `templates/train_tokenizer.rs.tera` — a CLI binary that trains a BPE
    tokenizer from a `data/` directory and writes `tokenizer.tiktoken` and
    `tokenizer.json`.
  - `templates/load_tokenizer.rs.tera` — runtime loader that reads either
    format.

Out:

- Modifications to `agent-tokenizer` crate template — that's part of change-004.

## Deliverables

1. Complete skill at `skills/rust/karpathy-tokenizer/`.
2. Templates that compile and run correctly with `rustbpe@latest`.

## Acceptance Criteria

- `forge template validate skills/rust/karpathy-tokenizer/` clean.
- The `train_tokenizer` template, rendered against a small text fixture,
  produces a working tokenizer that round-trips text via encode→decode.
- Documentation references current `rustbpe` API (link-checked).

## Files to Touch (all new)

- `skills/rust/karpathy-tokenizer/SKILL.md`
- `skills/rust/karpathy-tokenizer/skill.toml`
- `skills/rust/karpathy-tokenizer/references/{rustbpe-vs-tiktoken,nanochat-walkthrough}.md`
- `skills/rust/karpathy-tokenizer/templates/{train_tokenizer,load_tokenizer}.rs.tera`

## Test Plan

- Unit: render templates, run `cargo check` on output.
- Integration: a fixture-driven test that trains a tokenizer on
  `tests/fixtures/sample.txt` and confirms encode/decode round-trip.
- Doc sanity: `npm run validate` plus link-check on referenced URLs.
