# Managed OpenSpec lifecycle

This directory is self-contained and uses Node builtins. Copy the whole directory
when packaging it for another harness. Node must meet OpenSpec's requirement
(20.19.0 or newer); invoke JavaScript with `node`, without executable bits or
shell shims.

```text
node cli.mjs refresh --project /path/to/project --timeout-ms 120000
node cli.mjs run --project /path/to/project -- list --json
```

`cli.mjs` also exports `main(argv)`, returning the exit code. A wrapper must assign
that code to `process.exitCode`. Receipts are JSON on stderr. `run` preserves the
OpenSpec command's stdout, including JSON output.

## Selection and installation

Every explicit `refresh` queries the official npm registry's latest stable
`@fission-ai/openspec`, with minimum version **1.14.0**. A registry failure permits
the last successfully selected cached CLI, with `latestVerified: false` and an
explicit `cached-offline` policy. It never claims an offline cache is latest.
`run` reuses the selected CLI without a registry request or reinstall; its first
invocation ensures a version if none is selected.

The default managed home is `~/.prometheus/openspec`. Exact versions live beneath
`versions/<version>`. npm is resolved from the Node installation or PATH and its
JavaScript entry is invoked with Node, `shell: false`. Installation is local to
that version directory, with the official registry, exact dependency, lifecycle
scripts disabled, global mode disabled, and binary symlinks disabled. Commands
invoke the package's own JavaScript CLI directly; project dependencies and global
OpenSpec installations are not used or upgraded. A successful CLI version probe
is required before a newly installed version is reusable.

Human overrides are explicit environment variables:

| Variable | Behavior |
| --- | --- |
| `PROMETHEUS_OPENSPEC_HOME` | Override the managed home, including for isolated integration runs. |
| `PROMETHEUS_OPENSPEC_VERSION` | Pin an exact stable version at or above 1.14.0; receipts say `operator-pin`, latest unverified. |
| `PROMETHEUS_OPENSPEC_DISABLE=1` | Disable managed activity; refresh records disabled and exits 0, run exits 2 without executing a CLI. |

## Refresh boundary

Refresh searches upward for `openspec/` plus `.prometheus/project.json`,
`.kbd-orchestrator/project.json`, or `.kbd-orchestrator/current-waypoint.json`.
Unrelated directories produce a `skipped` receipt. `run` only needs an OpenSpec
ancestor, and can run `init` in the requested directory if none exists.

Refresh runs `openspec update <project> --force` with stdin closed,
`OPEN_SPEC_INTERACTIVE=0`, and `OPENSPEC_NO_UPDATE_CHECK=1`. Force permits the
official generated-integration cleanup after backups. The audited implementation
is [OpenSpec v1.14.0](https://github.com/Fission-AI/OpenSpec/tree/v1.14.0/src/core).
The selected package's own tool, skill, workflow, command, and legacy-migration
declarations determine backup paths. An incompatible internal API fails before
update instead of guessing paths.

Before update, backups cover OpenSpec skill directories and ownership markers,
generated workflow commands, detected legacy command artifacts and marker-bearing
root instructions, `openspec/AGENTS.md`, the exact OpenSpec global configuration,
and Copilot's OpenSpec agent/setup files. This includes global surfaces when the
official tool adapter uses them. It does not recursively copy unrelated harness
configuration directories. Symlinks are recorded as metadata, never created by
the backup. Authored specs, changes, config, legacy project notes, and KBD identity
and waypoint JSON are fingerprinted for audit, without recursively copying KBD
logs/state. An observed delta produces `migration-required`; it may be a concurrent
agent's edit and is not attributed to upstream. The runner never restores or
overwrites authored files. Broader authored KBD state is outside the updater's
declared generated footprint and is not traversed.

No specification `schemaVersion` is invented or rewritten. An update refreshes
generated agent instructions; it is not a semantic migration or validation of
authored specifications. Legacy `openspec/project.md` and an unconfigured project
produce `migration-required`, including when upstream exits 0. Resolve the
reported prerequisite explicitly and validate the project's specs separately.

## Deadlines, contention, and recovery

The timeout defaults to 120 seconds. Every spawned npm or OpenSpec process shares
the remaining deadline and is killed on timeout or helper SIGTERM/SIGINT. A short
startup refresh (for example 12 seconds) never starts npm installation when fewer
than 60 seconds remain: it records `pending` and exits 75. Run the longer refresh
at phase entry to complete installation. Nothing starts a daemon or background
retry. Set the host hook timeout above the helper budget so it can write its
receipt and release its lock.

All managed operations hold an exclusive `operation.lock` containing the owner PID,
operation, project, and start time. Contention exits 75 with an inspectable reason.
The lock also serializes global generated surfaces. External OpenSpec commands
must not update those same generated surfaces concurrently. A hard-killed
process can leave a lock: inspect its owner and remove the lock only after
confirming the operation has stopped. No automatic stale-lock eviction occurs.

Receipts are in `receipts/<project-hash>/`, with an atomic `latest.json`. Each
refresh receipt records its version policy, status, output, and backup path when
available. Backups are in `backups/<timestamp-uuid>/manifest.json`; every entry
names its original absolute path, saved copy, prior fingerprint, and symlink
metadata. Authored audit fingerprints are recorded separately. Generated files
may be partly updated after a failed or timed
out upstream operation. Use that manifest to restore the affected paths or retry
refresh after addressing the failure. Keep backups until the result is accepted.

Exit 0 means refreshed, skipped, or explicitly disabled; inspect the receipt's
status to distinguish them. Exit 2 means a migration prerequisite or disabled run;
75 means pending/contended; other nonzero codes report installation/update/CLI
failure. Successful cache selection alone does not certify a project refresh.
