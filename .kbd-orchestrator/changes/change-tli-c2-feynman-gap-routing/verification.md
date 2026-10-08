# Verification — change-tli-c2-feynman-gap-routing

Repository: `prometheus-skill-system`
Depends on: `change-tli-b7-cross-agent-awareness-beta`

## Acceptance criteria

- A problem prompt on an empty KB emits one gap line; a repeat emits none.
- After ingesting a covering document, no gap line; output equals `pk context --format hook`.
- A gap recorded inside a subagent carries its role in gaps.jsonl.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
command -v pk >/dev/null || { echo "BLOCKED: pk not on PATH" >&2; exit 2; }
pk --version 2>/dev/null | grep -Eq "1\.(1[0-9]|[2-9][0-9])\." || { echo "BLOCKED: pk >= 1.10.0 required (A5a not installed)" >&2; exit 2; }
/bin/bash shared/scripts/tests/test-prompt-gap.sh
npm run check:distribution
```
