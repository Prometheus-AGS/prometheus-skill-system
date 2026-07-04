---
stage: plan
from: plan
to: execute
phase: phase-dsg-cli-foundation
timestamp: 2026-07-04T00:05:00Z
artifacts:
  - .kbd-orchestrator/phases/phase-dsg-cli-foundation/plan.md
  - openspec/changes/change-dsg-002-push-tag/proposal.md
  - openspec/changes/change-dsg-003-release-workflow/proposal.md
  - openspec/changes/change-dsg-004-submodule-install/proposal.md
---

# Handoff: Plan → Execute

3 sequential changes. Start with change-dsg-002-push-tag: author
`.github/workflows/release.yml` in the dsg repo, commit it, push all 6
commits to origin/main, then tag v0.1.0. Change-dsg-003 verifies CI
triggered and documents Path B URL format. Change-dsg-004 advances the
submodule pointer and installs dsg to ~/.local/bin/dsg. Apply in order —
each change depends on the prior one completing.
