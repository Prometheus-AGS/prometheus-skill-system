# change-dsg-004-submodule-install

**Status**: done

## Summary

Advance `tools/disk-space-guardian` submodule pointer to `v0.1.0` and install
the `dsg` binary to `~/.local/bin/dsg` so `dsg --version` returns `0.1.0`
from PATH.

## Motivation

The submodule currently points to `852ab4c` (pre-feature). With the tag
pushed (change-dsg-002), the pointer can be advanced so `install-binaries.sh`
Path A builds from the correct source. Installing the binary satisfies G-01
and makes `cowork disk scan` etc. functional.

## Design

### Submodule advance

```bash
# In skill-pack worktree root
git -C tools/disk-space-guardian fetch --tags
git -C tools/disk-space-guardian checkout v0.1.0
git add tools/disk-space-guardian
git commit -m "chore(tools): advance tools/disk-space-guardian to v0.1.0"
```

### Binary install

```bash
# Build locally (Path A)
cargo build --release --manifest-path tools/disk-space-guardian/dsg/Cargo.toml
cp tools/disk-space-guardian/dsg/target/release/dsg ~/.local/bin/dsg
chmod +x ~/.local/bin/dsg

# Verify
dsg --version
```

Expected output: `dsg 0.1.0`

## Acceptance Criteria

- `git submodule status` shows pointer at v0.1.0 SHA (not pre-feature)
- `~/.local/bin/dsg` exists and is executable
- `dsg --version` returns `dsg 0.1.0` (or `disk-space-guardian 0.1.0`)
- Commit exists in skill-pack worktree advancing the pointer
