# Verification evidence

## Why there is no CI matrix here

Task 5.1 asks for a "four-leg CI matrix". This repository forbids one.
`CLAUDE.md`'s **Local-Only Validation** section is marked MANDATORY — "Never use
GitHub Actions or any hosted CI/CD runner for testing or validation" — and
`scripts/check-workflow-policy.mjs` enforces it mechanically: only `docs-sync`
and `docs-pages` may exist under `.github/workflows`, and any workflow
containing `npm test`, `cargo test`, `npm run validate`, or a doctor invocation
is rejected. A hosted matrix would fail the repository's own gate before it ran.

The matrix is therefore assembled the way this repository assembles everything
else. Each host runs its own leg **locally** and commits a receipt; a comparator
asserts the legs agree. The property the spec demands is unchanged:

> **WHEN** the same payload is activated on every supported host in the
> verification matrix **THEN** all hosts report the same bundle identity, and
> any divergence fails the change.

Only the place the legs run moves.

## Running a leg

```bash
npm run verify:host-leg        # on each host; writes evidence/legs/<legId>.json
npm run verify:host-matrix     # compare whatever legs have been collected
npm run verify:host-matrix:gate  # the release gate: all four legs required
```

## The leg id is derived, never declared

A leg is identified by what the host's filesystem can actually do, measured by
the capability probe — not by what the host says it is. This matters for exactly
one pair: a Windows host with Developer Mode enabled and one without are the
same platform and the same architecture, and differ only in whether a directory
symlink can be created. Deriving the id from the probe, and re-checking it in
the comparator against `config/host-legs.json`, is what makes

> a passing result cannot come from an elevated configuration

checkable rather than promised. A Developer-Mode-ON host filed as the OFF leg is
rejected by name.

## What each receipt records

- the probed capabilities, including which link primitive the host actually has
- `identity.bundleId` and `identity.goldenPayloadDigest` — the two values every
  leg must agree on, and the only ones the comparator gates on
- the degradations the host had to make, **reported and not hashed**: a host
  that wrote a link as a copy must still land on the same digest
- every local check, its exit code, and the SHA-256 of its raw output

Output is redacted before it is written — the home directory becomes `~`, the
temporary directory becomes `$TMPDIR`, and long digests are elided. The hash is
taken over the **raw** bytes, so redaction cannot launder a failing result into
a passing one.

## Collected legs

| Leg | Status | Notes |
|---|---|---|
| `windows-junction` | collected | Windows 11, Developer Mode **disabled**. `fs.symlinkSync` raises EPERM for `file`, `dir`, and untyped; activation descends to junctions. The hardest leg. |
| `linux` | not collected | needs a Linux host |
| `macos` | not collected | needs a macOS host |
| `windows-symlink` | not collected | needs a Windows host with Developer Mode enabled |

Three scenarios remain unexercised until the POSIX legs are collected, and each
prints an explicit `SKIP` rather than passing silently:

- **differing umask** — needs a host with POSIX mode semantics
- **unsupported entry type** — needs `mkfifo`; Windows cannot create a fifo,
  socket, or device node in the filesystem namespace at all
- **POSIX real-file key protection** — needs a volume that can represent `0600`

## Negative controls

The comparator was checked against three fabricated failures, because a gate
that never fails is not a gate:

| Fabricated fault | Result |
|---|---|
| A leg computing a different golden digest | fails, naming both values and the legs holding them |
| A junction-only host filed as `windows-symlink` | fails: `directory link strategy is junction, expected symlink` |
| A leg whose own local checks did not pass | fails: `local check failed: host capability probing` |
