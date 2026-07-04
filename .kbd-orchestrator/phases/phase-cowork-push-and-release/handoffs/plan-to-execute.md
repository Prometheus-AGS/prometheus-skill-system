---
stage: plan
from: plan
to: execute
phase: phase-cowork-push-and-release
timestamp: 2026-07-04T22:00:00Z
artifacts:
  - .kbd-orchestrator/phases/phase-cowork-push-and-release/plan.md
---

# Handoff: Plan → Execute

3 sequential changes ordered by dependency (push before pointer advance, pointer
advance before smoke test). Apply change-push-001 first: bump Cargo.toml to 0.2.0,
push all 10+1 commits to origin main, tag v0.2.0, and wait for CI. Then
change-push-002: fetch tags, checkout v0.2.0 in the submodule, stage and commit
the pointer advance in the skill-pack worktree. Finally change-push-003: add the
"Updating the Skill Pack" docs section to COMMANDS.md and run the end-to-end
smoke test confirming cowork --version returns 0.2.0 and pack/toolchain subcommands
are live. OQ-03 (release.yml secrets) must be confirmed before tagging — if CI
secrets are absent, the binary builds will fail silently.
