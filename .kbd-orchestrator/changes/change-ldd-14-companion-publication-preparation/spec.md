# change-ldd-14-companion-publication-preparation

Title: Recover document and prepare the Companion repository for publication
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: companion
Gaps: G16
scope:
  - companion:Cargo.toml
  - companion:Cargo.lock
  - companion:crates/**
  - companion:src-tauri/**
  - companion:scripts/**
  - companion:shared/**
  - companion:skill-package.json
  - companion:skills/**
  - companion:README.md
  - companion:docs/**
  - companion:.gitignore
  - phase:companion-publication.md

## Requirements

1. The new service repository is /Users/gqadonis/Projects/prometheus/prometheus-companion. Its current committed HEAD is 773aa5e33b2217135bc423e526c497cceb0e7058; remote list is empty and substantial workspace/service source is untracked. Recover all intended source and distinguish source from build output, session memory, keys and machine configuration; never git add -A over this checkout.

2. Document the actual shipped role: optional control/service supervisor and peer discovery/pairing/sync, sovereign-sync, sovereign-client, kbd-mobile and iroh-docs-adapter; connected sync-status/sync-peers/sync-push skills. Verify actual boundaries in source. The pack operates without Companion and never gains a reverse runtime dependency on it.

3. Replace the starter Tauri README with build/install/use/architecture/support/license information matching implemented behavior. Separate the large draft UI/mobile roadmap from supported release capabilities; neither complete speculative roadmap features nor present them as shipped. Preserve authored source and branding.

4. No origin/main exists here. Bootstrap an isolated private local Git mirror from the committed main snapshot, expose its honest origin/main as the initial local base, and make the publication worktree under ../worktrees from that base. Import only reviewed dirty source into it. Record that the origin is a local staging mirror until final publication; do not fake an upstream commit or change the original checkout.

5. Prepare a source-only publication manifest and remote destination. The user explicitly authorized checking in and pushing this repository; preferred destination is a new private Prometheus-AGS/prometheus-companion repository, with private current-account fallback if organization creation is unavailable. No public visibility change is implied. Actual commit/push follows final local gates in change 12; existing versions.toml changes and new tag choices still need explicit owner approval.

6. Align the four full-pack Cargo git dependencies from cfbc262b47ce794cb2ea845c64dc33e70b10e890 to the owner-selected ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61. See pin-proposal.md; this does not authorize protected version-file changes or a later SHA substitution.

## Binding constraints

Source work uses isolated worktrees under ../worktrees with declared base/provenance; protect deploy-main/deploy/main and other agents' changes. All tests, builds and reviewers wait for completed production and run only locally under change 12. Use scratch HOME/CODEX_HOME/CORTEX_DATA_DIR/plugin/learning roots; never test against live :23001. One Cargo/rustc process machine-wide; pgrep first. User merges every PR and explicitly approves versions.toml edits, tags and real Codex/Claude changes. No approval is inferred from planning. See the phase plan for ordering, exact task model assignments and final gates.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-14-companion-publication-preparation` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.

### Companion declared-license publication source — 2026-10-05
The recovered workspace declares MIT and names Travis James as author, but has no root LICENSE. Add companion:LICENSE to14/3 source publication scope and preserve the exact existing full-pack MIT License/copyright notice for relocated/owned source. This fulfills the declared license and redistribution notice; it creates no new license selection or ownership claim. Record its hash in the source/publication inventory before final12 outbound-history/local certification. Root owns this one file.
