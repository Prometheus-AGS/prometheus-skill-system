---
stage: plan
from: plan
to: execute
phase: phase-dsg-install-binaries-fix
timestamp: 2026-07-04T01:15:00Z
artifacts:
  - .kbd-orchestrator/phases/phase-dsg-install-binaries-fix/plan.md
  - openspec/changes/change-dib-001-release-archive-format/proposal.md
  - openspec/changes/change-dib-002-e2e-verify/proposal.md
---

# Handoff: Plan → Execute

2 sequential changes. Start with change-dib-001: update release.yml to
produce versioned tar.gz archives (dsg-<ver>-<target>.tar.gz), commit,
push to origin/main, tag v0.1.1, advance submodule. Then change-dib-002:
wait for CI to complete, verify 4 artifacts published, run install script
end-to-end.
