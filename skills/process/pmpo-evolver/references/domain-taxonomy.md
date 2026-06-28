# Domain Taxonomy Reference

Research clusters and signal sources for the `trend` evolution perspective. Each cluster defines the competitive landscape the evolver monitors when researching domain directions.

---

## Cluster: AI Tooling & Developer Assistants

**Description:** Tools and frameworks that assist software engineers with code generation, review, refactoring, and autonomous execution.

**Key sources:**
- GitHub Trending (language=all, since=weekly)
- Hacker News (search: "AI coding" OR "code assistant" OR "agent harness")
- arXiv cs.AI (keyword: agent, tool-use, code-generation)
- Releases: `anthropics/claude-code`, `cline`, `aider`, `cursor`, `continue-dev`

**Detection queries:**
```bash
gh search repos "AI code assistant" --sort stars --limit 20
gh search repos "agent harness claude" --sort updated --limit 10
```

**Polling frequency:** weekly

**Staleness TTL:** 7 days

---

## Cluster: LLM Infrastructure & Model Routing

**Description:** Inference routing, proxy layers, model aggregation, and cost optimization tools that sit between applications and LLM providers.

**Key sources:**
- GitHub Trending (topic: llm-router, openai-proxy, model-fallback)
- Releases: `BerriAI/litellm`, `lm-sys/vllm`, `openrouter-ai/openrouter-js`
- Vendor blogs: Anthropic, OpenAI, Google DeepMind
- Papers: arXiv cs.LG routing/efficiency

**Detection queries:**
```bash
gh search repos "llm router model routing" --sort stars --limit 20
gh search repos "litellm alternative" --sort updated --limit 10
```

**Polling frequency:** bi-weekly

**Staleness TTL:** 14 days

---

## Cluster: Rust Language & Ecosystem

**Description:** Rust language evolution, crate ecosystem changes, toolchain updates, and emerging patterns relevant to Rust-based skill development.

**Key sources:**
- Releases: `rust-lang/rust`, `tokio-rs/tokio`, `serde-rs/serde`, `clap-rs/clap`
- This Week in Rust newsletter
- Clippy lint additions (each Rust release)
- crates.io top downloads delta (weekly)

**Detection queries:**
```bash
gh search repos topic:rust --sort stars --limit 20
# Check stable/nightly release notes
gh api repos/rust-lang/rust/releases --jq '.[0:3] | .[].name'
```

**Polling frequency:** bi-weekly

**Staleness TTL:** 14 days

---

## Cluster: Developer Tooling & Skill Ecosystems

**Description:** Plugin/skill/extension ecosystems for AI harnesses — how other tools publish, distribute, and version skill packages.

**Key sources:**
- GitHub Topics: `claude-plugin`, `ai-skills`, `mcp-server`
- Repositories: `agentskills.io` spec implementations, `modelcontextprotocol/servers`
- Harness release notes: OpenCode, Kimi Code, MiniMax, Codex, Cursor, Zed AI

**Detection queries:**
```bash
gh search repos topic:mcp-server --sort stars --limit 20
gh search repos "agentskills" --sort updated --limit 10
gh search repos topic:claude-plugin --sort stars --limit 10
```

**Polling frequency:** weekly

**Staleness TTL:** 7 days

---

## Cluster: Autonomous Agent Frameworks

**Description:** Multi-agent orchestration, self-improving loops, and long-horizon task execution frameworks.

**Key sources:**
- GitHub: LangGraph, AutoGen, CrewAI, Agency Swarm, MetaGPT releases
- arXiv: "autonomous agent", "self-improving", "Darwin Gödel Machine"
- Blog posts from AI labs about agentic behavior

**Detection queries:**
```bash
gh search repos "autonomous agent framework" --sort stars --limit 20
gh search repos "multi-agent orchestration" --sort updated --limit 10
```

**Polling frequency:** weekly

**Staleness TTL:** 7 days

---

## Cluster: Product Analytics & Feedback Systems

**Description:** Tools for collecting, structuring, and analyzing user feedback, telemetry, and behavioral signals for product evolution decisions.

**Key sources:**
- GitHub: PostHog, Plausible, Mixpanel alternatives
- Papers on: RLHF, Constitutional AI, feedback loops
- Releases: `posthog/posthog`, `getlago/lago`

**Detection queries:**
```bash
gh search repos "product analytics open source" --sort stars --limit 20
```

**Polling frequency:** monthly

**Staleness TTL:** 30 days

---

## How the trend perspective uses this taxonomy

During a trend-perspective evolver cycle, the evolver:

1. Selects the 2-3 clusters most relevant to the current `evolution_name` (from `perspective_cursor.trend.clusters`)
2. Runs detection queries for each selected cluster
3. Passes results to liter-llm `complete --model medium` for relevance scoring
4. Surfaces top-3 relevant developments as LearningSignals with `source_type: research-query`
5. Updates `staleness_ttl_minutes` tracking in state.json per cluster

**Model routing summary:**

| Step | Class |
|------|-------|
| Detection queries (bash) | none |
| Relevance scoring | medium |
| Implication synthesis | frontier |
