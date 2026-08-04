# Readiness Evidence Register

Readiness is reported by evidence class. The former “92% production readiness”
score mixed code review, local tests, installed services, and external operation
into one number. Those claims are not interchangeable, so this release does not
publish a composite percentage.

## Evidence classes

| Evidence class | What qualifies | 1.7.0 status | What it does **not** prove |
|---|---|---|---|
| Artifact certification | Local format, compile, clippy with warnings denied, unit/integration/property tests, docs checks, generated-diff checks, and protected-test verification against an exact commit | Recorded under `docs/releases/1.7.0/` before publication | That a daemon is installed, loaded, reachable, or used by another machine |
| Disposable runtime certification | Isolated homes, sockets, ports, keys, peer identities, and data prove restart, replay, pairing, rejection, receipts, migrations, and crash recovery | Recorded under `docs/releases/1.7.0/` with redacted configs and latency samples | That the operator’s persistent service is configured or healthy |
| Installed-service status | This Mac’s signed binaries, LaunchAgents, socket permissions, queue/snapshot state, plugin generation, logs, and non-excluded doctors | Recorded only after the local installation/certification phase | That another host or production fleet has the same state |
| External deployment evidence | Named deployment owner, environment, duration, workload, incident history, recovery exercise, and independently retrievable evidence | **Not established by this repository** | Nothing may promote artifact or local-host evidence into this class |

## Release rule

A release record must name the commit, command, exit code, timestamp, sanitized
environment, and evidence path. Warnings need an explicit disposition. GitHub
workflow output is never certification evidence: hosted automation is limited to
deterministic documentation synchronization and Pages packaging/deployment.

## Current claim boundary

The repository contains implemented runtime and test artifacts. The authoritative
1.7.0 status is the evidence index produced during final local certification,
not this prose page. Until that index exists and is green, “implemented” must not
be restated as “installed,” “certified on this Mac,” or “production deployed.”

External production use remains unverified. That is an evidence gap, not an
eight-percent score.
