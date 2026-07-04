# Tasks — change-dsg-003-release-workflow

- [x] Run `gh run list --repo GQAdonis/disk-space-guardian --workflow=release.yml --limit 3` to confirm trigger
- [x] Verify workflow YAML has no syntax errors (`gh workflow view release.yml --repo GQAdonis/disk-space-guardian`)
- [x] Update `skills/process/cowork-management/references/COMMANDS.md` with Path B artifact URL format note
- [x] Commit doc update to skill-pack worktree
