## Context

See `proposal.md` for motivation. `ProjectRegistry` owns a locked atomic JSON document keyed by canonical replica paths. Sovereign-sync groups these registrations by project and attempts authority startup even after temporary worktrees have been removed.

## Goals / Non-Goals

**Goals:**

- Provide one safe maintenance primitive shared by library and CLI.
- Make dry run the default and require explicit apply authority.
- Retain enough evidence for deterministic manual rollback.

**Non-Goals:**

- Delete runtime project data.
- Infer whether an existing checkout should be adopted or merged.
- Prune automatically during daemon startup.

## Decisions

1. Add `ProjectRegistry::prune_missing(apply)` returning a structured report. Candidate evaluation for apply occurs while holding the existing exclusive registry lock to avoid deleting a path that reappeared after dry run.
2. Store backups below the KBD registry root in a dedicated maintenance backup directory. Each applied mutation records the original registry bytes, SHA-256 checksum, and a JSON receipt listing removed registrations.
3. Extend `prometheus kbd projects` with `--prune-missing` and `--apply`; reject `--apply` without `--prune-missing`. Existing list behavior remains unchanged.
4. Do not remove now-unreferenced project directories. Registry membership and runtime retention are separate policies.

## Risks / Trade-offs

- [Network-mounted paths can be temporarily unavailable] → Pruning is explicit, dry-run-first, and backed up; no daemon automation is added.
- [A path can change between dry run and apply] → Re-evaluate under lock at apply time.
- [Backups accumulate] → They are small and intentionally operator-retained; retention policy is deferred.

## Migration Plan

Add focused runtime tests, CLI parsing/output tests, and release builds. Install the CLI binary only after tests pass, run dry run against the live registry, apply once, and retain the reported backup path. Rollback restores the backed-up registry under the registry lock and restarts sovereign-sync.
