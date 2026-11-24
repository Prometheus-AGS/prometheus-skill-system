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

### Preview before installing

`install-binaries.sh --dry-run` prints every build and copy it would perform and
changes nothing: it compiles nothing, downloads nothing, installs no file and
registers no service.

```bash
bash scripts/install-binaries.sh --dry-run
```

```text
  → Building prometheus-cli...
  → [dry-run] would run cargo build --release -p prometheus-cli
  → [dry-run] would install <cargo target dir>/release/prometheus -> <home>/.local/bin/prometheus
```

Use it before the first install on a machine, after changing Cargo configuration,
and whenever `prometheus doctor` suggests a reinstall. The source path on each
line is where the installer will read the binary from, which is the fastest way to
confirm it is looking in the right place.

A real run continues past a component that fails to build. It names the failed
components at the end and exits non-zero, so one broken submodule does not leave
the remaining binaries stale. Read the last lines, not only the exit status of an
earlier step.

### Where build output lives

Cargo writes release binaries to `<workspace>/target/release` unless the machine
redirects its target directory, through `build.target-dir` in a Cargo config file
or the `CARGO_TARGET_DIR` variable. The shell installers never assume the default.
For each workspace they ask Cargo (`cargo metadata`) where it writes, and install
from there.

This matters when the directory is redirected. A hardcoded in-repository path is
then either missing, or still holds a binary from an older build that would be
installed as if it had just been compiled.

The `prometheus` CLI reads only the `CARGO_TARGET_DIR` variable, not Cargo config
files. On a machine that redirects build output, export the variable so the
installers, `prometheus doctor`, `prometheus setup` and `prometheus memory install`
all agree. The [Rust toolchain guide](/docs/guide/rust-toolchain#machines-that-keep-build-output-outside-the-repository)
has the details.

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

The execution installer requires the built binary to match a hash certified for
this machine's architecture. Its dry run prints the hash and target it would
require. See [the certified build hash](/docs/execution/installation-doctor-and-recovery#the-certified-build-hash).

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
point worktrees at one `CARGO_TARGET_DIR` to bypass contention. A machine that
deliberately keeps all build output in one redirected directory depends on that
serialization: two builds writing there at once corrupt each other's output.

KBD uses local signed history by default. Full 1.18.0 supports handoffs between two sessions on the same authority. For replication between separate authorities, use [Companion's source and installation instructions](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/installation.md) and its [control API](https://github.com/Prometheus-AGS/prometheus-companion/blob/main/docs/control-api.md).

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
documentation, deploy Pages or publish versioned binaries and service images; it never substitutes for local tests, doctors
or certification.
