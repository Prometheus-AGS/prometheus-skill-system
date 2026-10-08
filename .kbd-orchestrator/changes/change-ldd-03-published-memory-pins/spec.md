# change-ldd-03-published-memory-pins

Title: Freeze and apply owner-approved final release pins across full mini and services
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: full+mini+dependency releases
scope:
  - Full and mini gitlinks and their release/version declarations
  - mini:versions.toml (explicit approval required before edits)
  - Companion versions.toml if alignment requires it (explicit approval required)
  - Affected dependency Cargo/package/OpenAPI version declarations after release decision
  - phase:release-candidates.json, release-manifest.json, pin-proposal.md

## Superseding requirement

The owner's latest request supersedes the earlier v1.10.1-only target. Do not apply the old SHA proposal. v1.10.1 is historical evidence/a known baseline, not the final requested pin. The owner explicitly selected https://github.com/Prometheus-AGS/prometheus-skill-system/commit/ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61 as the full skill-system pin. This target is resolved and verified at the planning worktree HEAD/origin/main, including merged #157/#158. Companion Cargo.toml currently consumes cfbc262b47ce794cb2ea845c64dc33e70b10e890 in four git-rev dependencies; align those consumers to the selected exact SHA in the isolated production candidate. This is not a surreal-memory SHA and does not settle other dependency releases. Do not silently advance this exact pin; any later necessary pack commit requires a superseding owner decision. Protected versions.toml edits and tags still require a concrete approved patch/proposal.

## Required behavior

1. Record each current gitlink, upstream URL, latest fetched head/tag, every in-scope local commit/dirty patch, canonical consumer schema receipt and proposed final identity. Include ALL full/mini gitlinks and the relevant dependency/Companion source, not only surreal-memory. Reconcile worktree contents through change 13 first; versions are consequences of included source.
2. After production convergence, choose the final compatible release graph containing all intended cache/hook/team/service work. Record frozen SHAs, tree identities, versions, tag provenance, local evidence and explicit exclusions. Include the owner-selected full skill-system pin ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61; no silent substitution. No floating refs, local path dependencies, unreviewed upstream jump, retagging v1.10.1 or unaccounted work.
3. Produce a concrete minimal patch for protected version files and an exact tag/release proposal; get explicit chat approval before those mutations. Full has no root versions.toml; update existing release matrices/manifests, not an invented file. Reuse current authoritative manifests and regenerate derived outputs at change 12.
4. Apply approved exact pins and matching version declarations coherently across both products. Validate the whole graph in the final local gate. Publish dependency source first and verify remote reachability before presenting the parent release as installable. A squash/merge changing the approved SHA requires a new explicit pin decision and the smallest affected confirming gate.
5. The user merges every PR. Record the release freeze cutoff, re-fetch before approval/publication and dispose newer relevant work deliberately. Future maintenance updates follow the same compatibility/local-evidence policy; do not chase unrelated moving heads indefinitely.

## Acceptance

A fresh scratch checkout resolves every final pin from its actual remote and installs the certified product with no local checkout dependencies. The manifest maps every in-scope outstanding item to merged/released source or an explicit disposition. All package/site/generated declarations agree, and all protected edits/tags have recorded approval. Missing target/approval or unreachable pin is BLOCKED. Tests are deferred until all production work is complete; change 12 owns the final gate.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-03-published-memory-pins` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.

### Owner-approved release sequencing — 2026-10-05
Direct owner chat: "I approve the release proposal". release-approval.json records the exact approved proposal hashes, eleven Companion protected pins, stated conditional tags/plugin bumps and the named Immutable Implementation-First and Integration-Only Policy sequencing exception in that proposal. Preserve all other policy.03/task3 may apply the approved currently-resolvable metadata batch while03/task2 retains final dependency/remote identity freeze. After every functional and known packaging source edit is complete,12/task1 reconciles generation and authors scenarios, then12/task2 runs the initial consolidated real-candidate gate. Dependency certification/owner merge and exact mini SHA approval precede final parent metadata and the final fresh-clone gate.03 tasks are not falsely completed from an initial approval/batch. Tags, maintenance readiness and historical closure remain after required gates/owner merges. Approval resolves the preceding goal blocker and resets the blocked audit; no invented final SHA or validation evidence.
