# Update Considerations — Verified Skill-Pack Distribution

> **Status:** implementation complete; activation is performed from the focused
> clean source commit.
> **Updated:** 2026-08-09
> **Audience:** maintainers of the source, immutable-generation installer, and
> native plugin projections.

## Incident and root cause

`scripts/update-skill-pack.sh` originally failed while verifying
`shared/scripts/detect-toolchain.sh`. Commit `3c31581` changed that file and
`shared/scripts/service-probe.sh` without regenerating
`shared/harnesses/generated/release-manifest.json`. The manifest therefore held
two stale hashes. Regenerating the harness adapters made all runtime-file hashes
consistent.

The handoff also reported the same error once after regeneration. Code tracing
refuted the two leading explanations:

- `install-plugin-generation.js` copies the live source tree, not `git archive`
  or another HEAD-only payload.
- Every run uses a unique `.staging-<pid>-<random>` directory and removes it on
  failure, so a reused staging cache cannot explain the repeat.

The repeat no longer reproduces and there is no evidence of a second persistent
manifest defect. The source tree was concurrently changing during the original
investigation, while the old installer recorded only `modified` provenance and
did not require the source commit to remain stable. That consistency gap is now
closed rather than assigning an unsupported more-specific cause.

## Installer guarantees

Release updates now differ deliberately from development installs:

- `install-plugin-generation.js --require-clean-source` rejects a modified
  source before and after staging.
- `--expected-source-commit <sha>` pins the commit recorded in the generation.
- Source commit, clean/modified state, and submodule pins are captured before and
  after staging; any difference aborts activation.
- Release-payload errors report the expected and actual hash and mode plus the
  staged payload and manifest paths.
- Direct installer calls without the release flags can still package a stable
  dirty tree for development, and their manifest records `sourceTreeState` as
  `modified`.

`update-skill-pack.sh` is the release entrypoint. It now:

1. refuses a dirty tree before `git pull --ff-only`;
2. updates submodules and verifies the tree is still clean;
3. checks generated harness and Codex artifacts without rewriting them;
4. installs with the clean-source and expected-commit requirements;
5. verifies the active generation, clean provenance, and all 14 target receipts;
6. refreshes every detected native plugin surface through its supported tool;
7. writes `~/.prometheus/skill-pack-install-ref` only after those checks pass.

`--force` remains accepted for compatibility and requests a full native refresh.
It never bypasses source, manifest, signature, provenance, or receipt checks.

## Native plugin and cache policy

The immutable generation remains authoritative for these 14 targets:

```
.claude/skills                 .opencode/skills
.kimi-code/skills              .minimax/skills
.cursor/skills                 .codex/skills
.gemini/skills                 .roo/skills
.windsurf/skills               .codeium/windsurf/skills
.agents/skills                 .config/zed/skills
.zed/skills                    .cline/skills
```

Twelve targets link through `~/.prometheus/plugins/prometheus-skill-pack/current`.
Codex and MiniMax receive receipt-bearing real-directory copies.

Native plugin stores require separate supported refresh operations:

- **Claude Code:** update the directory marketplace, then update the installed
  umbrella and process plugins through `claude plugin`. Never edit
  `~/.claude/plugins/cache` directly. The umbrella advances from the installed
  1.6.2 payload to 1.7.0; the affected process slice advances to 1.5.1. Because
  `plugin list` can briefly return the prior registry state after a successful
  update, verification retries that supported CLI query for a bounded period.
- **Codex:** the registered local marketplace resolves this checkout directly.
  Re-add already installed umbrella and process plugins through `codex plugin
  add`, then verify their source paths and versions. This refreshes Codex-owned
  registration state through the CLI; the updater never edits its cache.
- **Kimi Desktop:** rerun its idempotent app-package installer and compare the
  complete bundled `kbd-init` payload to the source.
- **Absent tools:** report an explicit skip. A refresh or verification failure
  for a detected installed surface is fatal and prevents the install-ref receipt
  from advancing.

Old immutable cache versions are retained for host-managed rollback. Consumers
must resolve the new installed version or the active generation before the
update reports success.

## Legacy installer reconciliation

The former Codex WatchPaths LaunchAgent used `.prometheus-pack`, while the
generation installer uses `.prometheus-generation`. It consequently classified
canonical generation copies as user collisions and could not refresh them.

The immutable generation is now the sole global skill-copy authority:

- global multi-platform installs delegate to `install-plugin-generation.js`;
- project-scoped installs retain their prior repository-local behavior;
- new installs no longer register `ai.prometheus.codex-skills-sync`;
- updates remove that known legacy LaunchAgent if present;
- `codex-sync-skills.sh` recognizes generation receipts and leaves those copies
  alone without a false collision warning.

## Regression coverage

Local fixtures verify:

- standalone `kbd-init` resources are complete and match canonical references;
- missing mandatory volumes preserve paths and block execution without a local
  target fallback;
- clean commit pinning succeeds and dirty release installation fails;
- stale generated artifacts stop before installation;
- a provenance change during staging fails;
- native refresh failure leaves the install-ref unchanged;
- a briefly stale Claude registry listing converges before verification;
- Claude, Codex, and Kimi use their supported refresh paths, including Codex
  re-add ordering for installed affected plugins;
- no native refresher contains a direct Claude cache path;
- generation-owned Codex copies are not treated as user collisions.

All certification and distribution evidence is produced locally. Hosted CI is
not used as a test, diagnosis, or release gate.

## Final activation evidence

Activation evidence is intentionally not embedded as a generated hash in this
tracked file: changing the file would create a new source commit and therefore a
new content-addressed generation. The non-circular authorities are the signed
active-generation manifest, its 14 signed target receipts,
`~/.prometheus/skill-pack-install-ref`, and the updater's local terminal result.

Final verification compares those receipts to the focused commit, checks the
Claude and Codex installed versions, compares every installed `kbd-init` payload
to the committed source, and reruns the Compass validator. The resulting hashes
and blocked-volume result belong in the release/task report, not back in the
payload whose hash they identify.
