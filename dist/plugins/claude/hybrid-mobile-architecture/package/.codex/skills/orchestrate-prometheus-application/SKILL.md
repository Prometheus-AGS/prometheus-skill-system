---
name: orchestrate-prometheus-application
description: Classify and orchestrate Prometheus application work across hybrid, mobile-only, desktop, automation, SaaS, agent-client, ideation, native-agent, deployment, and documentation scenarios. Use when planning or prompting a multi-phase product build, selecting models or harnesses, running KBD/Feynman/Karpathy loops, or generating missing skills and agents.
---

# Orchestrate Prometheus Application

Read the consuming project's base rules, architecture authority, and dated model
registry. Never infer model capabilities from names or copy mutable prices/context
into stable guidance.

Progressive references:

- `references/scenario-classification.md` — known/composite scenario classification and dependency-ordered asset manifest.
- `references/control-loop.md` — Feynman/KBD/PMPO/Karpathy loop rules, authority boundaries, producer/critic selection, and completion gates.
- `references/native-agent-decision.md` — prompt versus skill versus native-agent decision guide.

## Control loop

`Feynman learn → KBD assess → research → decision-complete plan → bounded implementation → public-boundary verification → adversarial critic → Karpathy retention → next waypoint`

1. Classify the product using `references/scenario-classification.md`: full hybrid,
   Flutter-only, Tauri local agent, automation, SaaS, local agent client, ideation
   studio, native Rust agent, multi-cloud deployment, branded documentation, or a
   composite of those scenarios.
2. If domain or requirements are unclear, explain them simply, grade the explanation,
   research gaps, and repeat before architecture.
3. Emit a dependency-ordered reference manifest covering architecture, recipe,
   harness, loop, role, retention, and verification assets.
4. Produce staged harness-specific prompts with outcome, sources, authority, artifacts,
   public-boundary criteria, budgets, retry limits, and stop conditions.
5. Select producer and independent critic roles from the current registry. Do not
   duplicate or hand-edit the model catalog inside this skill.
6. Record Karpathy progress at phase boundaries. Require
   `hybrid-runtime-verification` before “working” or “complete.”
7. When a repeated operational gap is proven, use the skill creator and validate the
   new skill in a scratch project. When the missing capability requires an independent
   runtime/protocol lifecycle, use the native-agent creator.

## Current Prometheus control plane

- Require `prometheus kbd projects` to confirm project UUID identity before lifecycle writes.
- Treat the signed grow-only Loro event map and per-replica journals as canonical;
  generated waypoint/progress files are read-only projections.
- Use `prometheus kbd claim`, `conflicts`, and `resolve` for concurrent work. Do not
  emulate the removed lease/fencing or handoff model.
- Express work through typed `phase`, `stage`, `change`, `task`, `completion`,
  `decision`, and `blocker` commands; use pause/revise/resume/cancel for lifecycle.
- Inspect durable learning delivery with `prometheus learning status --json`.

Do not let PMPO rewrite requirements to fit a failing output, let a producer certify
its own work, or run an autonomous loop without explicit authority and termination.
