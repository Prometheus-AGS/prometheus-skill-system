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

### Step 1 — Generate at least 3 candidate sets, INDEPENDENTLY

Invoke the `generate_ideation_mindmap` tool from the `surreal-memory` MCP server
**at least three separate times**. Each call is its own dispatch and must receive
**only the topic** — never another call's output, and never a summary of it.

```
# Call 1
generate_ideation_mindmap(topic: "<the one-line concept from $ARGUMENTS>", branches: 6)
# Call 2 — same topic, fresh dispatch, no knowledge of call 1
generate_ideation_mindmap(topic: "<the one-line concept from $ARGUMENTS>", branches: 6)
# Call 3 — same topic, fresh dispatch, no knowledge of calls 1 or 2
generate_ideation_mindmap(topic: "<the one-line concept from $ARGUMENTS>", branches: 6)
```

Only **after** all sets exist do you pool them (Step 2).

#### Why independence is mechanical, not a prompting style

A single call, or a chain where each call sees the last, is the failure mode this
step exists to avoid — and it is the shape this skill originally had.

- **Multi-agent LLM ideation collapses toward agreement.** Chen et al. (2026),
  *Diversity Collapse in Multi-Agent LLM Systems* (arXiv 2604.18005): agents
  exhibit structural coupling and produce redundant ideas **despite architectural
  attempts to diversify**. Telling personas to disagree does not work.
- **Interacting groups underperform independent ones.** Mullen, Johnson & Salas
  (1991), 20 studies / 800+ teams: interactive brainstorming groups are
  significantly *less* productive than nominal groups in both quantity and
  quality, and the gap **grows with group size** (production blocking).

The evidence-backed structure is therefore **independent generation → pool →
judge**, which is what Steps 1–3 implement. Do not add a round-table, a debate
round, or a "have the agents critique each other" step: both findings say that
subtracts value while appearing to add it.

> **Auditable, not merely instructed.** Independence is asserted by inspecting
> what each dispatch received, not by reading this prose — see
> `scripts/assert-independent-dispatch.sh`.

### Step 2 — Pool the independent sets, then format

Now — and only now — combine the sets from Step 1.

1. **Merge** all branches from all sets into one pool.
2. **Collapse near-duplicates.** Branches expressing the same idea in different
   words become one entry, and record how many independent sets produced it.
3. **Keep the singletons.** A branch that appeared in only one set is *not* noise
   to be filtered — independent generation exists precisely to surface ideas a
   single pass would miss. Convergence is a useful signal; it is not a ranking.

Annotate each pooled branch with its independent-set count:

```
Branch 3 — <Branch Name>   [3/3 sets]     ← all three converged
Branch 7 — <Branch Name>   [1/3 sets]     ← only one set found this
```

That count is **evidence for the judge**, not a score. A `1/3` branch may be the
best idea in the pool; a `3/3` branch may just be the obvious one. Do not rank by
convergence, and do not drop low-count branches before Step 3.

Present the pooled result as a structured list:

```
Concept: "<original concept>"

Branch 1 — <Branch Name>
  • <concept cluster point 1>
  • <concept cluster point 2>
  • <concept cluster point 3>

Branch 2 — <Branch Name>
  • ...

[... repeat for every pooled branch]
```

Keep each branch to 3–5 sub-bullets. If the MCP result contains more detail, summarize to the most actionable points.

### Step 3 — Score the pool with a critic that did not generate it

Before asking the user to choose, verify independence and hand the pooled
branches to a separate scorer.

```bash
# 1. Independence is a property of what each dispatch RECEIVED, so assert it
#    against the recorded inputs — never by re-reading Step 1's instructions.
bash scripts/assert-independent-dispatch.sh --session "$SESSION" || exit 2
```

```
# 2. Score with a critic on its OWN dispatch.
Task(subagent_type="kbd-idea-critic", prompt=<the pooled branches from Step 2>)
```

The generator must never score its own branches. `kbd-idea-critic` exists for
exactly this and says so: *"the idea that proposed the idea should never also
grade it."* This is the same producer≠judge rule enforced everywhere else in this
pack — an agent that just argued for an idea is the worst-placed one to judge it.

Present the critic's aggregate alongside each branch, **with its
independent-set count from Step 2**:

```
Branch 3 — <Name>   [3/3 sets]  critic 8.2
Branch 7 — <Name>   [1/3 sets]  critic 9.1   ← one set found it; scored highest
```

That pairing is the point. Convergence and quality are different signals, and a
branch only one set produced can still be the best idea in the pool.

> If the critic is unavailable, **do not self-score**. Present the branches with
> `critic: UNAVAILABLE` and say plainly that they are unscored. A self-assigned
> score carries the appearance of review without the substance.

### Step 4 — Present selection prompt

After displaying all branches with their counts and scores, ask:

> Which branches resonate with your vision? You can:
> - **Accept all** — proceed with the full pooled concept tree
> - **Select branches** — name the numbers (e.g., "1, 3, 5") to narrow focus
> - **Refine** — describe what's missing and I'll regenerate
>
> When ready, proceed to `/zeespec-interrogate` with the selected branches.

### Step 5 — Handoff

Pass the selected branches (or all of them) as context to the next stage. When called from `/start-business-build`, the orchestrator captures this output automatically and passes it forward to Stage 2.

## Error Handling

| Failure | Response |
|---------|----------|
| `surreal-memory` unreachable | Emit a structured fallback: manually brainstorm 6 branches from the concept text; mark output as `[fallback — surreal-memory unavailable]` |
| A generation call returns < 6 branches | Accept the partial set; the pool draws on the other independent sets. Note the count. |
| Fewer than 3 sets recorded | `assert-independent-dispatch.sh` REJECTS. Do not pool: one or two samples is the single-pass case this design replaces. |
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

## Presenting to the user

This flow never renders its own prompts. Every user-facing question goes through
`scripts/emit-ui-intent.sh`, which emits a `UiIntent` and lets `ui-surface`
resolve the tier:

```bash
bash scripts/emit-ui-intent.sh \
  --title "Which idea to build?" \
  --body  "Three survived scoring." \
  --option "Standup generator" --option "PR summariser"
```

Exit `3` means the harness never answered — a stated limit, not delivery. Degrade
to Tier 0 text rather than reporting the question as delivered.

Verified tiers, stated limits (`zed` is Tier 0; only `codex` was exercised among
file-pair harnesses), and the round-trip evidence are in
[references/harness-delivery.md](references/harness-delivery.md).
