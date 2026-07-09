# Proposal — change-polish-002-static-install-step

## Phase
phase-prui-polish

## Goal
G-02: Replace the git-tracked `docs/deep-research/static` symlink with real
files and add a `cp -r` step to `scripts/install-binaries.sh`.

## Summary
1. Remove `docs/deep-research/static` from git (it is tracked as mode 120000 symlink).
2. Copy `substrate/prometheus-research/src/static/` to `docs/deep-research/static/` and commit the real files.
3. Add a `cp -r` step in the `prometheus-research` block of `scripts/install-binaries.sh`
   so that the docs static dir is refreshed on every binary install run.

## Acceptance Criteria
- [ ] `git ls-files --stage docs/deep-research/static` returns regular file entries (mode 100644), not a symlink (120000)
- [ ] `docs/deep-research/static/htmx.min.js` exists and is a regular file in git
- [ ] `docs/deep-research/static/alpine.min.js` exists and is a regular file in git
- [ ] `docs/deep-research/static/hls.min.js` exists and is a regular file in git (after polish-001 runs, or the stub is acceptable here)
- [ ] `scripts/install-binaries.sh` contains a `cp -r` of static assets after the binary install block (around line 308)
- [ ] `bash scripts/install-binaries.sh` dry-run (or partial) does not error on the new cp step

## Files Changed
- `docs/deep-research/static` — git-untracked symlink → replaced with real directory + files committed
- `scripts/install-binaries.sh` — new `cp -r src/static docs/deep-research/static` step added after line 308

## Implementation Notes
Steps:
```bash
git rm docs/deep-research/static          # remove symlink from git index
cp -r substrate/prometheus-research/src/static docs/deep-research/static
git add docs/deep-research/static/
```
In `install-binaries.sh`, after line 308 (`ok "prometheus-research → ${BIN_DIR}/..."`):
```bash
STATIC_SRC="${REPO_ROOT}/substrate/prometheus-research/src/static"
STATIC_DST="${REPO_ROOT}/docs/deep-research/static"
if [ -d "${STATIC_SRC}" ]; then
    cp -r "${STATIC_SRC}/." "${STATIC_DST}/"
    ok "prometheus-research static assets → docs/deep-research/static/"
fi
```
