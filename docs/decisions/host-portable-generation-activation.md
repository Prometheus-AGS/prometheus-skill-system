# Decision: derive generation identity and activation primitives from probed host capability

**Status:** proposed · 2026-09-01 · release 1.9.0

## Context

[Immutable plugin activation](immutable-plugin-activation.md) verifies file modes as part of manifest identity, and [signed transactional plugin generations](signed-transactional-plugin-generations.md) protects the signing identity with mode `0600`. Both encode POSIX permission semantics into invariants that Windows cannot satisfy rather than merely relax. libuv derives `st_mode` on Windows solely from `FILE_ATTRIBUTE_READONLY`, so a stat returns `0444` or `0666` and never `0600`, and `chmod` toggles only the read-only attribute. Recorded modes therefore make the generation hash host-dependent, and the key-protection gate fails permanently on the second install. The gate is also umask-dependent on POSIX, so identical payloads already produce different bundle identities across Linux hosts.

Windows software targets — Tauri installers and Flutter desktop builds — cannot be produced from a Linux host, and driving a Windows toolchain across the WSL filesystem boundary fails on cross-volume symlinks, UNC path folding, and target-directory locks. The pack must therefore activate natively on Windows, not reach into it.

## Decision

Generation identity records file type and a single normalized executable bit, and excludes modes, timestamps, ownership, and platform attributes. Symlink entries hash their recorded target text, so an entry materialized as a copy on a host without link support leaves the identity unchanged. The identity is canonicalized under RFC 8785, matching the receipt signing already used by `prometheus-exec`.

The installer probes symlink, junction, hardlink, and executable-bit support in the store root before materializing, caches the result, and records the strategy and any degradations in an unhashed sibling to the manifest. Directory indirection descends symlink, then junction, then copy; a junction satisfies both `isSymbolicLink()` and `test -L` and needs no elevation, so Developer Mode is never a prerequisite. The authoritative activation pointer is a file swapped by rename, because rename over an existing directory link is not atomic on Windows; the link remains as a resolved convenience.

Key protection is a platform-dispatched predicate over the same guarantee — owner-only access. POSIX asserts mode `0600` and owner identity. Windows asserts owner SID equality, a protected DACL, and non-inherited trustees restricted to the owner, `S-1-5-18`, and `S-1-5-32-544`. Remediation is reported, never applied silently.

Hooks are emitted in exec form with an argument vector and a real executable, so no shell interprets a path on any host. Hot-path hook logic moves into the `prometheus-exec` binary; cold-path scripts remain shell and are gated on probed shell availability.

## Alternatives considered

- **Require WSL 2 on Windows hosts:** rejected because Flutter cannot build Windows targets from a Linux host at all, and Windows toolchains fail against Linux-side paths for three independent reasons.
- **Relax the key-protection gate on Windows:** rejected because it removes the guarantee rather than expressing it; the DACL predicate is the equivalent assertion, not a weaker one.
- **Require Developer Mode for symlinks:** rejected because enabling it needs administrator access and can be disabled by policy; junctions carry the same semantics unprivileged.
- **Emit PowerShell counterparts for every hook script:** rejected because it doubles the surface and adds interpreter startup to every `PreToolUse` dispatch.
- **Keep shell-form hooks and normalize paths:** rejected because the failure is in the host shell's interpretation of the substituted string, which the pack does not control.

## Consequences

The manifest schema version increments and every bundle identity changes once; `hooks.json` is re-pinned in the same change. Verification gains a cross-host assertion — identical bundle identity on Linux, macOS, and Windows — that no single-host test can provide. Windows Tier P remains unavailable per [two-tier execution sandboxing](two-tier-execution-sandboxing.md); this decision governs activation, not sandboxing. The pack ships prebuilt dispatcher binaries per target because plugin install runs with `--ignore-scripts`.

## Verification

A four-leg matrix covers Linux, macOS, and Windows with Developer Mode both enabled and disabled, asserting byte-identical bundle identity across all legs. Fixtures prove the junction path satisfies both link assertions, that a degraded copy does not alter identity, that the key predicate passes for an owner-restricted file and fails for an inherited or over-granted DACL, and that exec-form dispatch carries an unmodified argument containing backslashes and `$`.
