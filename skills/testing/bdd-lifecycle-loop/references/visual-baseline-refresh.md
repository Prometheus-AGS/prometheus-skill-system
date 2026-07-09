# Visual Baseline Refresh Workflow

Playwright's `--update-snapshots` overwrites baseline images silently.
That's fine for local exploration but dangerous in CI — someone could
merge a visual regression and the diff would show only "baseline updated".

The refresh workflow adds a paper trail:

```
1. Change lands that alters a rendered element (intentionally or not)
2. Playwright fails: "expected screenshot mismatch"
3. Instead of running --update-snapshots locally and committing:
      a. Open `snapshot-refresh/YYYY-MM-DD-<short-description>` branch
      b. Run the refresh script (below)
      c. Push the branch → PR is opened automatically
      d. PR requires the 'visuals-approved' label to merge
      e. Merge into main; baseline images travel through code review
```

## The refresh script

`scripts/refresh-visual-baselines.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

DATE=$(date +%Y-%m-%d)
DESC="${1:?short-description required, e.g. 'header-nav'}"
BRANCH="snapshot-refresh/${DATE}-${DESC}"

git switch -c "$BRANCH"

npx playwright test --update-snapshots

# Update the manifest so reviewers can see WHICH images changed
git status --porcelain "tests/**/*.png" \
  | awk '{print $2}' \
  | while IFS= read -r file; do
      hash=$(sha256sum "$file" | cut -d' ' -f1)
      printf '%s\t%s\n' "$hash" "$file"
    done > tests/snapshots/MANIFEST.txt

git add tests/**/*.png tests/snapshots/MANIFEST.txt
git commit -m "chore(visuals): refresh baselines for ${DESC}"
git push -u origin "$BRANCH"
```

The `MANIFEST.txt` gives reviewers a checksum-per-image so a wholesale
image swap can't slip through as a small diff.

## PR requirements

Configure the PR checks (GitHub branch protection) to require:

1. Green CI (Playwright must pass with the refreshed baselines)
2. The `visuals-approved` label — added by a human after they've eyeballed
   the diffs in the PR's "Files changed" tab
3. A commit message on the merge that lists the affected screens

Without the label the PR can't merge. Baseline drift becomes visible.

## What NOT to do

- **Don't** run `--update-snapshots` on `main` directly. That defeats the
  paper trail.
- **Don't** add `git add tests/**/*.png` to a routine "fix CI" commit.
  Baseline updates deserve their own branch and their own review.
- **Don't** allow the CI job to auto-merge baseline refreshes. A human
  must confirm the visual change is intended.

## Prior art

- Chromatic (SaaS) — closest to what this workflow does; per-image
  approvals with audit log
- Percy (SaaS) — similar approval model
- Playwright's own docs — describe `--update-snapshots` but not a review
  workflow

None of the above are self-hosted; the workflow above is what you build
when you want the paper trail without adopting a SaaS.
