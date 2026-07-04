# Tasks — change-dsg-003-release-workflow

- [ ] Run `gh run list --repo GQAdonis/disk-space-guardian --workflow=release.yml --limit 3` to confirm trigger
- [ ] Verify workflow YAML has no syntax errors (`gh workflow view release.yml --repo GQAdonis/disk-space-guardian`)
- [ ] Update `skills/process/cowork-management/references/COMMANDS.md` with Path B artifact URL format note
- [ ] Commit doc update to skill-pack worktree
