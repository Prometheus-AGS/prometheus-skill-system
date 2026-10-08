# Verification — change-tli-c3a-pk-skill-discovery

Repository: `prometheus-knowledge-rs`
Depends on: `change-tli-c1a-pk-promotion-candidates`

## Acceptance criteria

- Three similar report transcripts across two projects yield one new-skill candidate.
- A `Skill(kbd-plan)` followed by corrections yields one update candidate attributed to its role.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi
cargo test -p pk-learning-worker --test skill_discovery
```
