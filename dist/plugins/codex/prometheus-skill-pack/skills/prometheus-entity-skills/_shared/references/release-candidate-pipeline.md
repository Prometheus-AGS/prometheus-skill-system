# Release-candidate pipeline agent contract

Load this reference for 3.0 RC planning, release rehearsal, npm staging,
dependency order, provenance, protected tags, or partial-publication recovery.

## Use the implemented gate

The machine-readable coverage ID is `release.pipeline.recoverable-rc`. Require:

```bash
pnpm run test:release-pipeline
pnpm run bdd:release-pipeline
pnpm run verify:release-pipeline
```

The verifier is non-mutating. It derives the candidate set from
`release/v3-release-contract.json`, packs twelve npm candidates, consumes only
those tarballs, verifies workflow policy, and writes a recovery/visual receipt.
A green result proves this local boundary; it does not prove external registry
or GitHub settings.

Require one Changesets-managed numbered version across the fixed group:
`3.0.0-rc.N`. Reject alpha packages, unnumbered `-rc`, mixed versions, and
other prerelease channels before creating an RC manifest. The current local
candidate is `3.0.0-rc.1`, not a published or stable release.

## Keep authority explicit

- RC staging requires GitHub Actions OIDC, the protected `npm-rc` environment,
  an exact source-SHA match, and explicit `stage-rc` authority.
- Long-lived npm write tokens are forbidden.
- RC staging targets `next`, never `latest`.
- Dart and standalone Rust registries remain dry-run-only; Tauri Rust is
  embedded in its npm distribution.
- Registry mutation is never implied by a plan, rehearsal, verifier result, or
  generated evidence file.

For partial progress, classify immutable versions as matching, absent, or
conflicting. Skip and record matching integrity, stage absent artifacts in
dependency order, and block conflicts. Never advise overwriting a registry
version.

Persist the staging journal after every confirmed transition and immediately
before a mutating stage attempt. A thrown partial failure must retain completed
packages and the last attempt in a `status: failed` report, and CI must upload
that report even when the stage command exits nonzero.

Rehearsal journals must record bundle-relative `packages/*.tgz` paths. Resolve
them only inside the downloaded candidate bundle in the stage job, and reject
absolute paths or traversal.

Before any stage-path registry read, require protected GitHub/OIDC authority.
Then require one exact closed rehearsal proof: matching schema,
source SHA, candidate version, dist-tag, order, complete seven-state npm
journals, dry-run gates and receipts, protected-tag snapshots, native dry-run
receipts, and embedded-native dispositions. Reject missing, extra, reordered,
or inconsistent records.

For every absent version, require the JSON response from
`npm stage publish --json` to contain the exact package name, version, tarball
integrity, and registry-issued stage UUID. Do not substitute local integrity
for a missing response field and do not require OIDC trust tokens to run stage
management subcommands. Matching public versions use the exact npm registry
lookup as their verification authority.

## Do not overclaim

This gate does not certify npm trusted-publisher setup, GitHub environment
reviewers, the production documentation site, a GitHub Release, stable 3.0.0,
or npm `latest`. Those remain owned by `v3-release-certification` and
`v3-stable-publication`. Read
[`release/release-candidate-pipeline.md`](../../../release/release-candidate-pipeline.md)
for the operator contract and explicit exclusions.
