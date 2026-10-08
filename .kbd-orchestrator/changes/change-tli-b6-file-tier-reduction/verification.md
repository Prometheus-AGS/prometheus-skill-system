# Verification — change-tli-b6-file-tier-reduction

Repository: `prometheus-skill-system`
Depends on: `change-tli-b5-subagentstart-delivery-alpha`, `change-tli-b3b-agent-team-memory-envelope`

## Acceptance criteria

- Partitioning a copy of a ~14 KB index yields MEMORY.md ≤ 4,096 bytes; every removed line is returned by learning_recall for its addressed role.
- Exported Codex agent names match `^[a-z0-9_]+$` and `codex exec` in a scratch CODEX_HOME accepts the exported agent.
- The gate never modifies the real index; the live index is partitioned only in the user-confirmed task, and `evidence/b6-live-index-size.txt` records its size ≤ 4,096 bytes afterwards.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
curl -fsS -m 3 "${SURREAL_MEMORY_URL:-http://127.0.0.1:23001}/health" >/dev/null || { echo "BLOCKED: surreal-memory not reachable" >&2; exit 2; }
/bin/bash scripts/tests/test-memory-partition.sh
awk '/^after:/{exit !($2<=4096)}' "$TLI_KBD/phases/team-aware-learning-memory-impl/evidence/b6-live-index-size.txt"
cd skills/process/agent-team-creator/runtime && npm ci --silent && npm run build && npm run build:tests && node --test ../tests/export.integration.mjs
```
