# Learning Signals Reference

> **⚠️ CORRECTED — `liter-llm complete` DOES NOT EXIST.**
> The `liter-llm` binary ships exactly two subcommands, `api` and `mcp`; it is a
> proxy *server*, not a completion CLI. There is also no `mcp-call` or
> `list_models` subcommand, and the MCP chat tool is named `chat`, not `complete`.
> Any `liter-llm complete ...` snippet below is **historical and non-functional** —
> callers paired it with `2>/dev/null || echo "{}"`, so the contract mismatch was
> invisible and results were silently empty.
>
> Use the shared helper instead, which speaks OpenAI REST to the resolved gateway
> and reports failures rather than swallowing them:
>
> ```bash
> . "${CLAUDE_PLUGIN_ROOT}/shared/scripts/lib/kbd-model-resolve.sh"
> out="$(kbd_complete "$(kbd_resolve_role critic)" "$SYSTEM" "$USER" 2048)" || {
>   echo "model call failed (see message above)" >&2
> }
> ```
>
> Configure models with `/liter-llm-bridge configure`. See
> `skills/process/adversarial-review/references/model-configuration.md`.


Protocol for collecting, normalizing, and persisting learning signals from feedback sources in the `self-learning` evolution perspective.

## LearningSignal normalization format

All source types produce this common shape:

```json
{
  "id": "uuid-or-timestamp",
  "source_type": "gh-issues | commit-history | sentiment-feed | telemetry-url | competitor-scan | changelog | research-query | usage-trace",
  "source_ref": "owner/repo, URL, or file path",
  "collected_at": "ISO8601",
  "signal": "1-2 sentence human-readable summary of what this signal means for product direction",
  "severity": "high | medium | low",
  "count": 42,
  "examples": ["up to 5 concrete examples from the source"],
  "model_used": "small | medium | frontier | none"
}
```

**Severity rules:**
- `high` — trend that demands immediate product attention (e.g., >40% fix ratio, competitor shipped major feature we lack)
- `medium` — worth including in next planning cycle
- `low` — informational; monitor but no immediate action needed

---

## Collection protocol per source type

### gh-issues

```bash
# Collect
ISSUES=$(gh api repos/<owner>/<repo>/issues \
  --jq '[.[] | select(.state=="open") | {number, title, labels: [.labels[].name], created_at}]')

# Count
ISSUE_COUNT=$(echo "${ISSUES}" | python3 -c "import json,sys; print(len(json.load(sys.stdin)))")

# Classify titles
# [MODEL_ROUTING] phase=evolver-signal-gh-issues class=medium
SENTIMENT=$(echo "${ISSUES}" | python3 -c "
import json,sys
items = json.load(sys.stdin)
titles = [i['title'] for i in items[:20]]
print('\n'.join(titles))
" | kbd_complete "$(kbd_resolve_role critic)" \
  --system 'Classify these GitHub issue titles into themes. Output JSON: {"themes": [{"name": "string", "count": int, "sentiment": "negative|neutral|positive"}]}')
```

**Severity:** count > 50 = high; > 20 = medium; else low

---

### commit-history

```bash
# [MODEL_ROUTING] phase=evolver-signal-commits class=small
bash scripts/commit-history-analyze.sh <repo_path> --since <ISO8601>
# Output: {period, total_commits, breakdown, fix_ratio, hotspots}
```

**Severity:** fix_ratio > 0.4 = high; > 0.25 = medium; else low

**Interpretation:** Fix ratio > 40% signals that more time is spent on bugs than features — quality debt requiring architectural attention.

---

### sentiment-feed

```bash
# Fetch RSS/JSON/CSV
ITEMS=$(curl -s <url> | python3 parse-feed.py --format <rss|json|csv>)

# [MODEL_ROUTING] phase=evolver-signal-sentiment class=medium
SENTIMENT=$(echo "${ITEMS}" | kbd_complete "$(kbd_resolve_role critic)" \
  --system 'Classify the sentiment of these user feedback items. Output JSON: {"overall_sentiment": "positive|mixed|negative", "negative_themes": ["string"], "positive_themes": ["string"], "total_items": int}')
```

**Severity:** overall_sentiment=negative = high; mixed = medium; positive = low (informational)

---

### telemetry-url

```bash
# Fetch and extract value
VALUE=$(curl -s -H "Authorization: Bearer $TOKEN" <url> | \
  python3 -c "import json,sys; d=json.load(sys.stdin); print(d['<jsonpath>'])")

# Compare against baseline (no LLM needed)
# [MODEL_ROUTING] phase=evolver-signal-telemetry class=small (none)
python3 -c "
baseline = <baseline>
value = float('${VALUE}')
direction = '<higher-is-better|lower-is-better>'
if direction == 'lower-is-better':
    if value > baseline * 1.2: severity = 'high'
    elif value > baseline * 1.05: severity = 'medium'
    else: severity = 'low'
else:
    if value < baseline * 0.8: severity = 'high'
    elif value < baseline * 0.95: severity = 'medium'
    else: severity = 'low'
print(severity)
"
```

---

### Learning signal synthesis

After all sources collected:

```bash
# [MODEL_ROUTING] phase=evolver-signal-synthesis class=medium
# Synthesize what signals collectively mean for product direction
ALL_SIGNALS_JSON="<normalized LearningSignal[] as JSON>"
SYNTHESIS=$(echo "${ALL_SIGNALS_JSON}" | kbd_complete "$(kbd_resolve_role critic)" \
  --system 'Given these normalized learning signals, synthesize the top 3 product direction implications. Output JSON: {"implications": [{"implication": "string", "confidence": "high|medium|low", "supporting_signals": ["source_type"]}]}')
```

---

## Persistence

**Per-tick archival:** `.evolver/<name>/learning-signals-<timestamp>.json`

**Evolution state:** append to `evolution_state.learning_signals[]` in `.evolver/<name>/state.json`

**Staleness:** before collecting a source, check `staleness_ttl_minutes` against `collected_at` of the most recent signal from that source. If within TTL, skip collection.

---

## Using feedback-digest.sh

The `feedback-digest.sh` script reads `loop.json` feedback_sources and runs collection for all configured types in one call:

```bash
bash scripts/feedback-digest.sh <evolution-name>
# Output: {collected: N, high_severity_count: N, new_signals: [...]}
```

Run as an isolated subprocess (not inline) to protect main session context:

```bash
DIGEST=$(bash skills/process/pmpo-evolver/scripts/feedback-digest.sh "${EVOLUTION_NAME}")
HIGH_COUNT=$(echo "${DIGEST}" | python3 -c "import json,sys; print(json.load(sys.stdin)['high_severity_count'])")
```
