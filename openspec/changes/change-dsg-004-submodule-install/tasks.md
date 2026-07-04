# Tasks — change-dsg-004-submodule-install

- [ ] Fetch tags in the dsg submodule: `git -C tools/disk-space-guardian fetch --tags`
- [ ] Checkout `v0.1.0` in the submodule: `git -C tools/disk-space-guardian checkout v0.1.0`
- [ ] Stage and commit the pointer advance in the skill-pack worktree
- [ ] Build dsg locally: `cargo build --release --manifest-path tools/disk-space-guardian/dsg/Cargo.toml`
- [ ] Copy binary to PATH: `cp ... ~/.local/bin/dsg && chmod +x ~/.local/bin/dsg`
- [ ] Verify: `dsg --version` returns `0.1.0`
