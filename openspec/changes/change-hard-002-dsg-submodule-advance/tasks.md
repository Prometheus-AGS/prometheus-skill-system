# Tasks — change-hard-002-dsg-submodule-advance

- [x] `cd tools/disk-space-guardian && git fetch origin && git checkout abe2e1c`
- [x] `cd ../.. && git add tools/disk-space-guardian`
- [x] Verify `git submodule status tools/disk-space-guardian` shows `abe2e1c (v0.1.4)`
- [x] Optional: add `# Requires dsg v0.1.4+` note to SKILL.md install section
- [x] Run `npm run validate:strict skills/devops/disk-space-guardian` — confirm 0 errors
- [x] Commit: `chore: advance disk-space-guardian submodule to v0.1.4`

## Notes

Commit: 96c1786
validate:strict: 0 errors, 0 warnings
