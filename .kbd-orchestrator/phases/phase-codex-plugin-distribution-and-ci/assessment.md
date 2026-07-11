# Assessment — phase-codex-plugin-distribution-and-ci

_Assessed 2026-07-11. Seeded from `phase-codex-plugin-implementation`'s reflection deltas. Method: repo inventory against the 6 goals._

## Headline

All six goals are **small, well-scoped integration tasks** — the plugin itself is
already generated + validated (prior phase). Feasibility **HIGH**. One goal (G-03,
hook-trust firing) is inherently a **manual interactive** check, not CI-automatable.

## Gap analysis (per goal)

| Goal | Current state | Gap | Effort |
|---|---|---|---|
| **G-01** wire `build:codex` into install-platforms | `scripts/install-platforms.ts` has a `codex` target (`name:'codex'`, `~/.codex/skills`, `.codex/skills`) but it installs **skills only** — never runs `build:codex`/regenerates `.codex-plugin/` + `.agents/plugins/` | add a step to the codex install path that runs `npm run build:codex` (or imports the generator) so install provisions the plugin artifacts | **S–M** |
| **G-02** `validate:codex` in CI | `.github/workflows/validate.yml` runs `npm run validate`, `validate:signals`, `validate:strict` — **no `validate:codex`** | add a `run: npm run validate:codex` step to `validate.yml` (fails on stale/invalid Codex artifacts) | **S** |
| **G-03** manual hook-trust firing check | no evidence; hooks are non-managed (interactive trust) — cannot be asserted headlessly (prior-phase finding) | perform an interactive `codex` session, trust the plugin, confirm a `SessionStart` hook writes to `${PLUGIN_DATA}`, record evidence under `references/` | **S (manual)** |
| **G-04** env-provisioning helper | `scripts/configure-mcp-all-tools.sh` configures codex MCP (`[mcp_servers.*]`) and `kbd-goal-codex-setup.sh` exists — but env-key seeding for the plugin's 7 servers (tavily key, forge token, etc.) from the environment is **not** consolidated | extend/author a helper that seeds `~/.codex/config.toml` env (or documents it) for the 7 servers from the environment; **no committed secrets** (mirror the tavily/firecrawl setup done this session) | **M** |
| **G-05** git-subdir/git marketplace sources | `build-codex-plugin.js` hardcodes `source: { source: 'local', path }` | make the source type configurable (e.g. env/flag or per-plugin override) to emit `git-subdir`/`git` sources for external publish; keep `local` default for dogfood | **S–M** |
| **G-06** `constraints.md` QA gate | **absent** — artifact-refiner never runs (0/8 last phase) | author a lightweight `.kbd-orchestrator/constraints.md` (e.g. generated-artifacts-must-be-in-sync, no-committed-secrets, docs-updated) so future phases get a real QA gate | **S** |

## Key observations / risks

- **G-01 and G-04 overlap the existing platform tooling** (`install-platforms.ts`, `configure-mcp-all-tools.sh`) — extend, don't duplicate. Check bash-3.2 compatibility for any launchd-invoked path (per CLAUDE.md Codex note).
- **G-03 is a documentation/evidence task, not code** — it will produce a `references/hook-trust-verification.md`, not a passing CI check. Plan should mark it explicitly manual so it isn't mistaken for automatable.
- **G-05 risk:** `git-subdir` sources require the marketplace to reference a committed/pushed commit — only meaningful once the plugin is published externally. Could be scoped as "support + document" rather than "switch the default."
- **G-02 depends on G-01 lightly:** if CI runs `validate:codex`, the generated artifacts must be committed (they are, as of `4a6ca87`) — good.

## Suggested change decomposition (input to /kbd-plan)

~5 changes: (1) `constraints.md` [G-06, do first — enables QA for the rest]; (2) CI `validate:codex` step [G-02]; (3) install-platforms `build:codex` integration [G-01]; (4) generator `git-subdir`/`git` source support [G-05]; (5) env-provisioning helper [G-04]. Plus (6) the manual hook-trust verification [G-03] as a doc/evidence change.

## Overall

Low-risk closeout of the prior phase's caveats. Mostly small CI/tooling edits reusing
existing platform scripts. The only non-code item is the manual hook-trust verification,
which should be planned as an explicit manual step with an evidence artifact.
