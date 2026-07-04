# change-dsg-003-release-workflow

**Status**: done

## Summary

Verify the GitHub Actions release workflow fired on the `v0.1.0` tag. If binary
artifacts are not yet published (CI still running), document the expected
artifact names and URLs. Update the skill-pack docs to note that
`install-binaries.sh` Path B requires published release artifacts.

## Motivation

After pushing v0.1.0, the release CI may take a few minutes to build four
cross-platform targets. This change ensures we have confirmed the workflow
exists, is syntactically valid, and documents any discrepancy between what
install-binaries.sh expects and what will be published.

## Design

Verification sequence:

```bash
# Check workflow run triggered
gh run list --repo GQAdonis/disk-space-guardian --workflow=release.yml --limit 3

# Check artifact download URL format (used by install-binaries.sh Path B)
# Expected: github.com/GQAdonis/disk-space-guardian/releases/latest/download/dsg-aarch64-apple-darwin
```

If the run triggered, status is DONE regardless of whether the binary build
completes — the workflow itself is the deliverable for this change.

## Acceptance Criteria

- `gh run list` shows at least one run for `release.yml` triggered by the v0.1.0 tag
- No YAML syntax errors in the workflow file
- `skills/process/cowork-management/references/COMMANDS.md` notes the Path B artifact URL format
