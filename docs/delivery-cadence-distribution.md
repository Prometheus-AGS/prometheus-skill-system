# Delivery Cadence distribution

The full pack is the canonical source for the complete `delivery-cadence` skill. The targeted Node installer copies only that skill and its declared adapter dependencies. It does not activate hooks, change agent configuration, install services, or move the full pack's shared generation pointer.

From this repository, preview the installation and then apply it:

```text
node scripts/distribute-delivery-cadence.mjs --targets all --dry-run
node scripts/distribute-delivery-cadence.mjs --targets all
```

| Target ID | Global skill directory | Discovery basis |
| --- | --- | --- |
| `agents` | `~/.agents/skills/delivery-cadence` | Canonical skill root (owner decision D3), read directly by Codex, OpenCode, Kimi Code, MiniMax, Cursor, Gemini CLI, [Zed](https://zed.dev/docs/ai/skills) and other AgentSkills harnesses |
| `claude` | `~/.claude/skills/delivery-cadence` | Claude Code personal skill directory |
| `qwen` | `~/.qwen/skills/delivery-cadence` | Qwen Code personal skill directory |

The former per-harness IDs `codex`, `kimi-code`, `minimax`, `zed` and
`opencode` are accepted as aliases of `agents`, so existing invocations install
once into the canonical root instead of into retired per-harness directories.

Use a comma-separated subset with `--targets agents,claude`. `--home PATH` changes the home used for every destination. `--source-root PATH` selects another full-pack checkout or packaged full-pack distribution. The source contract must name `prometheus-skill-pack`; mini cannot use this global installer. The source layout is `skills/process/delivery-cadence` in the checkout or `skills/delivery-cadence` in a packaged distribution, alongside the declared pack adapters and their dependencies. Mini receives the shared skill through the existing full-to-mini synchronization workflow.

Every destination is a complete directory copy. The installer creates no symlinks and preserves existing parent-directory aliases. It refuses a symlink at the skill destination and refuses symlinks inside payloads. When `~/.agents/skills` is itself a symlink to a shared tree, the copy lands in the directory it names. Selected targets that resolve to the same physical directory are refused.

## Ownership and recovery

The `.cadence-install.json` receipt records the owner, schema version, source checkout, installation time, and SHA-256 of every installed payload file. A later installation compares the entire current file set with that receipt before replacing anything. Added, deleted, or edited files cause refusal. All selected destinations are preflighted before the first installation; a filesystem failure during application can still leave earlier targets successfully updated, so retain the output and rerun after resolving the failure.

An unmanaged destination is preserved. Only an exact byte-for-byte payload can be adopted using explicit `--adopt-identical`; custom copies must first be reconciled by their owner. Managed updates and identical adoption create checked backups under `~/.prometheus/delivery-cadence/backups/<run>/<target>`. Backups include the previous ownership receipt when present. An unchanged installation writes nothing. Dry run performs the same read-only preflight and reports planned paths/actions without creating directories or receipts.

For recovery, preserve the current destination separately and restore the complete reported backup directory to that destination. Keep its receipt with the files. The installer does not automatically delete backups or remove unrelated skills.

## Optional adapter support

The separate managed root `~/.prometheus/delivery-cadence/support` contains:

- `shared/scripts/cadence-kbd-adapter.mjs`
- `shared/scripts/cadence-karpathy-adapter.mjs`
- Their relative JavaScript import dependencies, and `shared/scripts/record-progress.mjs` with its dependencies when that recorder exists in the source.

The JSON installation result reports absolute adapter paths. Configure those paths deliberately when selecting KBD or Karpathy integration, using the skill's adapter reference. Distribution does not register hooks or change project bindings. Existing KBD authority and optional-service behavior remain in the adapters. This support directory is independent of both project Cadence state and `~/.prometheus/plugins/prometheus-skill-pack/current`.

Reload or start fresh native sessions after installation. On-disk receipt and checksum verification establishes copied payload availability; it does not establish that each native harness discovered or exercised the skill. Report those acceptance states separately.

## Pipeline upgrade (1.2.0 / state v3)

Complete shared source wiring before copying. The full-to-mini copier preflights all owned bytes, refuses changed/added/deleted user content, stages the full directory, and retains the previous directory plus ownership manifest in `.prometheus/delivery-cadence-backups/`. Retired unchanged owned files disappear only as part of that backed-up replacement. The payload digest identifies copied bytes even before source commits; record the eventual full and mini commits separately. Never describe the copier's source HEAD as the commit of uncommitted bytes.

The global installer recursively includes the KBD dispatch adapter and its imports for every target listed above. No new installer or extra service is required. Perform installation only after the completed production boundary, stop old mutators, then explicitly migrate the selected run with its backup. Updating source files alone does not migrate live state. Preserve v1/v2 event history, receipts, command IDs, unresolved effects and publication obligations. Update project bindings to the accepted payload deliberately.
