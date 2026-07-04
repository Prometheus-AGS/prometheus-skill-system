---
id: change-int-003-dsg-skill-md
title: dsg agentskills.io SKILL.md
phase: cowork-integration
priority: P1
effort: M
wave: 5
agent: general-purpose
status: done
gap_id: G-05-dsg
verdict: BUILD
scope:
  - prometheus-skill-pack (skill-pack repo)
  - skills/devops/disk-space-guardian/SKILL.md (new)
  - skills/devops/disk-space-guardian/references/SAFETY.md (new)
  - skills/devops/disk-space-guardian/references/ECOSYSTEMS.md (new)
---

# change-int-003 — dsg agentskills.io SKILL.md

## Context

The dsg CLI is implemented and the submodule is registered. An agentskills.io
SKILL.md teaches AI assistants when and how to invoke dsg, covering safety
model, ecosystem detection, activity verification, and retention policies.

## Strategy

Create skills/devops/disk-space-guardian/SKILL.md with full 8-section body
per the plan spec. Add two reference files for detailed safety and ecosystem
information. Run validate:strict to confirm compliance.

## Scope

1. Create skills/devops/disk-space-guardian/SKILL.md (agentskills.io compliant)
2. Create skills/devops/disk-space-guardian/references/SAFETY.md
3. Create skills/devops/disk-space-guardian/references/ECOSYSTEMS.md
4. Run npm run validate:strict skills/devops/disk-space-guardian
5. Update KBD orchestrator
6. Commit

## Verification

- npm run validate:strict exits 0
- SKILL.md has valid frontmatter (name, version, license, metadata.tags)
- 8 sections present: Quick Start, Safety First, Ecosystem Detection,
  Activity Verification, Retention Policies, Automation Setup,
  Knowledge Logging, Troubleshooting
