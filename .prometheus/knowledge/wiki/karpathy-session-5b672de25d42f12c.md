---
type: SessionRecord
id: karpathy-session-5b672de25d42f12c
title: Karpathy session 5b672de25d42
tags:
- karpathy
- session-learning
sources:
- resource: session:1db84095-da34-4e96-ab6a-dfa84d559c8f
generated:
  by: pk/1.11.0
  at: 2026-11-17T09:07:24.458866+00:00
created_at: 2026-11-17T09:07:24.458866+00:00
updated_at: 2026-11-17T09:07:24.458866+00:00
revision: 0
---

## Delta

I didn't change anything in the repo. I searched the web with Firecrawl and checked what the repo already has. Here are the tools that fit, grouped by what they check, from cheapest to strongest evidence.

## 1. Anthropic's own tools (behavioral evals)

| Tool | What it does | Notes |
|---|---|---|
| **skill-creator** (`anthropics/skills`, installed here as `anthropic-skills:skill-creator`) | The test-and-compare loop. It runs each eval with the skill and without it in parallel, grades the runs, and reports mean ± stddev per metric. It also does blind A/B comparison of two skill versions and has an HTML eval viewer. | Cases live in `evals/evals.json` inside each skill. Scripts: `run_eval.py`, `aggregate_benchmark.py`, `eval-viewer/generate_review.py`, `package_skill.py`, and `quick_validate.py` for structure checks. |
| **skill-creator description optimizer** (`run_loop.py` + `improve_description.py`) | Tests whether a skill triggers when it should. You give it about 20 queries, each marked "should trigger" or "should not". It holds some out as a test set and rewrites the `description` until triggering is accurate. | This targets what most likely fails across a 300-skill pack: descriptions that collide or don't trigger. |
| **`claude plugin eval`** (Claude Code CLI) | Runs cases at plugin level. Each case is a `prompt.md` plus a `graders/` folder (`tool_used: Skill`, regex, file-created, LLM judge). It runs every case with and without the plugin and reports the Δ, plus HTML and JSON reports, cost caps and mock MCP servers. | Docs: https://code.claude.com/docs/en/plugin-evals. It's in **early access**. Its case format is different from skill-creator's `evals.json`, and neither tool reads the other's files. Its `--json` output and cost caps (`--max-cost-usd`) are designed for CI, but it can run locally, which matches this repo's local-only validation rule. |
| **`/skill-doctor`** | Claude Code's built-in skill health report. | Quick static triage. |

## 2. Spec and static linters (cheap and deterministic)

- **`skills-ref validate`** (`agentskills/agentskills` repo, also on pip and npm): the official agentskills.io reference validator. It checks frontmatter and naming. Running it next to `npm run validate:strict` gives a second opinion on spec compliance.
- **agnix** (`agent-sh/agnix`): a linter and language server with 385 rules across six harnesses. That includes 26 rules for skills and cross-platform rules, plus checks for CLAUDE.md, hooks and MCP configs. It suits this pack because the pack targets several harnesses.
- **avocetta** (`korya/avocetta`): deterministic compliance checks against the spec, Claude Code and Codex.
- **skills-lint** (`pachara-n/skills-lint`, also a GitHub Action, so run it locally): finds broken references to files and scripts, missing or over-long descriptions, and duplicate names.
- **skillcheck**: scores how discoverable a description is (0–100) and flags problems between harnesses.
- **agent-skills-lint** (`swarmclawai`): a validator for skills across agents.

## 3. Outside eval harnesses (comparing versions, models or harnesses)

- **Promptfoo, "Test Agent Skills" guide**: providers for the Claude Agent SDK, Codex and OpenCode. It can compare two versions of a skill with a JSON output schema and a `skill-used` signal. This is the best fit for checking that skills behave the same on Claude Code and Codex.
- **Inspect AI** (UK AISI): Google has a codelab on evaluating agent skills with it.
- **MLflow skill testing** (`mlflow/skills`): YAML cases with a setup script, the skills to load, and Python judges such as "skill invoked" or "artifact created".
- **Braintrust** `bt` CLI and its eval skills: an experiment you can inspect for each run.
- **OpenAI, "Testing Agent Skills Systematically with Evals"**: a write-up of the method, worth reading for Codex-side cases.
- **SkillsBench** (arXiv 2602.12670): a benchmark design comparing curated skills, self-written skills and no skills. Useful as a model f

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 1db84095-da34-4e96-ab6a-dfa84d559c8f
- Captured: 2026-11-17T09:06:40.628005Z
- Project: /Volumes/T7/Projects/prometheus/prometheus-skill-pack

## Changed Paths

- No changed paths detected.
