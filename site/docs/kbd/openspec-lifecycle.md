---
title: OpenSpec lifecycle updates
sidebar_label: OpenSpec lifecycle
description: Latest-stable OpenSpec selection, automatic project refresh and recoverable update receipts.
---

# OpenSpec lifecycle updates

The release baseline is **OpenSpec 1.14.0**, confirmed on 2026-10-03 from the official npm latest tag and [upstream release](https://github.com/Fission-AI/OpenSpec/releases/tag/v1.14.0). Source dependencies pin this version. The managed KBD runner checks the latest stable release, with 1.14.0 as its minimum, whenever it performs a refresh.

## What updates automatically

The bounded startup context hook refreshes existing KBD/OpenSpec projects. New/next phase and child entrypoints run the longer refresh before lifecycle mutations. Stage instructions require the same preflight when the harness has no startup hook; raw canonical CLI use must follow that preflight too.

A project must have an openspec directory and a recognized KBD identity or waypoint. Unrelated directories are skipped. The runner invokes upstream openspec update --force to regenerate the configured agent skills, commands and supported legacy integrations.

**Authored specifications are preserved.** This is not a semantic migration of specification prose, a fabricated schema-version rewrite or a replacement of KBD task identity. Existing project notes, missing configuration or observed authored-file changes can require explicit migration work. Validate the project's changes separately after resolving that prerequisite.

## Run the same CLI throughout a task

From this pack's source checkout:

```text
node skills/process/kbd-process-orchestrator/shared/openspec/cli.mjs refresh --project /path/to/project --timeout-ms 120000
node skills/process/kbd-process-orchestrator/shared/openspec/cli.mjs run --project /path/to/project -- list --json
node skills/process/kbd-process-orchestrator/shared/openspec/cli.mjs run --project /path/to/project -- validate change-id --type change --strict
```

In an installed package, locate the complete bundled helper directory through the installed skill. Preserve all helper modules together. The run command keeps OpenSpec stdout intact, including JSON, while lifecycle receipts go to stderr.

The managed installation lives under ~/.prometheus/openspec/versions. It executes the package's JavaScript entry with Node and shell disabled. Exact versions are installed without lifecycle scripts or binary symlinks. It does not upgrade project dependencies or global CLI installations. Mini retains its Node-only portability contract; Windows execution still requires independent runtime validation.

For a separately installed global CLI, use the package manager that owns that installation and verify every executable found on PATH. An npm-managed installation can be explicitly upgraded with npm install --global @fission-ai/openspec@1.14.0. A newer managed KBD CLI does not prove an older global shim was replaced.

## Availability and operator controls

| Condition/control | Behavior |
| --- | --- |
| Registry available | Resolve latest stable, install the exact managed version if needed and probe its CLI version |
| Registry unavailable with a selected cache | Use that cache with latestVerified: false and an explicit offline policy |
| Cold startup with insufficient time to install | Report pending (exit 75); complete the longer refresh before OpenSpec work |
| Concurrent managed operation | Report contention (exit 75); no parallel updater or background daemon |
| PROMETHEUS_OPENSPEC_HOME | Set a different managed cache, receipt and backup root |
| PROMETHEUS_OPENSPEC_VERSION | Pin an exact stable version at or above 1.14.0; report operator-pin, not latest |
| PROMETHEUS_OPENSPEC_DISABLE=1 | Record disabled refresh; managed run exits 2 without executing OpenSpec |

The normal refresh budget is 120 seconds. Startup uses a shorter budget and never begins installation with fewer than 60 seconds remaining. A timeout can leave partially regenerated files; use the recorded backup or retry after resolving the failure.

## Receipts and recovery

Before mutation, the selected OpenSpec package's own declarations identify generated files to back up, including legacy commands and relevant global generated configuration. An incompatible upstream API fails before update instead of guessing the update footprint. Authored specs, changes, configuration and KBD identity/waypoint data are fingerprinted for audit; the runner never rolls back another agent's authored edits.

Receipts live under receipts/project-hash with a latest.json pointer; backups under backups/timestamp-uuid contain a manifest naming original paths, saved copies and fingerprints. Retain them until the update is accepted. Inspect status and latestVerified: exit 0 alone can mean refreshed, skipped or explicitly disabled. Exit 2 signals a migration prerequisite or disabled run; exit 75 signals pending or contention.

After a hard-killed process, inspect the operation.lock owner before removing a leftover lock. Restore only the generated paths named in the backup manifest, accounting for later edits. Do not claim semantic compatibility merely because CLI selection or generated refresh succeeded.

## Release and publication

Mini 0.2.0 and full 1.11.0 include this lifecycle. The full process plugin is 1.7.0. Reload existing harness sessions after updating the pack. On machines using the full pack, distribute the full pack only and preserve custom harness configuration and the previous generation.

The GitHub Pages workflow builds the Docusaurus site and skills catalog after these changes reach main. Local builds validate content before publication; hosted deployment is not runtime certification. A PR alone is not evidence that the public site has updated.

See [Task model assignments](/docs/kbd/task-model-assignments) for the planning and execution contract.
