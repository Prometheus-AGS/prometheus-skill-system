## Context

The pack's activation substrate encodes three POSIX assumptions as integrity invariants. Each has an exact Windows equivalent that preserves the guarantee; none requires relaxing it. The design below records why each equivalent is the correct one and where the non-obvious failure modes are.

## Goals / Non-Goals

**Goals**

- One bundle identity for one payload, byte-identical on Linux, macOS, and Windows.
- Owner-only key protection expressed as a guarantee, asserted per platform.
- Activation and rollback that are atomic on every supported host.
- Hook dispatch that no host shell can misinterpret.

**Non-Goals**

- Windows Tier P sandboxing. It remains unavailable.
- Cross-compilation of Windows or macOS application targets. Each host builds its own.
- Migration of cold-path shell scripts. They remain shell.
- Requiring Developer Mode, elevation, or WSL on a Windows host.

## Decisions

### Identity records an executable bit, not a mode

Git normalizes tree permissions to a two-valued space, and Nix's archive format records only an `executable` flag — which is precisely why Nix store hashes are portable. The manifest adopts the same model: `t` in `{f, d, l}`, `x` as `Boolean(mode & 0o100)`, content hash, size, and for links the recorded target text. Timestamps, ownership, SIDs, file attributes, and ACLs are excluded.

Recording the link target rather than materialized bytes is what makes degradation safe: a host that must copy where a link was intended still computes the same identity, and the substitution is recorded out-of-band. The degradation record is a sibling of the manifest and is deliberately not hashed, so a portable identity and an honest local account of how it was realized can coexist.

Modes are re-applied from the manifest at materialization on POSIX (`0755` / `0644`), which also removes the existing umask dependence.

### Junctions satisfy both link assertions

The two assertions the runtime makes about the bundle index survive unchanged on Windows. libuv sets `S_IFLNK` for any reparse point when performing an lstat, so `isSymbolicLink()` reports true for a junction. The MSYS2 runtime's reparse-point check reports a junction whose substitute name is a drive-letter path as a POSIX symlink, so `test -L`, `readlink`, and `pwd -P` all behave under Git Bash. Junctions require no privilege, which is why the ladder prefers a real symlink but never depends on one.

Two mechanical constraints follow. `fs.symlinkSync` must always be called with an explicit type, because autodetection selects `dir` and raises `EPERM` without Developer Mode; and junction targets must be absolute. Replacement cannot use rename: `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING` errors when the destination is a directory, and both directory symlinks and junctions carry the directory attribute. The link is therefore unlinked and recreated under the store mutex.

### The pointer is a file

Because no atomic directory-link swap exists on Windows, the authoritative pointer becomes a small file holding the generation identity, swapped by rename — which is atomic over an existing file everywhere. This is strictly stronger than the link it replaces: a byte string can be hashed, signed, and swapped atomically. The link remains as a resolved convenience and its `-L` check degrades from an integrity gate to an advisory check.

### Key protection is a predicate, not a mode

Win32-OpenSSH solved the same problem for the same reason and its model is adopted directly: owner-only means the file's owner matches the process token, the DACL is protected so nothing is inherited, and every remaining trustee is the owner, `SYSTEM`, or `Administrators`. The last two are unavoidable — they can take ownership regardless — so excluding them buys nothing and breaks backup and antivirus.

Trustees are compared as SIDs. Name comparison fails on non-English Windows and is the most common defect in hand-rolled versions of this check. Remediation is printed as an `icacls` invocation and never executed: silently repairing a key's ACL would mask a real compromise.

Node exposes no ACL API, so inspection is delegated to the Rust binary rather than to output parsing of a localized tool.

### Hooks carry an argument vector

Exec-form hooks — a `command` plus `args` — are spawned directly with no shell on any platform, so substituted paths pass through verbatim regardless of backslashes or `$`. This retires the entire class of Windows shell-form hook defects at once. The Windows constraint is that `command` must be a real executable, which is why a compiled dispatcher is required rather than a wrapper script.

Being compiled is necessary but not sufficient. A comparable hook manager ships a single Go binary and still fails on multi-line scripts because it constructs a command *string* for the shell instead of an argument vector. The rule the design enforces is compiled binary, explicit argv, no shell.

## Risks / Trade-offs

- **One-time identity churn.** Every bundle identity changes. Mitigated by bumping the manifest schema version, having the verifier accept the prior version while the creator refuses it, and re-pinning `hooks.json` in the same commit.
- **Verbatim path prefix.** `fs::canonicalize` returns a `\\?\`-prefixed path on Windows. The prefix must be stripped before the generation-store containment check or the escape guard fires on every valid bundle. This is the single most likely defect in the port and is covered by a dedicated fixture.
- **Binary distribution weight.** Prebuilt dispatchers for six targets enlarge the payload. Unavoidable: plugin install runs `npm ci --ignore-scripts` under a short timeout, so no build step executes.
- **Two-tier script surface.** Hot-path Rust and cold-path shell is a seam. Accepted because it converts roughly fifteen percent of the rewrite into most of the portability benefit, and the cold path already works through the host's Bash tool.

## Migration Plan

1. Land the capability probe and key predicate first; without them the installer cannot complete on Windows and nothing downstream is observable.
2. Land manifest v2 and re-pin. Gate progress on identity equality across hosts before any further work.
3. Move hot-path hooks into the binary and emit exec form.
4. Add the four-leg matrix, then wire application build pipelines.

Rollback at any step is the previous generation via the existing pointer mechanism; the pointer file and the link coexist during migration.

## Open Questions

- Whether the degradation record should participate in receipt certification, given it is deliberately excluded from identity.

## Resolved

- **Windows local IPC transport.** Settled in [serve the Windows local API on an owner-restricted named pipe](../../../docs/decisions/windows-named-pipe-local-ipc.md). The daemon binds a named pipe with an explicit protected DACL naming only the owner and the local system account, created with the first-instance flag, with client impersonation as defence in depth. AF_UNIX on Windows was rejected because it carries no peer credentials and its file-permission enforcement is undocumented. This change does not implement it; `execution-sidecar-service` takes the requirement delta in a follow-on change.
