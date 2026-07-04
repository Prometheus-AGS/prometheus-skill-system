# change-dib-001-release-archive-format

**Status**: done

## Summary

Fix `release.yml` to produce versioned tar.gz archives matching the download
URL format expected by `install-binaries.sh` Path B. Push a `v0.1.1` patch
tag to trigger the fixed workflow.

## Root Cause

`install-binaries.sh` constructs:
```
dsg-${version}-${target}.tar.gz
# e.g. dsg-0.1.1-aarch64-apple-darwin.tar.gz
```

Current `release.yml` uploads bare binaries:
```
dsg-aarch64-apple-darwin   (no version prefix, no .tar.gz)
```

## Fix

In the `Rename binary` step, tar the binary into a versioned archive:

```yaml
- name: Package binary
  shell: bash
  run: |
    VERSION="${GITHUB_REF_NAME#v}"
    ARCHIVE="dsg-${VERSION}-${{ matrix.target }}.tar.gz"
    tar -czf "${ARCHIVE}" -C "$(dirname "$SRC")" "$(basename "$SRC")"
    echo "ARCHIVE=${ARCHIVE}" >> $GITHUB_ENV
```

Then upload `${{ env.ARCHIVE }}` instead of `${{ matrix.artifact }}`.

## Acceptance Criteria

- `release.yml` produces archives named `dsg-<semver>-<target>.tar.gz`
- `v0.1.1` tag pushed to `GQAdonis/disk-space-guardian`
- GitHub Actions release run triggered for `v0.1.1`
- Submodule pointer in skill-pack advanced to `v0.1.1`
