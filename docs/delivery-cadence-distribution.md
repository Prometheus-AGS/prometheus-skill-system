# Delivery Cadence distribution

The full pack is the canonical source for the complete `delivery-cadence` skill. The targeted Node installer copies only that skill and its declared adapter dependencies. It does not activate hooks, change agent configuration, install services, or move the full pack's shared generation pointer.

From this repository, preview the installation and then apply it:

```text
node scripts/distribute-delivery-cadence.mjs --targets all --dry-run
node scripts/distribute-delivery-cadence.mjs --targets all
```

| Target ID | Global skill directory | Discovery basis |
| --- | --- | --- |
| `codex` | `~/.codex/skills/delivery-cadence` | Existing full-pack Codex copy convention |
| `claude` | `~/.claude/skills/delivery-cadence` | Claude Code personal skill directory |
| `kimi-code` | `$KIMI_CODE_HOME/skills/delivery-cadence`, default `~/.kimi-code/skills/delivery-cadence` | [Kimi native skills](https://github.com/moonshotai/kimi-code/blob/main/docs/en/customization/skills.md) |
| `minimax` | `~/.minimax/skills/delivery-cadence` | Existing `scripts/install-minimax-skills.js` copy and `_meta.json` convention |
| `zed` | `~/.agents/skills/delivery-cadence` | [Zed native skills](https://zed.dev/docs/ai/skills); this shared directory also serves compatible agents |
| `opencode` | `$XDG_CONFIG_HOME/opencode/skills/delivery-cadence`, default `~/.config/opencode/skills/delivery-cadence` | [OpenCode native skills](https://opencode.ai/docs/skills) |

Use a comma-separated subset with `--targets codex,claude`. `--home PATH` installs into an isolated home and deliberately ignores the real session's `KIMI_CODE_HOME` and `XDG_CONFIG_HOME`; it is suitable for an integration sandbox. `--source-root PATH` selects another full-pack checkout or packaged full-pack distribution. The source contract must name `prometheus-skill-pack`; mini cannot use this global installer. The source layout is `skills/process/delivery-cadence` in the checkout or `skills/delivery-cadence` in a packaged distribution, alongside the declared pack adapters and their dependencies. Mini receives the shared skill through the existing full-to-mini synchronization workflow.

Every destination is a complete directory copy. The installer creates no symlinks and preserves existing parent-directory aliases. It refuses a symlink at the skill destination and refuses symlinks inside payloads. MiniMax receives the existing numeric skill ID convention, the skill version, `platform: minimax`, and an `updated_at` timestamp derived from the source skill file modification time so repeated installations remain unchanged.

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
