---
type: SessionRecord
id: karpathy-session-f2aa837f241d4290
title: Karpathy session f2aa837f241d
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-07T13:56:17.237923+00:00
created_at: 2026-08-07T13:56:17.237923+00:00
updated_at: 2026-08-07T13:56:17.237923+00:00
revision: 0
---

## Delta

No single industry term covers what we're doing. It's a **composite of four established patterns**, and the last one is the least standardized.

## The four names

**1. Dogfooding** — the umbrella. You're building KBD and using it on your own real work rather than a synthetic test. Standard practice; not the interesting part.

**2. Reference implementation / exemplar project** — the closest fit for UAR's *role*. A working system that demonstrates correct behavior and, critically, **verifies the specification is implementable**. That's precisely what UAR is doing: proving the KBD lifecycle survives contact with 35k lines, 187 OpenSpec changes, and 47 phases of accumulated legacy state.

**3. Pilot project** — the closest fit for the *session structure*. Time-bounded, scoped, feedback-driven validation before wider rollout. The literature calls this **process prototyping**: running a real project to reveal a process's strengths and weaknesses early, tailoring the process *during* development rather than after.

**4. Self-hosting / bootstrapping** — the closest fit for the *fix loop*. This is the one that actually describes today. I used `prometheus kbd` to manage the work, and that use exposed defects in `prometheus kbd`, which I then fixed, rebuilt, reinstalled, and re-exercised through the same tool. A compiler compiling itself.

Notably, [a 2026 paper](https://arxiv.org/html/2603.17399) extends the self-hosting analogy directly to coding agents — arguing the **specification**, not the bootstrap tool, is the stable artifact. That maps onto your setup: the KBD lifecycle is the spec; Claude Code and Codex are interchangeable bootstrap tools.

## What has no clean name

The **cross-harness handoff under a shared canonical state** — Claude Code running assess/analyze/plan, Codex resuming execute, both writing to one event-sourced control plane. The agent-harness literature is all *benchmarks*: sandboxed environments with hidden ground truth and deterministic graders. Nobody's named "two production agents in different harnesses passing a live phase back and forth."

That gap is where the real finding came from. The C-00 reversion was only possible **because** two harnesses shared state across a migration boundary — a defect no single-agent benchmark would surface.

## What I'd call it

**Bootstrapped dogfooding against an exemplar project** — or, in your own vocabulary, a **proving case**. Which is what you called it before asking, and it's about as precise as the industry gets.

The most defensible single term is **process prototyping on a pilot project**, since that's the one with actual academic backing for "run the real thing to find out whether the process works."

Sources:
- [Dogfooding at JetBrains](https://blog.jetbrains.com/life-at-jetbrains/2026/05/dogfooding-at-jetbrains/)
- [Reference implementation — Wikipedia](https://en.wikipedia.org/wiki/Reference_implementation)
- [Observation-based Development of Software Process Baselines](https://arxiv.org/pdf/1403.2718)
- [Bootstrapping Coding Agents: The Specification Is the Program](https://arxiv.org/html/2603.17399)
- [Self-hosting (compilers) — Wikipedia](https://en.wikipedia.org/wiki/Self-hosting_(compilers))
- [Beyond Accuracy: Evaluating Enterprise Agentic AI Systems](https://arxiv.org/pdf/2511.14136)

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-07T13:51:36.001038Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- .prometheus/knowledge/wiki/index.md
- .prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-complete-unknown-change.md
- .prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-completion-record.md
- .prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-completion-unknown-change.md
- .prometheus/knowledge/wiki/log.md
- tools/prometheus-cli/.prometheus/events.jsonl
- tools/prometheus-cli/.prometheus/knowledge/.prompt-snapshots/project/current
- tools/prometheus-cli/.prometheus/knowledge/wiki/executor-session-completion-kimi-desktop-extensibility.md
- tools/prometheus-cli/.prometheus/knowledge/wiki/index.md
- tools/prometheus-cli/.prometheus/knowledge/wiki/log.md
- .prometheus/knowledge/wiki/karpathy-session-01343af7d0fe4293.md
- .prometheus/knowledge/wiki/karpathy-session-20dadb89eb822742.md
- .prometheus/knowledge/wiki/karpathy-session-253e6d789e51e2ca.md
- .prometheus/knowledge/wiki/karpathy-session-2580f8aab12a344f.md
- .prometheus/knowledge/wiki/karpathy-session-3d8fb7e5d4301eb7.md
- .prometheus/knowledge/wiki/karpathy-session-57b550052706da1d.md
- .prometheus/knowledge/wiki/karpathy-session-5ba81ce56f70adfa.md
- .prometheus/knowledge/wiki/karpathy-session-6e6925d2d6588b9d.md
- .prometheus/knowledge/wiki/karpathy-session-7870daf25bc9f28f.md
- .prometheus/knowledge/wiki/karpathy-session-7d4f066577adcb95.md
- .prometheus/knowledge/wiki/karpathy-session-8e446017ed66cb65.md
- .prometheus/knowledge/wiki/karpathy-session-a6ff6efa34616b26.md
- .prometheus/knowledge/wiki/karpathy-session-b59e456a02d42622.md
- .prometheus/knowledge/wiki/karpathy-session-c78d9c4b94ed6241.md
- .prometheus/knowledge/wiki/karpathy-session-d68b8a8c3be4f9df.md
- .prometheus/knowledge/wiki/karpathy-session-da3c988e8062b513.md
- .prometheus/knowledge/wiki/karpathy-session-e6f5d70de34880a9.md
- .prometheus/knowledge/wiki/karpathy-session-f2c5b757e52fc16e.md
- .prometheus/knowledge/wiki/karpathy-session-f6805d6e53df91bd.md
- .prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-unknown-change-completion.md
- .prometheus/knowledge/wiki/kimi-desktop-extensibility-session-completed-change-unknown.md
- tools/prometheus-cli/.prometheus/knowledge/.prompt-snapshots/project/generations/08f8dab316aa33a1cc148d8c6b37f588e9df1e23633df6019ccbd6c50bfe64ee.json
- tools/prometheus-cli/.prometheus/knowledge/.prompt-snapshots/project/generations/cc0848be681ebe313a51bd02c28aecf3be9353ebd64830989d6145d0553198e1.json
- tools/prometheus-cli/.prometheus/knowledge/wiki/karpathy-session-6c8842013efef528.md
- tools/prometheus-cli/.prometheus/knowledge/wiki/karpathy-session-d6126f64f63475e4.md
- tools/prometheus-cli/.prometheus/knowledge/wiki/karpathy-session-fac64b52a0f6fa43.md
- tools/prometheus-cli/.prometheus/knowledge/wiki/kimi-desktop-extensibility-executor-session-complete.md
