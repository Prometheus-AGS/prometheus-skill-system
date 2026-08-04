# Decision: certify locally and automate only deterministic documentation

**Status:** accepted · 2026-08-03 · release 1.7.0

## Context

Hosted CI was being used as a slow development loop and its output was confused
with evidence from the target Mac. Documentation still needs a mechanism to
detect deterministic source drift after `main` changes.

## Decision

All format, lint, tests, doctors, builds, security checks, and certification run
locally before push. GitHub Actions may only run deterministic `docs:sync`, open
or update one managed documentation PR, and package/deploy Pages. A local workflow
policy rejects hosted test, lint, doctor, certification, or PR-validation jobs.
`docs:sync` changes managed blocks only; authored architecture and use-case prose
remain human-reviewed.

## Alternatives considered

- Hosted PR validation was rejected for latency and evidence-location mismatch.
- Fully generated documentation was rejected because code cannot author design
  rationale or operational judgment.
- Direct bot pushes to `main` were rejected because review and rollback need a PR.

## Consequences

Contributors need a capable local environment. Main remains self-synchronizing
for deterministic reference data, while workflow logs are never release
certification evidence.

## Verification

Local fixtures check workflow policy, managed-only diffs, idempotence, branch
reuse, concurrency settings, action SHA pins, permissions, and Pages separation.
