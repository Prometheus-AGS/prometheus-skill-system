# Tasks — change-dsg-004-submodule-install

- [x] Fetch tags in the dsg submodule: `git -C tools/disk-space-guardian fetch --tags`
- [x] Checkout `v0.1.0` in the submodule: `git -C tools/disk-space-guardian checkout v0.1.0`
- [x] Stage and commit the pointer advance in the skill-pack worktree
- [x] Build dsg locally: binary at `tools/disk-space-guardian/target/release/dsg` (workspace root, not crate subdir)
- [x] Copy binary to PATH: `cp tools/disk-space-guardian/target/release/dsg ~/.local/bin/dsg`
- [x] Verify: `dsg --version` → `dsg 0.1.0` ✓
