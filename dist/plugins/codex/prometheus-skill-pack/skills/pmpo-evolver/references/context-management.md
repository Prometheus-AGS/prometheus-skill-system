# Context Management Reference

Seven rules for keeping pmpo-evolver sessions within context limits when running long evolution cycles.

---

## The Core Problem

Evolution cycles accumulate context fast:
- Feedback source raw outputs (gh issues, commit logs, changelogs) can be large
- Competitor registry and parity matrix grow over time
- Learning signals accumulate across ticks
- Carrying all of this inline exhausts the session window

**The solution:** run all heavyweight collection as isolated subprocesses; the evolver session reads only the JSON output.

---

## Rule 1: Never inline raw collection output

```bash
# WRONG — inlines potentially huge git log into evolver context
GIT_LOG=$(git log --oneline --since="30 days ago")

# CORRECT — subprocess handles it, returns compact JSON
COMMIT_SIGNAL=$(bash scripts/commit-history-analyze.sh . --since "30 days ago")
```

All collection scripts (`feedback-digest.sh`, `commit-history-analyze.sh`, `changelog-fetch.sh`) follow this pattern.

---

## Rule 2: Read only structured summaries from feedback sources

The evolver session reads:
- `{collected, high_severity_count, new_signals[]}` from `feedback-digest.sh`
- `{feasibility_score, recommendation}` from Gate 2
- `{lessons_added}` from `post-cycle-dream.sh`

It does NOT read raw issue bodies, raw commit messages, or raw changelog text.

---

## Rule 3: Assign the smallest adequate model class

Use the liter-llm class table (see `model-routing.md`) and never upgrade without cause.

Upgrading from `small` to `frontier` for a classification task wastes 10-50× tokens. The classes exist to prevent this.

---

## Rule 4: Run dreaming as an isolated subprocess

`post-cycle-dream.sh` ingests journal.md + reflection.md + existing lessons — potentially thousands of tokens. Run it isolated and read only `{lessons_added, lessons_file}`.

---

## Rule 5: Scope competitor registry reads

Never read the full competitor registry inline. Instead:
- Extract only the fields needed: `jq '[.[] | {id, name, last_scanned}]' registry.json`
- Pass only relevant competitor IDs to changelog-fetch.sh

---

## Rule 6: Archive idea specs to disk, not memory

Gate 3 specs (SPEC.md) are written to `.evolver/<name>/archive/<idea-id>/SPEC.md`. The evolver session holds only the path, not the content.

---

## Rule 7: Compress learning signals before synthesis

Before passing learning signals to a synthesis step, compress the array to essentials:

```bash
# From full signals, extract only signal + severity for synthesis
SIGNALS_COMPACT=$(echo "${SIGNALS_JSON}" | python3 -c "
import json, sys
signals = json.load(sys.stdin).get('new_signals', [])
compact = [{'source': s['source_type'], 'signal': s['signal'], 'severity': s['severity']} for s in signals[:5]]
print(json.dumps(compact))
")
```

---

## Cost estimation table

Approximate token costs per evolver operation (GPT-4o class as baseline):

| Operation | Tokens in | Tokens out | Class | Cost est. |
|-----------|-----------|------------|-------|-----------|
| Plausibility check (Gate 1) | ~200 | ~50 | small | < $0.001 |
| Commit classification | ~500 | ~100 | small (none) | $0 (regex) |
| Feedback synthesis | ~1,000 | ~200 | medium | ~$0.01 |
| Gate 2 domain research | ~2,000 | ~500 | medium | ~$0.05 |
| Gate 3 spec generation | ~3,000 | ~1,000 | frontier | ~$0.20 |
| Strategic dreaming | ~4,000 | ~800 | frontier | ~$0.30 |
| Competitor analysis | ~2,000 | ~500 | frontier | ~$0.20 |
| Parity scoring | ~1,500 | ~300 | frontier | ~$0.15 |

**Cost per full evolver cycle:** approximately $1-3 depending on perspective and model availability.

**To reduce cost:** use `class=small` or `class=medium` where frontier is not required. Set `PROMETHEUS_EVOLVER_MAX_CLASS=medium` in the environment to cap all calls at medium (useful for daily monitoring cycles).
