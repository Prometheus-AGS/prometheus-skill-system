---
id: change-credibility-014-package-lock
title: Commit package-lock.json and switch CI to npm ci
phase: phase-credibility-closure
priority: P2
effort: XS
wave: 3
parallel: true
agent: claude
status: done
gap_id: P2-G
verdict: BUILD
scope:
  - .gitignore
  - package.json
  - package-lock.json
  - .github/workflows/validate.yml
---

# change-credibility-014 — Commit package-lock.json and switch CI to npm ci

## Context

The root `package-lock.json` is currently in `.gitignore` (or simply not committed). As a result, every CI run generates a fresh lockfile, meaning packages can silently drift between runs and the assessment's "28 npm advisories" may not be stable or reproducible.

Committing the lockfile and using `npm ci` (instead of `npm install`) locks the dependency tree to exactly what was last audited and makes CI deterministic.

## Scope

1. Add `!package-lock.json` exception to `.gitignore` (if it's being excluded) or simply commit the file
2. Run `npm install` locally to regenerate a clean lockfile
3. Commit `package-lock.json`
4. Change any `npm install` in `.github/workflows/validate.yml` to `npm ci`

## Implementation Notes

Check if `package-lock.json` is gitignored:
```bash
git check-ignore -v package-lock.json
```

If yes, add exception to `.gitignore`:
```gitignore
!package-lock.json
```

Generate and commit:
```bash
npm install
git add package-lock.json
```

In `.github/workflows/validate.yml` — change install step:
```yaml
- name: Install dependencies
  run: npm ci
```

Note: the Docusaurus `site/` subproject has its own `package-lock.json` at `site/package-lock.json`. That should also be committed if it is not already.

## Verification

- `package-lock.json` is committed (not listed in `.gitignore`)
- CI uses `npm ci`
- `npm ci` passes without error (lockfile is consistent with `package.json`)
