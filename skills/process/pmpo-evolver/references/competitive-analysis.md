# Competitive Analysis Reference

Protocol for the `competitive` evolution perspective: registry format, parity matrix format, changelog ingestion, and model routing.

## Competitor registry

`.evolver/<name>/competitor-registry.json`:

```json
{
  "evolution_name": "my-product",
  "last_updated": "2026-06-28T00:00:00Z",
  "competitors": [
    {
      "id": "competitor-a",
      "name": "Competitor A",
      "url": "https://competitor-a.example.com",
      "github_repo": "owner/competitor-a",
      "category": "direct",
      "last_scanned": "2026-06-27T12:00:00Z",
      "last_changelog_tag": "v2.3.0",
      "feature_claims": [
        "Real-time collaboration",
        "Plugin marketplace",
        "One-click deploy"
      ],
      "notes": "Main competitor; focus on their plugin ecosystem"
    }
  ]
}
```

**Category values:**
- `direct` — same target user, same problem domain
- `adjacent` — overlapping user base, different approach
- `aspirational` — where the market is heading; not yet a direct threat

**Initialization:** `bash scripts/competitor-registry-init.sh <evolution-name>` creates a stub. Edit manually to add competitor entries.

---

## Parity matrix

`.evolver/<name>/parity-matrix.json`:

```json
{
  "evolution_name": "my-product",
  "last_updated": "2026-06-28T00:00:00Z",
  "features": [
    {
      "id": "feat-001",
      "name": "Plugin marketplace",
      "category": "distribution",
      "our_status": "missing",
      "competitors": {
        "competitor-a": "has",
        "competitor-b": "partial"
      },
      "priority": "high",
      "effort_estimate": "l",
      "last_updated": "2026-06-28T00:00:00Z",
      "source_signal": "competitor-a changelog v2.3.0 + user feedback"
    }
  ]
}
```

**Status values:** `has | missing | partial | better | n/a`

**Effort estimates:** `xs | s | m | l | xl`

**Priority:** Set based on: competitor advantage size + user demand signal + strategic fit

---

## Changelog ingestion protocol

Fetches competitor release history and extracts structured feature data.

### Steps

1. Run `bash scripts/changelog-fetch.sh <owner/repo> --since-tag <last-tag> --evolution-name <name>`
2. Script calls GitHub Releases API: `gh api repos/<owner>/<repo>/releases`
3. Passes release notes to liter-llm `complete(model=medium)` for feature extraction
4. Output stored in `.evolver/<name>/changelogs/<competitor-id>-<timestamp>.json`:

```json
{
  "repo": "owner/competitor-a",
  "fetched_at": "2026-06-28T00:00:00Z",
  "since_tag": "v2.2.0",
  "to_tag": "v2.3.0",
  "release_count": 3,
  "features_added": [
    "Plugin marketplace launch",
    "Real-time presence indicators"
  ],
  "breaking_changes": [],
  "deprecations": ["Legacy API v1 deprecated"]
}
```

5. Compare extracted features against our parity matrix → identify new gaps
6. Parity matrix update via liter-llm `complete(model=frontier)` — judgment required to assess equivalence

---

## Model routing

```
[MODEL_ROUTING] phase=evolver-changelog-extract class=medium
```
Structured extraction from release notes — bounded NLP, not open-ended synthesis.

```
[MODEL_ROUTING] phase=evolver-competitive-parity class=frontier
```
Parity gap assessment — requires judgment about feature equivalence across different codebases.

```
[MODEL_ROUTING] phase=evolver-competitive-scan class=frontier
```
Full competitive landscape synthesis — cross-domain synthesis, novelty detection.

---

## Competitive scan cadence

| Competitor category | Recommended TTL |
|--------------------|----------------|
| `direct` | 1440 min (daily) — scan on major release |
| `adjacent` | 10080 min (weekly) |
| `aspirational` | 43200 min (monthly) |

For high-velocity competitors (> 1 release/week), subscribe to their GitHub releases via a `changelog` feedback source in `loop.json`.

---

## Example loop.json configuration

```json
{
  "name": "my-product-competitive-loop",
  "evolution_name": "my-product",
  "perspective": "competitive",
  "goal": {
    "description": "Achieve feature parity with direct competitors in the plugin ecosystem domain",
    "measurable_criteria": [
      "parity-matrix.json: 'plugin-marketplace' our_status != 'missing'",
      "parity-matrix.json: all 'high' priority features our_status == 'has' OR 'better'"
    ]
  },
  "feedback_sources": [
    {
      "type": "competitor-scan",
      "competitor_ids": ["competitor-a", "competitor-b"],
      "registry_path": ".evolver/my-product/competitor-registry.json",
      "interpret": "New features in direct competitors require parity evaluation",
      "staleness_ttl_minutes": 1440
    },
    {
      "type": "changelog",
      "repo": "owner/competitor-a",
      "since_tag": "v2.3.0",
      "format": "github-releases",
      "interpret": "New releases expose feature gaps we need to close",
      "staleness_ttl_minutes": 720
    }
  ],
  "termination": {
    "max_ticks": 10,
    "goal_satisfied": true,
    "max_no_progress_ticks": 3
  }
}
```
