---
id: change-credibility-011-submodule-https
title: Change artifact-refiner submodule URL from SSH to HTTPS
phase: phase-credibility-closure
priority: P2
effort: XS
wave: 3
parallel: true
agent: claude
status: done
gap_id: P2-D
verdict: BUILD
scope:
  - .gitmodules
---

# change-credibility-011 — Change artifact-refiner submodule URL from SSH to HTTPS

## Context

The `.gitmodules` file for `skills/imported/artifact-refiner` uses an SSH URL (`git@github.com:...`). SSH URLs require the CI runner to have a configured SSH key, which most GitHub Actions runners do not. This means `git submodule update --init --recursive` fails in CI unless the job does special SSH key setup.

HTTPS URLs work with `actions/checkout` `submodules: recursive` out of the box using the `GITHUB_TOKEN`.

## Scope

Change the `url = git@github.com:...` line in `.gitmodules` to `url = https://github.com/...` for the `artifact-refiner` submodule.

Also check and update `skills/imported/sycophancy-correction` if it uses SSH.

## Implementation Notes

Current `.gitmodules` likely has:
```
[submodule "skills/imported/artifact-refiner"]
    path = skills/imported/artifact-refiner
    url = git@github.com:Prometheus-AGS/artifact-refiner-skill.git
```

Change to:
```
[submodule "skills/imported/artifact-refiner"]
    path = skills/imported/artifact-refiner
    url = https://github.com/Prometheus-AGS/artifact-refiner-skill.git
```

Then run:
```bash
git submodule sync
git submodule update --init --recursive
```

To verify the updated URL is used.

## Verification

- `.gitmodules` has no `git@github.com` URLs
- `git submodule sync && git submodule update --init --recursive` succeeds without SSH key
- CI job with `actions/checkout@v4 submodules: recursive` passes
