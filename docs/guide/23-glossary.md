# 23 · Glossary & Sources

## Glossary

**A2A (Agent-to-Agent).** A protocol for agents to call each other. A generated native agent advertises an agent card at `/.well-known/agent.json` and accepts tasks at `/a2a/tasks`.

**A2UI.** The Prometheus combined agent-and-UI protocol, served at `/a2ui/session`, that streams both agent interaction and UI updates.

**AG-UI.** A CopilotKit-compatible protocol for streaming agent runs to a UI over Server-Sent Events (`agui.*` events). Drives the generated React frontend.

**AgentSkills.io.** The portable skill specification every native skill in the pack conforms to. Defines the `SKILL.md` frontmatter schema, the name pattern, and the standard directory layout.

**Cedar.** The policy language used by the Skill-Mutation policy enforcement point. Default-deny; governs `skill.mutate`, `skill.generate`, `skill.promote`, and `trace.capture` per environment.

**Constitution.** A per-language set of standards and denied patterns that forge-rs checks code against and injects into enrichment context (`constitution_summary`).

**Constraint manifest.** The output of the ZeeSpec interrogator — a scored GO / CAUTION / NO-GO assessment of whether work is specified well enough to proceed.

**evolver-bridge.json.** The write-back contract between the KBD inner loop (L1) and the evolver (L2). Maps completed changes to evolution items so the strategic loop knows what landed.

**forge-rs.** The Layer 4 code-enrichment engine. Injects language knowledge before an agent writes code; processes reflections back into the Karpathy loop.

**Karpathy pattern.** A flat-file Markdown knowledge base, maintained by the AI and queried directly by a long-context model, used instead of vector RAG. Every claim traces to a readable file.

**KBD (Knowledge-Based Development).** The Prometheus methodology that keeps domain knowledge and code aligned across sessions, via KB priming, hard phase discipline, and waypoint continuity.

**liter-llm.** The multi-provider LLM gateway (140+ providers) that does per-phase model routing. Frontier models for reasoning phases, cheap models for status.

**Loop levels (L0–L3).** L0 harness micro-loop, L1 tactical KBD loop, L2 strategic evolver loop, L3 outer standing loop. State is harness-agnostic; the driver is harness-specific.

**loop.json.** The definition of an L3 standing loop: goal, feedback sources, termination, cadence, and the backing evolution name.

**loop-tick.sh.** The runner that advances one outer-loop tick. Exit codes: 0 continue, 1 escalate, 2 terminate, 3 error.

**Metaprompting.** Designing a system of prompts — routing, critique, evaluation — rather than a single prompt. The basis for critic-context isolation.

**PMPO (Prometheus Meta-Prompting Orchestration).** The two-loop methodology (inner task loop + outer evolution loop) whose load-bearing constraint is phase discipline.

**prometheus-knowledge / pk.** The Rust implementation of the Karpathy KB. `pk focus` primes context; `pk ingest` writes learning back. MCP bridge `pk-cherry` on port 8942.

**Progress signals.** Mandatory start/completion lines emitted every phase and task, with counts read from `progress.json`, so multi-session work stays resumable.

**Sycophancy patterns (S-01–S-08).** The eight classified patterns the correction server detects, from unprompted affirmation (S-01) to reflect-phase inversion (S-08).

**surreal-memory.** The semantic knowledge graph (SurrealDB + HNSW) on port 23001. Stores relationships, scoped memory, task streams, and mindmaps.

**Waypoint.** The resume contract in `.kbd-orchestrator/current-waypoint.json` / `position-reminder.txt` that restores an agent's exact position at session start.

**ZeeSpec.** The Zachman-Framework 5W1H interrogator that gates under-specified work at Layer 1.

## Sources

External claims in this guide are validated against the following sources.

### Loop engineering and Boris Cherny

- [The Anthropic leader who built Claude Code says he ditched prompting — now he just writes loops — The New Stack](https://thenewstack.io/loop-engineering/)
- [Loop Engineering — Addy Osmani](https://addyosmani.com/blog/loop-engineering/)
- [Loop Engineering — Cobus Greyling](https://cobusgreyling.substack.com/p/loop-engineering)
- [Claude Code Loop Engineering: Stop Prompting, Start Designing Autonomous Agent Workflows — TechTimes](https://www.techtimes.com/articles/318828/20260622/claude-code-loop-engineering-stop-prompting-start-designing-autonomous-agent-workflows.htm)
- [Boris Cherny, Claude Code creator, on shifting from prompting to autonomous loops — Digg](https://digg.com/ai/v1igoqs7)

### Karpathy knowledge-base pattern and minBPE

- [Karpathy shares 'LLM Knowledge Base' architecture that bypasses RAG — VentureBeat](https://venturebeat.com/data/karpathy-shares-llm-knowledge-base-architecture-that-bypasses-rag-with-an)
- [What Is the Karpathy LLM Wiki Pattern? — MindStudio](https://www.mindstudio.ai/blog/karpathy-llm-wiki-knowledge-base-pattern)
- [LLM Knowledge Bases — DAIR.AI Academy](https://academy.dair.ai/blog/llm-knowledge-bases-karpathy)
- [karpathy/minbpe — minimal Byte Pair Encoding for LLM tokenization (GitHub)](https://github.com/karpathy/minbpe)

### Kimi K2.6 / K2.7 Code

- [Moonshot AI Releases Kimi K2.7-Code: +21.8% on Kimi Code Bench v2 over K2.6 — MarkTechPost](https://www.marktechpost.com/2026/06/12/moonshot-ai-releases-kimi-k2-7-code-a-coding-model-reporting-21-8-on-kimi-code-bench-v2-over-k2-6/)
- [Moonshot AI's Kimi K2.7-Code Targets Token Efficiency in Agentic Coding — DevOps.com](https://devops.com/moonshot-ais-kimi-k2-7-code-targets-token-efficiency-in-agentic-coding/)
- [Kimi K2.7 Code: Open-Source Agentic Coding Model — Moonshot AI](https://www.kimi.com/resources/kimi-k2-7-code)

### Firecrawl vs. Tavily

- [Firecrawl vs. Tavily: 2026 guide for RAG and agent pipelines — Apify](https://blog.apify.com/firecrawl-vs-tavily/)
- [Firecrawl vs Tavily: Complete Comparison for AI Agents & RAG (2026) — Firecrawl](https://www.firecrawl.dev/alternatives/firecrawl-vs-tavily)
- [firecrawl/firecrawl — AGPL-3.0, self-hostable (GitHub)](https://github.com/firecrawl/firecrawl)

### Metaprompting and critic models

- [Meta-Prompting: Enhancing Language Models with Task-Agnostic Scaffolding — arXiv 2401.12954](https://arxiv.org/pdf/2401.12954)
- [The Meta-Prompting Protocol: Orchestrating LLMs via Adversarial Feedback Loops — arXiv 2512.15053](https://arxiv.org/abs/2512.15053)
- [Meta-Prompting: LLMs Crafting & Enhancing Their Own Prompts — IntuitionLabs](https://intuitionlabs.ai/articles/meta-prompting-llm-self-optimization)

### Internal references

The authoritative internal sources are the repository itself: `README.md`, `SKILLS.md`, `CLAUDE.md`, `docs/loops-architecture-spec.md`, `docs/CONTRIBUTING.md`, `docs/SUBMODULES.md`, `docs/IMPORTING_SKILLS.md`, the article `docs/articles/autonomous-loops-prometheus-skill-pack.md`, each skill's `SKILL.md`, each tool's `Cargo.toml` and source, `hooks/hooks.json`, `.claude-plugin/plugin.json`, `marketplace/marketplace.json`, `scripts/mcp-port-table.json`, and the `sycophancy-correction` server's own `skill_info` output.

---

*Previous: [← 22 · Advantages & Impact](22-advantages-and-impact.md) · Back to [Index](README.md)*
