# Verification — change-cpc-010-kbd-state-migration

Repository: `prometheus-skill-pack`  
Depends on: none

## Acceptance criteria

- Before/after `prometheus kbd status --json` revision and frontier are identical; seven receipts recorded under `.kbd-orchestrator/archives/` or as adopted markers.
- A typed stage transition after migration emits no `refusing to overwrite` line.
- `--test kbd` passes including the new migration case.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository named above, locally, after the edit batch.

```verify
cargo test --manifest-path tools/prometheus-cli/Cargo.toml -p prometheus-cli --test kbd
prometheus kbd --path . status --json | jq -r '.revision,.frontier' > /tmp/before && prometheus kbd --path . migrate --projections && prometheus kbd --path . status --json | jq -r '.revision,.frontier' | diff - /tmp/before && test $(ls .kbd-orchestrator/archives/*.receipt.json 2>/dev/null | wc -l) -ge 7 && ( prometheus kbd --path . stage enter --command-id cpc-010-proof-$(date +%s) --phase control-plane-to-companion --id cpc-010-proof --title proof 2>&1 | grep -c 'refusing to overwrite' | grep -q '^0$' )
```

## Evidence

_Recorded at execution: commands, outputs, dates, commit hashes. Hosted CI is never cited._
