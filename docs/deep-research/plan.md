# Plan: Deep Research UI Multi-Tenancy Redesign

## Reference

- Current UI: `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/docs/deep-research/deep-research-ui.html` (3,342 lines, tj-deep-research reference)
- Master spec: `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/docs/deep-research/index.md`
- Feedback: Anthropic Fable 5 analysis with two-zone navigation, job-scoped state, settings taxonomy, protocol architecture
- System context: UAR, Liter-LLM providers, MCP/AG-UI/A2A/A2UI, surreal-memory, Feynman learning

## Design Decisions

1. **Job-scoped state model** replaces flat app-level state. Every source, graph node, contradiction, report, and trace belongs to exactly one job.
2. **Two-zone navigation** separates workspace (global) from job context (per-job). Job context only appears when a job is selected.
3. **Job switcher** replaces static nav — a dropdown showing the active job name + status dot, with all running/recent jobs listed.
4. **Settings** is a single left-rail panel with 10 sub-panels, not a top-level nav item. Skill Pack moves from sidebar to Settings → Skills.
5. **Knowledge Bases** are global assets that can be attached to any job at setup time.
6. **New Research** is expanded from a simple form to a full wizard with: Query, KBs, Agents, Model routing, Skills, Budget.
7. **Protocols panel** renders the AG-UI + MCP + A2A projection diagram as a visual system architecture.
8. **A2UI component registry** is managed in Settings → Protocols, with install/version/extend capabilities.
9. **Research Profiles** are saved run configurations (skills + KBs + agents + model routing) for quick re-runs.
10. **Per-job telemetry** shows tokens, provider spend, wall-clock, and stage timings.

## State Model (Alpine.js)

```js
jobs: [
  {
    id: string,           // UUID
    name: string,         // auto-generated or user-edited
    query: string,        // original query
    status: 'idle'|'running'|'paused'|'completed'|'failed'|'queued',
    config: {
      deepMode: bool,
      verifySources: bool,
      buildGraph: bool,
      feynmanCheck: bool,
      activeSkills: string[],
      knowledgeBases: string[],
      agentTopology: 'single'|'planner-workers'|'adversarial-verify',
      concurrencyLimit: number,
      modelRouting: { planner: string, searcher: string, verifier: string, critic: string },
      budgetCaps: { tokens: number, cost: number }
    },
    stages: Stage[],      // per-job 10-stage pipeline
    sources: Source[],
    graphNodes: Node[],
    contradictions: Contradiction[],
    report: string,       // markdown content
    trace: AGUIEvent[],   // persisted AG-UI event stream
    metrics: { tokens: number, cost: number, wallClock: number, stageTimings: {} },
    createdAt: Date,
    updatedAt: Date
  }
],
activeJobId: string|null,   // null = no job selected
workspaceView: 'research'|'new'|'knowledge-bases'|'settings',  // global views
jobView: 'overview'|'report'|'sources'|'graph'|'contradictions'|'activity'|'export',  // per-job views
settingsPanel: 'agents'|'models'|'skills'|'mcp'|'context'|'memory'|'feynman'|'wiki'|'protocols'|'resources',
researchProfiles: Profile[],
knowledgeBases: KnowledgeBase[],
a2uiComponents: A2UIComponent[],
providers: Provider[],     // Liter-LLM catalog
uarAgents: Agent[],        // UAR agent registry
```

## Navigation Redesign

### Sidebar (desktop) / Bottom Nav (mobile)

**Workspace Zone (global, always visible):**
- Research (job list with status chips)
- New Research
- Knowledge Bases
- Settings

**Job Context Zone (appears below workspace when activeJobId set):**
- Job Switcher (dropdown: active job name + status dot)
- Overview (per-job progress, stages, media cards)
- Report
- Sources (with count badge)
- Knowledge Graph
- Contradictions (with count badge)
- Activity (AG-UI event trace)
- Export

### Mobile
- Bottom nav: Research, New, Sources, Report, More (drawer)
- Job switcher accessible from Research or top bar
- Settings accessed via More drawer

## Settings Panels (10 total, functional vs stubbed)

### Functional (v2 ships working):
1. **Agents** — UAR connection, agent registry browser, default topology, concurrency
2. **Models & Providers** — Liter-LLM provider catalog, role→model routing, fallbacks
3. **Skills** — Skill pack management (activate/deactivate per job)
4. **MCP** — Server endpoints, auth, health checks; MCP apps registry
5. **Protocols** — AG-UI endpoint config, A2A agent card, projection diagram, A2UI component registry

### Stubbed ("configured via UAR" placeholder):
6. **Context** — Context-management strategy (windowing, compression, checkpoint cadence)
7. **Memory** — surreal-memory scopes, retention, hybrid-search weights
8. **Feynman** — Quality-gate thresholds, learning-loop behavior
9. **LLM Wiki** — Karpathy-style wiki management: page registry, refresh policy
10. **Resources** — Global concurrency, rate limits, spend ceilings

## New Research Wizard (5 sections)

1. **Query** — textarea + mode toggles (deep mode, verify, graph, feynman)
2. **Knowledge Bases** — multi-select existing KBs + "create new" inline
3. **Agents** — UAR agent picker, topology selection, concurrency limit
4. **Model Routing** — per-role provider assignment from Liter-LLM catalog
5. **Skills & Budget** — skill activation grid + token/cost ceilings

## Protocols Panel Architecture Diagram

Visual diagram showing UAR agent at center, three protocol edges:
- **AG-UI** — this interface is itself an AG-UI client; event trace is actual protocol stream
- **MCP Server** — same agent exposed as tools (`start_research`, `get_status`, etc.) for Claude Code/Desktop
- **A2A** — agent card for agent-to-agent delegation

A2UI registry feeds both native client and MCP-app adapter.

## Build Sequence

### Phase 1: Job-scoped state + two-zone nav + job switcher
- Rewrite state model from flat to job-scoped
- Implement two-zone sidebar navigation
- Add job switcher dropdown
- Create job list view (Research tab) with status chips
- Migrate all existing demo data into job-scoped structure
- Preserve existing mobile bottom nav pattern

### Phase 2: New Research expansion + KB management
- Expand New Research from simple form to 5-section wizard
- Create Knowledge Bases global view
- Implement KB multi-select in New Research
- Add demo KB data

### Phase 3: Settings surface
- Implement Settings panel with left-rail sub-panel navigation
- Functional panels: Agents, Models, Skills, MCP, Protocols
- Stubbed panels: Context, Memory, Feynman, Wiki, Resources
- Create protocol architecture diagram (CSS/SVG)
- Add A2UI component registry table

### Phase 4: Polish
- Add research profiles (saved configurations)
- Add per-job telemetry (tokens, cost, wall-clock)
- Add job lifecycle controls (pause/resume/cancel/re-run)
- Add completion notifications (bell + toast)
- Queue state display
- Ensure all mobile interactions work
- Verify dark mode, brand consistency, responsive breakpoints

## Output

Single file: `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/docs/deep-research/deep-research-ui.html`

Must remain:
- Single HTML file, self-contained
- HTMX 2.0.8 + Alpine.js 3.14.3 (CDN)
- PWA-ready (manifest already exists)
- travisjames.ai brand system preserved
- Mobile responsive with bottom nav
- Demo data pre-loaded so page is not blank
- Working on load without user interaction
