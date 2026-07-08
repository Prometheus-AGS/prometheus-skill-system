# Tasks: change-drs-002-parent-skill-md

- [x] Write YAML frontmatter block (name, description, license, version, allowed-tools, model_routing, triggers, metadata)
- [x] Write "When to Use" section (activation scenarios, 8-10 bullet points)
- [x] Write "Quick Start" section (/deep-research <query> invocation pattern)
- [x] Write "10-Stage Pipeline" section (table of stages + sub-skill refs)
- [x] Write ".research Package Format" section (OKF frontmatter + Prometheus extensions)
- [x] Write "Integration Guide" section (surreal-memory, liter-llm, Feynman gate)
- [x] Write "Examples" section (3 example queries with expected pipeline paths)
- [x] Write "Common Issues" section (troubleshooting)
- [x] Verify file is under 500 lines (284 lines confirmed)
- [x] Run: npm run validate:strict skills/research/deep-research — parent passes; sub-skill errors expected until change-003
