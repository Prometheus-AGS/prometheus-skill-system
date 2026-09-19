# Verification — change-cpc-009-pack-removal

Repository: `prometheus-skill-pack`  
Depends on: change-cpc-004-relocate-sovereign-sync, change-cpc-012-pack-sync-skills-removal

## Acceptance criteria

- The footprint command prints nothing: whole repo, excluding .git, node_modules, target, .kbd-orchestrator, .prometheus, .refiner, dist, memory, openspec/changes/archive; ignoring the three contract-v1 identifiers; allowing docs/integration-contract.md, docs/decisions, docs/audits, docs/releases, docs/reports, CHANGELOG.md.
- `node scripts/generate-service-manifest.mjs --check` exits 0 after the plist and unit edits.
- `--test doctor` passes with a case asserting no sovereign-sync item and exit 0 when the Companion is absent; `test-service-exclusions.sh` passes.
- `npm run validate:codex` and `npm run check:skills-index` pass; `docs/codex-plugin.md` describes the install flow without `--sharing`.
- `prometheus kbd --path . status --json` and one typed mutation succeed on this repo with the Companion absent.

- `docs/decisions/sovereign-sync-relocated-to-companion.md` exists and records the retirement of the four OpenSpec main specs, whose directories are gone.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository named above, locally, after the edit batch.

```verify
test -f docs/decisions/sovereign-sync-relocated-to-companion.md && ! test -d openspec/specs/sovereign-sync-daemon-health && ! test -d openspec/specs/sovereign-sync-ci && ! test -d openspec/specs/iroh-docs-adapter && ! test -d openspec/specs/mcp-client-pool
node scripts/generate-service-manifest.mjs --check
cargo test --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test doctor && bash shared/scripts/tests/test-service-exclusions.sh
npm run validate:codex && npm run check:skills-index && test -z "$(grep -rnE 'sovereign-sync|sovereign_sync' . --exclude-dir=.git --exclude-dir=node_modules --exclude-dir=target --exclude-dir=.kbd-orchestrator --exclude-dir=.prometheus --exclude-dir=.refiner --exclude-dir=dist --exclude-dir=memory --exclude-dir=archive | grep -vE 'sovereign-sync\.sock|SOVEREIGN_SYNC_SOCKET|sovereign-sync/device-key\.json' | grep -vE '^\./(docs/(integration-contract\.md|decisions/|audits/|releases/|reports/)|CHANGELOG\.md)')"
prometheus kbd --path . status --json | jq -e '.revision' && prometheus kbd --path . stage enter --command-id cpc-009-proof-$(date +%s) --phase control-plane-to-companion --id cpc-009-proof --title proof
```

## Evidence

_Recorded at execution: commands, outputs, dates, commit hashes. Hosted CI is never cited._
