---
type: SessionRecord
id: karpathy-session-385ca7911c7dc0df
title: Karpathy session 385ca7911c7d
tags:
- karpathy
- session-learning
sources:
- resource: session:5789740d-cbcd-4a6a-8e3e-182eb55399dd
generated:
  by: pk/1.11.0
  at: 2026-11-16T14:37:11.132122+00:00
created_at: 2026-11-16T14:37:11.132122+00:00
updated_at: 2026-11-16T14:37:11.132122+00:00
revision: 0
---

## Delta

The documentation now matches the code for everything added since 1.15.0, and the full local docs check (`npm run docs:check`, including the Docusaurus build) passes. Nothing is committed.

Three sets of features had only a changelog line, or nothing, in the guide and site: the VLBV R4 merge, the liter-llm loopback fix, and this session's installer, exec pin and CLI changes. Each now has usage examples, the reasoning behind it, and when to use it or not.

## New site pages (KBD → Quality)

- **[Boundary authorization and early-execution evidence](site/docs/kbd/boundary-authorization.md):** why the B4/B5 authorization is recorded, when to record it and when not to, the command with its options and exit codes, how to declare `runnerOutputGlobs`, and how to read the four scan results in the closure packet.
- **[Protocol evals and capability evidence](site/docs/kbd/protocol-evals-and-capability-evidence.md):** what the 23 cases measure, when to run them (boundary only, since each sample is a paid session), the run commands and negative controls, and how a capability becomes `supported` only on evidence.

## Updated pages

- **[Exec installation](site/docs/execution/installation-doctor-and-recovery.md):** the per-target hash pin, what makes the build reproducible, when to recertify, and the override variables.
- **[Installation and upgrades](site/docs/operations/installation-and-upgrades.md):** installer dry runs and where build output lives.
- **[Doctors](site/docs/operations/doctors-and-mac-certification.md):** the managed-binary check, how to read its two hashes, and where doctor looks for the built file.
- **Guide:** the [Rust toolchain](docs/guide/14-rust-toolchain.md), [scripts reference](docs/guide/16-cli-and-scripts.md), [liter-llm](docs/guide/09b-liter-llm.md), [service operations](docs/guide/26-service-operations.md) and [scenario workflow](docs/guide/value-scenario-workflow.md) pages.
- **Project files:** [CHANGELOG.md](CHANGELOG.md) gains entries for this session's fixes, [CLAUDE.md](CLAUDE.md) gains two rules (never read a hardcoded `target/release`; the exec pin is per target) and loses a stale note, and the [eval README](evals/vlbv-protocol/README.md) now says how Claude Code authenticates in isolated runs.

## Found while checking the docs against the machine

- **The site build was already broken.** A guide page linked a Markdown file outside the guide, so the build has failed since PR #204. I fixed the link.
- **The liter-llm gateway here still listens on every interface.** It was started on 13 November, before the loopback fix, and has not been restarted, so `lsof` shows `*:4000`. A restart picks up the new wrapper.
- **A liter-llm build failure still aborts `install-binaries.sh`.** The lookup is guarded now, but the `cargo build` in that block is not. I documented it accurately in CLAUDE.md and did not change the script.
- **No capability has been promoted from eval evidence yet.** The committed reports cover Claude Code and Kimi, and most agent cases are `BLOCKED`. The page says to read the reports rather than claiming compliance.

One number carried over from earlier is still unverified: the Apple Silicon pin. The changelog entry says so and gives the command to run on that machine.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 5789740d-cbcd-4a6a-8e3e-182eb55399dd
- Captured: 2026-11-16T14:37:08.659906Z
- Project: /Users/gqadonis/Projects/prometheus/prometheus-skill-system

## Changed Paths

- CHANGELOG.md
- CLAUDE.md
- docs/guide/09b-liter-llm.md
- docs/guide/14-rust-toolchain.md
- docs/guide/16-cli-and-scripts.md
- docs/guide/26-service-operations.md
- docs/guide/value-scenario-workflow.md
- evals/vlbv-protocol/README.md
- site/docs/execution/installation-doctor-and-recovery.md
- site/docs/operations/doctors-and-mac-certification.md
- site/docs/operations/installation-and-upgrades.md
- site/sidebars.js
- site/docs/kbd/boundary-authorization.md
- site/docs/kbd/protocol-evals-and-capability-evidence.md
