# Acceptance — change-ldd-12-integration-rollout

Production changes must be complete before these scenarios are authored or run. All execution is delegated to the single final boundary in change 12; this document is not a runnable per-change gate.

- AC-1: All sibling acceptance results are traceable to candidate commits/tree hashes, exact commands, scratch paths, tool versions and output receipts. 0=PASS, 1=FAIL, 2=BLOCKED; skipped prerequisites, missing judge, unavailable model or missing approval never count as completion.

- AC-2: Every test owns scratch HOME, CODEX_HOME, CORTEX_DATA_DIR, PROMETHEUS_PLUGIN_ROOT, learning log/index/outbox and service namespaces. Use no real-home auth/config/data symlink or DB connection. Server process trees and scratch directories are cleaned without touching unrelated sessions.

- AC-3: G1 packaged integrity, G2 actual Codex startup control, G3 published pin identity/rollout, G4/G5 real cache behavior, G6 ownership negative controls, G7/G8 installed repair/home routing, G9/G10 real bounded Cortex, G11 byte/idempotence, G12/G13 mini package behavior and G14 authored acceptance examples all have explicit results.

- AC-4: Final report separates source implementation, local certification, user merges and approved deployment. If merge/config/version approval is pending, retain pending tasks and show exact remaining work; do not close the phase.

## Gate ownership and evidence

See `.kbd-orchestrator/phases/phase-learning-deploy-and-debt/verification-contract.md` for isolation, command contract, protected-test integrity and result handling. Change 12 records this change's evidence as pending, PASS, FAIL or BLOCKED and links exact receipts. Supplementary static/schema checks cannot replace these production boundaries. No acceptance result exists yet.

## Release-closure scope amendment

The owner expanded this phase to finish full/mini as stable products. This final boundary also covers changes 13-18, both Docusaurus production builds and locally served page/route/example checks, all content inventory dispositions, actual team/handoff/model recipes, Companion two-process and absent-service behavior, current mini production baseline and fresh remote clone/installability of every final pin. Do not run aggregate scripts that implicitly add unit suites as acceptance.

Companion check-in/push is explicitly authorized by the user; after local source/security/integration gates, stage only manifest-listed source, create the proposed private remote if needed, push and verify the remote commit/fresh clone. User-owned merges, versions.toml changes and tags retain their explicit boundaries. Publish site content only from merged locally certified identities. Do not merely prepare a repository without pushing it and call publication complete.

Reconcile the four historical control-plane changes through their existing canonical IDs only after the final evidence; preserve and explain missing-receipt provenance. Finish all in-scope release blockers, record maintenance readiness and remove merged worktrees only after checking live processes and marketplace references. Never alter deploy-main/deploy/main.
