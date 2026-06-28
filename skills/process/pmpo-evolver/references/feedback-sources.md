# Feedback Sources Reference

Reference for all `feedback_sources[].type` values supported in `loop.json`. Each type produces a normalized `LearningSignal` consumed by the pmpo-evolver self-learning perspective.

## Normalization output (all types → LearningSignal)

```json
{
  "source_type": "gh-issues",
  "source_ref": "owner/repo",
  "collected_at": "ISO8601",
  "signal": "1-2 sentence summary of what the signal means for product direction",
  "severity": "high | medium | low",
  "count": 42,
  "examples": ["up to 5 example strings"],
  "model_used": "small | medium | none"
}
```

---

## gh-issues

Counts open GitHub issues matching label/state filters. Sentiment-classifies titles.

```json
{
  "type": "gh-issues",
  "repo": "owner/repo",
  "labels": ["bug", "user-feedback"],
  "state": "open",
  "since": "2026-01-01",
  "interpret": "A rising count of 'bug' issues indicates quality regression. High count = high severity signal.",
  "staleness_ttl_minutes": 30
}
```

**Collection:** `gh api repos/<owner>/<repo>/issues --jq '[.[] | select(.state=="open")]'`

**Model routing:** `[MODEL_ROUTING] phase=evolver-signal-gh-issues class=medium` — sentiment classification of titles

**Staleness TTL default:** 30 minutes

---

## commit-history

Analyzes git log to produce a commit-type histogram and identify churn hotspots.

```json
{
  "type": "commit-history",
  "repo_path": ".",
  "since": "2026-06-01",
  "classify_by": "conventional-commits",
  "interpret": "A fix_ratio above 0.35 signals quality debt. High churn files are candidates for refactor.",
  "staleness_ttl_minutes": 60
}
```

**Collection:** `bash scripts/commit-history-analyze.sh <repo_path> --since <date>`

**Output:** `{period, total_commits, breakdown: {fix, feat, refactor, chore, test, docs}, fix_ratio, hotspots: [{file, fix_count}]}`

**Model routing:** `[MODEL_ROUTING] phase=evolver-signal-commits class=small` — no LLM needed for conventional-commit classification; `class=medium` for `classify_by: llm`

**Staleness TTL default:** 60 minutes

---

## sentiment-feed

Fetches an RSS, JSON, or CSV feed and classifies item sentiment.

```json
{
  "type": "sentiment-feed",
  "url": "https://example.com/feedback.rss",
  "format": "rss",
  "sentiment_field": "description",
  "interpret": "Negative sentiment trend on feature X suggests user friction. Majority negative = high severity.",
  "staleness_ttl_minutes": 60
}
```

**Collection:** `curl -s <url>` then parse per format

**Model routing:** `[MODEL_ROUTING] phase=evolver-signal-sentiment class=medium` — NLP sentiment classification

**Staleness TTL default:** 60 minutes

---

## telemetry-url

Fetches a JSON API endpoint and extracts a numeric or string value via JSONPath for comparison against a baseline.

```json
{
  "type": "telemetry-url",
  "url": "https://api.example.com/metrics/errors",
  "headers": {"Authorization": "Bearer $TELEMETRY_TOKEN"},
  "jsonpath": "$.data.error_rate",
  "direction": "lower-is-better",
  "baseline": 0.01,
  "interpret": "Error rate above baseline = high severity. Below = low.",
  "staleness_ttl_minutes": 15
}
```

**Collection:** `curl -s -H "..." <url>` + JSONPath extraction

**Model routing:** `[MODEL_ROUTING] phase=evolver-signal-telemetry class=small` — deterministic comparison, no LLM

**Staleness TTL default:** 15 minutes

---

## competitor-scan

Reads the competitor registry and optionally triggers web search for each competitor. Diffs against last scan to detect new features.

```json
{
  "type": "competitor-scan",
  "competitor_ids": ["competitor-a", "competitor-b"],
  "registry_path": ".evolver/my-product/competitor-registry.json",
  "interpret": "New features in direct competitors require parity evaluation. Each new feature = potential gap.",
  "staleness_ttl_minutes": 1440
}
```

**Collection:** reads `competitor-registry.json`, checks `last_scanned` vs TTL, runs `changelog-fetch.sh` per competitor if stale

**Model routing:** `[MODEL_ROUTING] phase=evolver-competitive-scan class=frontier` — cross-domain synthesis to assess competitive significance

**Staleness TTL default:** 1440 minutes (once per day)

---

## changelog

Fetches GitHub releases or a local CHANGELOG.md file since the last known tag and extracts feature additions.

```json
{
  "type": "changelog",
  "repo": "anthropics/anthropic-sdk-python",
  "since_tag": "v0.40.0",
  "format": "github-releases",
  "interpret": "New SDK features may enable capabilities we should adopt. Breaking changes require migration planning.",
  "staleness_ttl_minutes": 720
}
```

**Collection:** `bash scripts/changelog-fetch.sh <repo> --since-tag <tag>`

**Model routing:** `[MODEL_ROUTING] phase=evolver-changelog-extract class=medium` — structured feature extraction from release notes

**Staleness TTL default:** 720 minutes (twice per day)

---

## Staleness TTL guidance

| Source type | Recommended TTL | Rationale |
|-------------|----------------|-----------|
| `gh-issues` | 30 min | Issues are created frequently; fresh data matters |
| `commit-history` | 60 min | Commits accumulate slowly; 1hr is sufficient |
| `sentiment-feed` | 60 min | RSS feeds typically update hourly |
| `telemetry-url` | 15 min | Telemetry is near-real-time; use short TTL |
| `competitor-scan` | 1440 min (1 day) | Competitor features don't change hourly |
| `changelog` | 720 min (12hr) | Releases happen infrequently |

Set `staleness_ttl_minutes: 0` to always re-fetch regardless of recency.

---

## Model routing summary

| Source type | Signal step | liter-llm class |
|-------------|-------------|----------------|
| `gh-issues` | Count delta | `small` |
| `gh-issues` | Title sentiment | `medium` |
| `commit-history` | Classification | `small` |
| `sentiment-feed` | Sentiment | `medium` |
| `telemetry-url` | Value comparison | `small` (none) |
| `competitor-scan` | Competitive significance | `frontier` |
| `changelog` | Feature extraction | `medium` |
| All sources | Signal synthesis (what do these collectively mean?) | `frontier` |
