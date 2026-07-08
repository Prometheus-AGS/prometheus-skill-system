---
id: change-drs-005-references-hooks-agents
title: Write 9 references + 4 hooks + 4 agents for deep-research skill
phase: phase-deep-research-skill
priority: P1
effort: L
wave: 2
agent: general-purpose
status: pending
gap_id: G-04
verdict: BUILD
depends_on: change-drs-003-stage-sub-skills
scope:
  - skills/research/deep-research/references/pipeline-architecture.md
  - skills/research/deep-research/references/okf-research-format.md
  - skills/research/deep-research/references/model-routing.md
  - skills/research/deep-research/references/surreal-memory-integration.md
  - skills/research/deep-research/references/sycophancy-correction-integration.md
  - skills/research/deep-research/references/feynman-quality-gate.md
  - skills/research/deep-research/references/contradiction-resolution-guide.md
  - skills/research/deep-research/references/citation-formats.md
  - skills/research/deep-research/references/research-package-spec.md
  - skills/research/deep-research/hooks/pre-research.sh
  - skills/research/deep-research/hooks/post-stage.sh
  - skills/research/deep-research/hooks/on-contradiction.sh
  - skills/research/deep-research/hooks/post-export.sh
  - skills/research/deep-research/agents/research-planner.md
  - skills/research/deep-research/agents/source-verifier.md
  - skills/research/deep-research/agents/contradiction-resolver.md
  - skills/research/deep-research/agents/report-synthesizer.md
---

# change-drs-005 — References, Hooks, Agents

## Context

References provide lazy-loaded deep documentation. Hooks are shell scripts fired
at stage boundaries. Agents are subagent descriptors for split-role tasks.

## References (9 files, all markdown)

### `pipeline-architecture.md`
Full DAG diagram (mermaid), stage descriptions, token budget by stage, retry
policy, sequential execution contract, future threaded-execution feature flag.

### `okf-research-format.md`
OKF v0.1 base spec summary + Prometheus research extensions:
- Required: `type: research-report`, `title`, `date`, `confidence`
- Prometheus extensions: `verification_status`, `research_stage`, `sources_count`,
  `feynman_grade`, `contradictions_resolved`

### `model-routing.md`
Tier mapping table (frontier/medium/small → model IDs via liter-llm-bridge),
environment variables, fallback behavior when tier is unavailable.

### `surreal-memory-integration.md`
How to call `create_entity`, `create_relation`, `add_memory`, `semantic_search`
from within a research stage. Entity types: `ResearchSource`, `Claim`, `Topic`,
`Entity`. Relation types: `cites`, `contradicts`, `supports`, `related_to`.

### `sycophancy-correction-integration.md`
How Stage 5 (verify) invokes `detect_sycophancy` to check source framing.
Threshold values, severity levels, rejection criteria.

### `feynman-quality-gate.md`
How Stage 9 (report) routes the draft report through `learn-grade` for
pedagogical quality. Grade ≥ 0.7 AND `misconceptions_absent == 1.0` required.
Fallback when learn-grade is unavailable.

### `contradiction-resolution-guide.md`
Step-by-step resolution strategies:
1. Source authority resolution
2. Recency-favored resolution
3. Consensus resolution (majority claim wins)
4. Escalation to pmpo-elicit

### `citation-formats.md`
Supported citation styles: APA 7, MLA 9, Chicago 17, IEEE, Vancouver.
Default: APA 7. How to switch via `RESEARCH_CITATION_STYLE` env var.
Citation object schema with all required fields.

### `research-package-spec.md`
Full `.research` package directory specification:
```
<job_id>/
  manifest.json      # OKF-extended metadata
  index.md           # Human-readable entry point
  sources/           # Raw collected sources (JSON per source)
  graph.json         # Knowledge graph export
  citations.json     # Citation list with confidence scores
  contradictions.json  # Unresolved or resolved contradictions
  report.md          # Final OKF report
```

## Hooks (4 scripts, all executable)

### `pre-research.sh`
Fired before Stage 1. Validates: `QUERY` non-empty, `DEPTH` is one of
shallow/deep/exhaustive, at least one search tool available (TAVILY_API_KEY
or FIRECRAWL_API_KEY). Exits non-zero to block research if validation fails.

### `post-stage.sh`
Fired after each stage completes. Receives `STAGE_NUM` and `STAGE_NAME`
as args. Logs stage completion to `~/.prometheus/research-progress.log`.
Exits 0 (never blocks).

### `on-contradiction.sh`
Fired when detect-contradictions finds unresolved contradictions. Receives
`CONTRADICTIONS_JSON` as stdin. Logs to `~/.prometheus/research-contradictions.log`.
If `RESEARCH_AUTO_ESCALATE=1`, calls pmpo-elicit-checkpoint.sh.

### `post-export.sh`
Fired after Stage 10 export. Receives `PACKAGE_PATH`. Optionally ingests
to surreal-memory palace if `RESEARCH_AUTO_INGEST=1`. Logs export location.

## Agents (4 markdown descriptor files)

### `research-planner.md`
Subagent descriptor: frontier-class agent for Stage 1. Takes `QUERY` and
`CONTEXT` as inputs. Returns structured research plan with sub-questions,
search strategy, and token budget estimate. Prompted to think adversarially
about query ambiguity.

### `source-verifier.md`
Subagent descriptor: frontier-class agent for Stage 5. Takes batch of
sources. Returns credibility scores with reasoning per source. Prompted to
check for circular citation, low-authority domains, and stale data.

### `contradiction-resolver.md`
Subagent descriptor: frontier-class agent for Stage 6. Takes contradiction
pairs. Returns resolution with chosen position, confidence, and strategy.
Knows when to escalate to pmpo-elicit vs. resolve autonomously.

### `report-synthesizer.md`
Subagent descriptor: frontier-class agent for Stage 9. Takes graph JSON +
citations + resolved contradictions. Returns OKF-compliant report with
executive summary, findings, evidence table, and recommendations.

## Acceptance Criteria

- [ ] All 9 reference files exist in `skills/research/deep-research/references/`
- [ ] All 4 hook scripts exist in `skills/research/deep-research/hooks/`
- [ ] All 4 hooks are executable (chmod +x)
- [ ] All 4 agent descriptors exist in `skills/research/deep-research/agents/`
- [ ] No reference file exceeds 300 lines
