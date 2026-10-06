---
title: Installation and upgrades
description: Install independently versioned components and preserve services and data during updates.
---

# Installation and upgrades

Keep the approved source commit, dependency pins and artifact manifest together.
Skills, CLI, memory, knowledge and execution packages have independent versions.
Compare each installed binary with its own manifest entry; no shared version
string describes every component.

The [service operations guide](/docs/guide/service-operations) lists ownership,
platform templates, interfaces, data locations and recovery boundaries.

## Select the installation

The full skill installer supports a skills-only path and a full path:

```bash
bash scripts/install-skills-flat.sh --skills-only
bash scripts/install-skills-flat.sh
```

`--best-effort` is a development option whose partial outcome needs inspection;
it is not certification. Building learner/bridge binaries does not start their
services. From the same checkout, install binaries and inspect the service plan:

```bash
bash scripts/install-binaries.sh
bash scripts/install-mcp-services.sh --dry-run
bash scripts/install-mcp-services.sh
```

Use Bash 4 or newer for the native service installer. macOS `/bin/bash` 3.2
does not support its associative arrays. Platform support follows the actual
templates: the execution service and liter-llm API currently have macOS
templates, even where their binaries may also run elsewhere.

Select services with repeatable `--exclude <service>`; other options include
`--restart`, `--unload`, `--learning-recovery` and `--render-only <directory>`.
Inspect the exact selections before modifying a machine. An existing reused
listener retains its own provenance and data.

Execution has a separate atomic binary installer and service plan:

```bash
bash scripts/install-prometheus-exec.sh --dry-run
bash scripts/install-prometheus-exec-service.sh --dry-run
```

Artifact verification and service registration are separate from functional
execution. The general service installer can warn about an execution-service
failure and continue, so its exit status alone does not prove every service
usable. See [execution recovery](/docs/execution/installation-doctor-and-recovery).

## Harness and source ownership

Install for the selected harness and home. Codex's effective home is a nonempty
`CODEX_HOME`, otherwise the selected user's `.codex` directory. Preserve that
selection through doctor, removal and rollback. Compatibility copies and native
plugins have different ownership receipts; unrelated user skills remain theirs.

Keep the clean source while marketplace/service registrations refer to it.
Never edit installed plugin generations or caches. Change source, regenerate
at the completed production boundary and activate through the verified
installer. Hook Python suppresses bytecode writes into immutable payloads.

Codex memory configuration disables the feature and both generation/use flags.
Doctor checks the actual selected home's configuration and known summary paths.
The historical generation-only setting did not stop startup consolidation.

## Upgrade sequence

1. Record source/artifact identities, service definitions, selected homes and
   plugin generation receipts.
2. Back up signed KBD state, database, knowledge library, learner store,
   identities and pending queues, with relevant writers stopped.
3. Finish production and complete the required local integration gate using
   isolated test homes/data before deploying.
4. Install approved artifacts and the verified plugin generation, then apply
   selected service definitions.
5. Exercise installed production paths and record functional evidence; an
   open port or version string is insufficient.
6. Recover through preserved artifact/generation receipts and compatible data
   backups. Retain failed-attempt evidence and reconcile uncertain writes.

Serialize Cargo/rustc machine-wide. Check active processes first, keep each
workspace/worktree's own target directory and use `sccache` for reuse. Do not
share `CARGO_TARGET_DIR` across worktrees to bypass contention.

KBD is local by default. Companion owns optional connected control through its
separate recovered source repository and [integration contract](/docs/kbd/integration-contract).
Its absence is normal. The current source has no public remote or certified
release; pack installation never builds or installs it.

For an explicitly selected Companion installation, follow its separate
[installation and ownership](/docs/sovereign-sync/installation) procedure.
Choose one headless, windowed or standalone socket owner, preserve the enrolled
key and selected socket/config/data overrides, and register clients against that
same endpoint. Pack service adoption requires an explicit manifest path.
Companion's Claude skill registration and macOS service replacement have their
own private ownership receipts/backups; neither takes over unrelated entries.

Preserve its durable pre-POST MCP intent outbox and matching push receipts during
upgrades. Resolve an uncertain result on the original host before authorized
replay; replacing the ID or recreating a key is not recovery. See
[signed pushes](/docs/sovereign-sync/signed-pushes-and-receipts).

Push only after applicable local gates pass. The owner merges PRs and approves
protected versions and tags. Hosted automation may synchronize deterministic
documentation or deploy Pages; it never substitutes for local tests, doctors
or certification.
