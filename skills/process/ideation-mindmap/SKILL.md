---
name: ideation-mindmap
description: Stage-zero onramp for /start-business-build. Takes a one-line business concept and generates a 6-branch concept mindmap via surreal-memory, structuring raw ideas into actionable branches ready for zeespec constraint capture.
license: MIT
version: '1.0.0'
authors:
  - Prometheus AGS
metadata:
  category: process
  tags: [ideation, mindmap, surreal-memory, business-build, stage-zero]
triggers:
  keywords:
    - ideation mindmap
    - concept tree
    - expand idea
    - branch concept
    - business concept branches
    - mindmap my idea
  semantic: >
    User provides a one-line business concept or outcome and wants it
    structured into branched concept clusters before deeper specification.
    Also auto-invoked by /start-business-build as Stage 1.
---

# /ideation-mindmap

Stage-zero onramp for the Prometheus build pipeline. Turns a one-line concept into a 6-branch concept tree using `surreal-memory`'s `generate_ideation_mindmap` tool. Output is formatted for immediate handoff to `/zeespec-interrogate`.

## When to Use

- Standalone: user has a raw business idea and wants to see structured branches before committing to specification
- Auto-invoked: `/start-business-build` calls this as Stage 1 before `/zeespec-interrogate`

## MCP Dependency

Requires `surreal-memory` MCP server to be running. Verify with:

```bash
npm run doctor
```

## Instructions

### Step 1 — Call generate_ideation_mindmap

Invoke the `generate_ideation_mindmap` tool from the `surreal-memory` MCP server:

```
generate_ideation_mindmap(
  topic: "<the one-line concept from $ARGUMENTS>",
  branches: 6
)
```

The tool returns a mindmap with 6 named branches, each containing concept clusters relevant to the topic.

### Step 2 — Format the output

Present the mindmap result as a structured list:

```
Concept: "<original concept>"

Branch 1 — <Branch Name>
  • <concept cluster point 1>
  • <concept cluster point 2>
  • <concept cluster point 3>

Branch 2 — <Branch Name>
  • ...

[... repeat for all 6 branches]
```

Keep each branch to 3–5 sub-bullets. If the MCP result contains more detail, summarize to the most actionable points.

### Step 3 — Present selection prompt

After displaying all 6 branches, ask:

> Which branches resonate with your vision? You can:
> - **Accept all 6** — proceed with the full concept tree
> - **Select branches** — name the numbers (e.g., "1, 3, 5") to narrow focus
> - **Refine** — describe what's missing and I'll regenerate
>
> When ready, proceed to `/zeespec-interrogate` with the selected branches.

### Step 4 — Handoff

Pass the selected branches (or all 6) as context to the next stage. When called from `/start-business-build`, the orchestrator captures this output automatically and passes it forward to Stage 2.

## Error Handling

| Failure | Response |
|---------|----------|
| `surreal-memory` unreachable | Emit a structured fallback: manually brainstorm 6 branches from the concept text; mark output as `[fallback — surreal-memory unavailable]` |
| MCP returns < 6 branches | Accept the partial result; label missing branches as `[pending]` and note count |
| Concept text is empty | Ask the user: "What is the one-line business concept you want to explore?" |

## Example Session

```
User: /ideation-mindmap track competitor pricing changes in real-time

Concept: "track competitor pricing changes in real-time"

Branch 1 — Data Acquisition
  • Web scraping scheduled crawlers (Playwright / Puppeteer)
  • Retailer API integrations (where available)
  • Third-party price intelligence feeds (PriceSpider, Wiser)

Branch 2 — Change Detection
  • Delta comparison engine (previous vs current snapshot)
  • Threshold-based alerting (>5% change triggers)
  • Historical trend indexing for anomaly detection

Branch 3 — Storage & State
  • Time-series database for price history (TimescaleDB / InfluxDB)
  • Product catalog normalization and deduplication
  • Snapshot versioning with audit trail

Branch 4 — Alerting & Delivery
  • Real-time webhook push to Slack / email / PagerDuty
  • Digest reports (daily / weekly) for non-urgent updates
  • Dashboard with competitor comparison views

Branch 5 — Business Rules
  • Competitor selection and weighting logic
  • Price floor / ceiling rules to filter noise
  • Segment-level overrides (geographic, SKU category)

Branch 6 — Deployment & Scale
  • SaaS multi-tenant architecture vs single-tenant
  • Rate limiting and anti-bot evasion strategy
  • Cost model: crawl frequency × product catalog size

Which branches resonate with your vision? ...
```
