## 1. Implementation

- [ ] 1.1 Land c400-c404 to main
- [ ] 1.2 `git checkout main && git pull --ff-only`
- [ ] 1.3 `git push origin main` — a fast-forward pull moves nothing outward
- [ ] 1.4 `bash scripts/update-skill-pack.sh --force`

## 2. Verification

- [ ] 2.1 Working tree clean; update-skill-pack.sh --force exits 0
- [ ] 2.2 `grep -c resolver_missing ~/.claude/skills/adversarial-review/scripts/preflight-models.sh` -> 1
- [ ] 2.3 With CLAUDE_PLUGIN_ROOT UNSET, a run through ~/.claude/skills/... reports status: ok — the criterion that reproduces the original failure
- [ ] 2.4 `git ls-remote origin main` resolves to a commit containing c400-c404
