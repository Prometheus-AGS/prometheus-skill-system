# KBD Constraints

Constraints the artifact-refiner QA gate validates for every change from
`phase-codex-plugin-distribution-and-ci` onward. Each completed change is checked
against these before archiving (`/refine-validate <change-id>`); a violation marks
the change BLOCKED.

## C-01 — Generated artifacts must be in sync

Any source consumed by a generator must have its generated outputs regenerated
and committed before the owning parent phase completes or its final phase
commit is pushed. A parent phase may name one later reconciliation change to batch this
work when several sibling changes edit the same generated surface; every earlier
change must identify that reconciliation change in persisted execution evidence,
and none may claim distribution certification in the interim. The reconciliation
change must run the generator twice, prove byte-identical tracked hashes, and run
the applicable drift validators before phase certification. A deferral that
crosses the parent-phase boundary, omits the named owner, or reaches parent-phase
certification, final commit, or push without regenerated outputs is a violation.

Specifically: after touching `.claude-plugin/*`, `.mcp.json`,
`hooks/hooks.json`, or `scripts/build-codex-plugin.js`, `npm run validate:codex`
MUST pass (no drift, valid manifest + marketplace) in the same change because
those files directly define the generator or already-generated plugin surface.
Hand-editing `.codex-plugin/plugin.json` or
`.agents/plugins/marketplace.json` is always a violation — they are generated.

## C-02 — No committed secrets

No API keys, tokens, or passwords in tracked files. Env vars and `${VAR:-default}`
placeholders only; real secret values live in the environment or user-local config
(`~/.codex/config.toml`, `~/.bash_profile`), never in the repo. Localhost dev
credentials that are already an established repo convention (e.g. surrealdb
`root/root`, the forge localhost dev token) are exempt.

## C-03 — Docs updated with surface changes

A change that alters the Codex plugin surface (manifest fields, marketplace
schema, MCP servers, hooks, install flow) must update `docs/codex-plugin.md` and,
where relevant, the CLAUDE.md "Codex CLI Integration" section in the same change.

## C-04 — Generators stay idempotent

`node scripts/build-codex-plugin.js` run twice must produce byte-identical output.
`--check` must exit non-zero on drift or invalid artifacts. New generator features
(e.g. source-type knobs) must preserve idempotency for a fixed input.

## C-05 — Scripts under launchd are bash 3.2 compatible

Any script that can be invoked by a launchd agent (macOS `/bin/bash` is 3.2) must
avoid `mapfile` / `declare -A`. Test with `/bin/bash script.sh`, not just `bash`.

## When QA is skipped

Per `/kbd-execute`: changes with fewer than 3 files modified, documentation-only
changes, or `--skip-qa`. Skips are logged, not silent. When the
`sycophancy-correction`/artifact-refiner binary is absent, the gate logs the skip
and passes (graceful degradation) — as in prior phases.
