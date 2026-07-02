---
id: change-goal-013
title: Goal-Time Skill/MCP Discovery
phase: goal-loop-support
subphase: B (integration)
depends_on: [change-goal-002]
agent: claude-code
status: done
scope:
  - scripts/kbd-goal-discover.sh
  - skills/process/kbd-goal/references/skill-discovery.md
  - kbd-goal/SKILL.md
---

# change-goal-013 — Goal-Time Skill/MCP Discovery

## Problem

When a goal is stated ("build a weekly standup generator in Go"), the relevant skills (`golang-patterns`, `golang-testing`) and MCP servers (Context7 for Go docs) are not automatically identified. Users must manually pre-load them, causing friction and missed capabilities.

## Solution

Build `kbd-goal-discover.sh` that does keyword matching on the goal description against a mapping table (`skill-discovery.md`) and outputs recommended skills + MCP servers as an advisory list printed at goal start.

## Files

- `scripts/kbd-goal-discover.sh` (CREATE)
- `skills/process/kbd-goal/references/skill-discovery.md` (CREATE)

## Tasks

- Write `skill-discovery.md`: mapping table of keywords → skills + MCPs:
  - Go / Golang → golang-patterns, golang-testing + context7
  - Rust → rust-patterns, rust-testing, prometheus-rust-auditor + context7
  - React / TypeScript → react-vite-stack, typescript-reviewer + context7
  - Python → python-patterns, python-testing + context7
  - API / REST → api-design, backend-patterns
  - Database / SQL → database-migrations, postgres-patterns
  - Docker / K8s → docker-patterns, deployment-patterns + kubernetes MCP
  - Testing → tdd-workflow, bdd-testing
- Write `kbd-goal-discover.sh`: grep goal description against keyword table; output JSON with `recommended_skills[]`, `recommended_mcps[]`, `rationale`
- Integrate into `kbd-goal-start.sh`: call discover at start, print advisory block to user (non-blocking)
- Update `kbd-goal/SKILL.md` Start section with discovery advisory documentation

## Tasks

- [x] 1. Write `skill-discovery.md`: mapping table of keywords → skills + MCPs:
- [x] 2. Go / Golang → golang-patterns, golang-testing + context7
- [x] 3. Rust → rust-patterns, rust-testing, prometheus-rust-auditor + context7
- [x] 4. React / TypeScript → react-vite-stack, typescript-reviewer + context7
- [x] 5. Python → python-patterns, python-testing + context7
- [x] 6. API / REST → api-design, backend-patterns
- [x] 7. Database / SQL → database-migrations, postgres-patterns
- [x] 8. Docker / K8s → docker-patterns, deployment-patterns + kubernetes MCP
- [x] 9. Testing → tdd-workflow, bdd-testing
- [x] 10. Write `kbd-goal-discover.sh`: grep goal description against keyword table; output JSON with `recommended_skills[]`, `recommended_mcps[]`, `rationale`
- [x] 11. Integrate into `kbd-goal-start.sh`: call discover at start, print advisory block to user (non-blocking)
- [x] 12. Update `kbd-goal/SKILL.md` Start section with discovery advisory documentation
