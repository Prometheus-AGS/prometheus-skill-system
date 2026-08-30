## Context

See `proposal.md` for motivation. Three source changes converge in shared generated distributions and user installations. A top-level compatibility directory also resembles a live phase even though the canonical child record exists in the nested hierarchy.

## Goals / Non-Goals

**Goals:**

- Give shared generation and install refresh one owner.
- Preserve compatibility evidence outside live discovery.
- Certify combined behavior through the actual local services.

**Non-Goals:**

- Rewrite canonical KBD history or retained project runtimes.
- Modify the dirty UAR worktree.
- Use GitHub Actions or another hosted runner as validation evidence.

## Decisions

1. Verify both paths before moving compatibility evidence. The backup destination includes a timestamp and receipt containing source, destination, canonical target, and hashes.
2. Generate twice before installation and compare tracked output hashes. This detects nondeterministic generators before copies are refreshed.
3. Use the repository-owned user installer rather than copying managed skills manually. Installed-source parity is checked after refresh.
4. Apply registry pruning only after the release CLI is installed. Ordinary KBD commands commit directly to the signed local runtime; sovereign-sync remains stopped and disabled unless an operator explicitly enables the sharing profile.
5. Treat live memory writes as test entities with unique names and retain them as explicit certification evidence; no secret-bearing hook output is stored.

## Risks / Trade-offs

- [Refreshing all managed skills can surface unrelated drift] → Inspect installer plan and preserve user-owned collisions; fail rather than clobber.
- [Historical logs contain old unavailable warnings] → Capture timestamps/offsets before each restart and inspect only newly appended lines.
- [Moving an untracked compatibility directory is hard to review in Git] → Write a tracked reconciliation receipt in the active phase and include the preserved backup path and digest.

## Migration Plan

Confirm the compatibility and canonical paths, preserve evidence, generate and validate distributions, refresh managed skills, install the tested CLI, apply registry pruning, and certify daemon-free local operation. Rollback uses the registry backup and compatibility receipt; source rollback is a normal Git revert followed by the same deterministic refresh.
