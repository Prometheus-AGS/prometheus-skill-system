# Verification — change-cpc-011-integration-evidence

Repository: `prometheus-companion`  
Depends on: change-cpc-005-node-hosting, change-cpc-006-discovery-and-pairing, change-cpc-007-supervisor-and-health, change-cpc-009-pack-removal (the harness drives the pack CLI through its repointed production transport)

## Acceptance criteria

- `--test two_node` passes locally with two real processes; log shows both PIDs, mDNS resolution, gossip `NeighborUp`, the receipt hash on both nodes.
- `bash scripts/audit-all.sh` exits 0. No hosted CI is cited; commands and outputs recorded in this file.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository named above, locally, after the edit batch.

```verify
cargo test -p prometheus-substrate --features headless --test two_node
bash scripts/audit-all.sh
```

## Evidence

_Recorded at execution: commands, outputs, dates, commit hashes. Hosted CI is never cited._
