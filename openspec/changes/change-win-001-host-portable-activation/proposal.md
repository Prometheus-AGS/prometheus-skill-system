## Why

The pack cannot activate on a Windows host, and the Windows software it is meant to help build — Tauri installers and Flutter desktop applications — cannot be produced from a Linux host at all. Flutter's build commands are host-gated in `flutter_tools`; Tauri's MSI bundler and `signtool` path are compiled out on non-Windows hosts. Driving a Windows toolchain across the WSL filesystem boundary fails independently on cross-volume plugin symlinks, MSBuild UNC path folding, and Cargo target-directory locking, so the pack must run natively on each host rather than reach across a boundary.

Three activation invariants block that today. The signing-key gate asserts mode `0600`, which libuv can never report on Windows because it derives `st_mode` from `FILE_ATTRIBUTE_READONLY` alone. The bundle index is created with an untyped `fs.symlinkSync` and replaced by rename, neither of which works on Windows. The canonical manifest records a full permission mode per entry, so the generation hash is host-dependent — and, because the recorded mode is umask-dependent, already drifts between Linux hosts.

## What Changes

- Add a filesystem capability probe for symlink, junction, hardlink, and executable-bit support, executed in the store root and cached, replacing all inference from `process.platform`.
- Replace recorded permission modes in the canonical manifest with an entry type and a single normalized executable bit; hash symlink entries by recorded target text so a materialized copy does not alter identity.
- Replace the mode-`0600` key gate with a platform-dispatched owner-only predicate: mode and owner on POSIX, owner SID plus protected DACL with a restricted trustee set on Windows.
- Make directory indirection descend symlink, then junction, then copy, and make the authoritative activation pointer a file swapped by rename rather than a link replaced by rename.
- Emit hook configuration in exec form with an argument vector and a real executable, and move hot-path hook logic into the `prometheus-exec` binary.
- Extend installed-surface verification to assert identical bundle identity across every supported host.

## Capabilities

### New Capabilities

- `host-capability-probing`: Empirical detection and caching of filesystem primitives, with a recorded materialization strategy and explicit degradation record.
- `portable-generation-identity`: Host-independent canonical manifest and bundle identity over entry type, normalized executable bit, content hash, and recorded link targets.
- `owner-restricted-key-protection`: A single owner-only access guarantee for private key material, asserted through platform-appropriate mechanisms with reported, never applied, remediation.
- `shell-free-hook-dispatch`: Hook invocation through an argument vector and a real executable, with no host shell interpreting substituted paths.

### Modified Capabilities

- `installed-surface-verification`: Verification of an installed fix extends from the authoring host to every supported host, with bundle identity equality as the cross-host assertion.

## Impact

- Modifies `scripts/install-plugin-generation.js`: capability probe, manifest v2 emission, key predicate dispatch, typed link ladder, pointer-file activation.
- Modifies `shared/scripts/hook-runtime-v1.sh` and `shared/scripts/bootstrap-hook-runtime.sh`: pointer-file resolution, removal of `-x` gating, verbatim-path normalization on canonicalized paths.
- Modifies `hooks/hooks.json` generation to exec form and re-pins the bundle identity once.
- Extends `crates/prometheus-exec` with hook dispatch subcommands and a Windows security-descriptor inspector; adds prebuilt per-target binaries to the payload because plugin install runs with `--ignore-scripts`.
- Adds a four-leg CI matrix covering Windows with Developer Mode both enabled and disabled.
- Does not change sandboxing tiers. Windows Tier P remains unavailable; this change governs activation only.
- Does not migrate cold-path shell scripts, which stay shell and are gated on probed shell availability.
