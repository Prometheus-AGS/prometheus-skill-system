# change-dsg-002-push-tag

**Status**: pending

## Summary

Add `release.yml` GitHub Actions workflow to the dsg repo, then push all 5
local unpushed commits plus the new workflow commit to `origin/main`, and tag
`v0.1.0`. The workflow must be committed before the tag push so GitHub CI
fires on the tag event.

## Motivation

Five feature commits (ecosystem detectors, scanner, safety, Cargo scaffold,
capability specs) exist only locally. Until pushed, the submodule pointer
cannot be advanced and `install-binaries.sh` Path B (GitHub Releases download)
is unreachable. Adding `release.yml` now ensures cross-platform binaries are
published on the first real tag.

## Design

### Files to change

| File | Repo | Action |
|------|------|--------|
| `.github/workflows/release.yml` | disk-space-guardian | Create |

### release.yml targets

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-musl`
- `x86_64-pc-windows-msvc`

Trigger: `push` on tags matching `v*`.

### Command sequence

```bash
# In /Users/gqadonis/Projects/prometheus/disk-space-guardian
git add .github/workflows/release.yml
git commit -m "ci: add release workflow for cross-platform binary builds"
git push origin main
git tag -a v0.1.0 -m "dsg v0.1.0 — initial public release"
git push origin v0.1.0
```

## Acceptance Criteria

- `git log origin/main` shows all 6 commits (5 existing + release.yml commit)
- `git tag --list v0.1.0` on the remote returns the tag SHA
- `.github/workflows/release.yml` exists in the pushed repo
