# PLAN: phase-sovereign-sync-hardening

**Produced by:** /kbd-plan
**Date:** 2026-06-29
**Project:** prometheus-skill-pack
**OpenSpec available:** yes (`openspec/`)
**Backend:** OpenSpec changes with KBD waypoint tracking

---

## Planning Basis

This phase does not yet have a formal `assessment.md` or assessment handoff. The stage gate therefore ran in legacy mode and the plan is based on:

- `.kbd-orchestrator/phases/phase-learn-sovereign-sync/reflection.md`
- `.kbd-orchestrator/phases/phase-learn-sovereign-sync/assessment.md`
- current hardening `progress.json`
- memory entries for the completed real `IrohDocsAdapter`
- repository inspection of OpenSpec and KBD conventions

TD-01 from the previous reflection, "IrohDocsAdapter unimplemented", was completed before this plan. This plan treats that work as closed and focuses on the remaining hardening required to make the sovereign-sync stack verifiable and usable.

## Sycophancy Self-Check

Rejected scope creep:

- Do not build the full Tauri plugin, WASM module, Flint Gate auth, or embedding matcher in this phase.
- Do not re-plan already completed scaffold work from `phase-learn-sovereign-sync`.
- Do not present the existing iroh-docs adapter as fully multi-device ready until share/import ticket support and two-node regression coverage exist.

Accepted hardening scope:

- Make real iroh-docs synchronization demonstrable between nodes.
- Add CI and regression tests around the Rust crates already created.
- Cover the MCP client pool integration path with an end-to-end test.
- Fix the docs site brand/package reproducibility gaps called out by reflection.
- Add a minimal local health check for the daemon port used by downstream installers.

## Ordered Changes

### 1. change-hardening-001-iroh-docs-share-import

**Scope:** `substrate/storage-provider`
**Depends on:** completed real `IrohDocsAdapter`
**Recommended agent:** Codex
**Estimated complexity:** M
**Complexity score:** 6/10
**Model class:** frontier
**Customer value:** HIGH

Expose the missing iroh-docs namespace transfer path so a second node can import the same document and actually sync the same keyspace. Add adapter methods for exporting a share ticket, importing a ticket, and a focused regression test that writes on one node and reads from another after sync.

**Details:**

- Use the official iroh-docs share/import API rather than ad hoc namespace bytes.
- Keep peer dialing and sync lifecycle explicit; avoid hidden background global state.
- Document any timing/retry expectations in the test helper.
- Preserve the existing `StorageProvider` behavior.

### 2. change-hardening-002-sovereign-sync-ci

**Scope:** `.github/workflows`, Rust substrate crates
**Depends on:** none
**Recommended agent:** Codex
**Estimated complexity:** S
**Complexity score:** 3/10
**Model class:** small
**Customer value:** HIGH

Add CI coverage for the sovereign-sync substrate crates so future adapter, MCP, REST, and install changes are gated by repeatable checks.

**Details:**

- Run `cargo fmt --check`, `cargo clippy`, and `cargo test` for the relevant workspace/crates.
- Include `substrate/storage-provider`, `substrate/sovereign-sync`, and adjacent crates required by dependency closure.
- Use stable Rust and cache dependencies without caching build outputs into source control.
- Keep global flags in documented CLI examples before subcommands.

### 3. change-hardening-003-mcp-client-pool-e2e

**Scope:** `substrate/sovereign-sync`
**Depends on:** existing MCP client pool implementation
**Recommended agent:** Codex
**Estimated complexity:** M
**Complexity score:** 5/10
**Model class:** frontier
**Customer value:** MEDIUM

Add an end-to-end test for `McpClientPool` that exercises outbound forwarding against a controlled local MCP server process or in-process test transport.

**Details:**

- Verify happy-path `call_tool` forwarding.
- Verify allowed-tools filtering.
- Verify failure propagation when the upstream server exits or returns an error.
- Keep the fixture deterministic and suitable for CI.

### 4. change-hardening-004-docusaurus-brand-and-lock

**Scope:** docs site / Docusaurus package
**Depends on:** existing Docusaurus scaffold
**Recommended agent:** Codex
**Estimated complexity:** S
**Complexity score:** 4/10
**Model class:** small
**Customer value:** MEDIUM

Apply the KnowMe visual identity to the Docusaurus site and make the package reproducible with a lockfile.

**Details:**

- Replace the generic purple theme with KnowMe brand tokens: Ember `#E04E28` / `#FF6A3D`, Conviction mark usage, and project font stack.
- Pin package versions and commit `package-lock.json`.
- Keep the page content functional and avoid adding marketing-only sections.
- Run the docs package build or validation command after changes.

### 5. change-hardening-005-daemon-health-detect-toolchain

**Scope:** install/detect tooling and `substrate/sovereign-sync`
**Depends on:** existing daemon/server mode
**Recommended agent:** Codex
**Estimated complexity:** S
**Complexity score:** 4/10
**Model class:** small
**Customer value:** MEDIUM

Add a minimal health check around the sovereign-sync daemon/server on localhost port `7892`, and wire it into detect-toolchain or installer diagnostics.

**Details:**

- Expose a stable health endpoint or status command if one is not already available.
- Detect occupied port `7892` and distinguish "healthy sovereign-sync" from "different process".
- Surface actionable diagnostics without auto-killing user processes.
- Add a lightweight test or fixture for the detection behavior.

## Round Order

Round 1:

1. `change-hardening-001-iroh-docs-share-import`
2. `change-hardening-002-sovereign-sync-ci`

Round 2:

3. `change-hardening-003-mcp-client-pool-e2e`
4. `change-hardening-004-docusaurus-brand-and-lock`

Round 3:

5. `change-hardening-005-daemon-health-detect-toolchain`

## Commands

OpenSpec change skeletons have been created. Continue with:

```bash
/kbd-apply change-hardening-001-iroh-docs-share-import
/kbd-apply change-hardening-002-sovereign-sync-ci
/kbd-apply change-hardening-003-mcp-client-pool-e2e
/kbd-apply change-hardening-004-docusaurus-brand-and-lock
/kbd-apply change-hardening-005-daemon-health-detect-toolchain
```

Fallback if OpenSpec automation is unavailable:

```bash
cat openspec/changes/change-hardening-001-iroh-docs-share-import/tasks.md
```

## Deferred

- Full Tauri sidecar plugin packaging
- WASM module packaging
- Flint Gate/Ory auth integration
- Embedding or LLM-intent matching for skill discovery
- Cloud Axum node deployment

PLAN COMPLETE
