# change-evolver-002 — Feedback source taxonomy extension (loop-definition schema)

**Phase:** pmpo-evolver
**Priority:** HIGH — Karpathy perspective's most novel gap; enables self-learning loops
**Gaps:** G-09
**Goals:** G5
**Model class:** small (schema extension + docs authoring)

## Problem

The `loop-definition.schema.json` `feedback_sources[].type` enum only covers a narrow set of source types (`command`, `gh-query`, `file`, `url`). The Karpathy self-learning perspective requires: `gh-issues` (with label/state filtering + sentiment), `commit-history` (git log → LLM classification), `sentiment-feed` (external RSS/JSON), `telemetry-url` (JSON API + jsonpath extraction), `competitor-scan` (reads competitor-registry.json), and `changelog` (GitHub releases or CHANGELOG.md).

There is also no `staleness_ttl_minutes` field to prevent redundant re-fetching on every tick.

## Solution

Extend `loop-definition.schema.json` with new source type entries (additive, backward-compatible). Create `feedback-sources.md` as the authoritative reference for how to configure and interpret each source type.

## Modified schema: loop-definition.schema.json

Extend the `feedback_sources` items `oneOf` array with the following new entries. The existing entries remain unchanged.

New base property added to all items (via shared `$defs` or direct addition):
```json
"staleness_ttl_minutes": {
  "type": "integer",
  "description": "Do not re-fetch this source if last_collected_at is within this many minutes. 0 = always refresh."
}
```

New `oneOf` entries:

### gh-issues
```json
{
  "type": "object",
  "required": ["type", "repo"],
  "properties": {
    "type": { "const": "gh-issues" },
    "repo": { "type": "string", "description": "owner/repo" },
    "labels": { "type": "array", "items": { "type": "string" } },
    "state": { "type": "string", "enum": ["open", "closed", "all"], "default": "open" },
    "since": { "type": "string", "description": "ISO8601 date — only issues updated after this date" },
    "interpret": { "type": "string", "description": "What a high count means for product direction" },
    "staleness_ttl_minutes": { "type": "integer", "default": 30 }
  }
}
```

### commit-history
```json
{
  "type": "object",
  "required": ["type", "repo_path"],
  "properties": {
    "type": { "const": "commit-history" },
    "repo_path": { "type": "string" },
    "since": { "type": "string", "description": "ISO8601 date" },
    "classify_by": { "type": "string", "enum": ["conventional-commits", "llm"], "default": "conventional-commits" },
    "interpret": { "type": "string" },
    "staleness_ttl_minutes": { "type": "integer", "default": 60 }
  }
}
```

### sentiment-feed
```json
{
  "type": "object",
  "required": ["type", "url"],
  "properties": {
    "type": { "const": "sentiment-feed" },
    "url": { "type": "string" },
    "format": { "type": "string", "enum": ["rss", "json", "csv"] },
    "sentiment_field": { "type": "string", "description": "JSON field containing the text to classify" },
    "interpret": { "type": "string" },
    "staleness_ttl_minutes": { "type": "integer", "default": 60 }
  }
}
```

### telemetry-url
```json
{
  "type": "object",
  "required": ["type", "url"],
  "properties": {
    "type": { "const": "telemetry-url" },
    "url": { "type": "string" },
    "headers": { "type": "object", "additionalProperties": { "type": "string" } },
    "jsonpath": { "type": "string", "description": "JSONPath expression to extract the numeric or string value" },
    "direction": { "type": "string", "enum": ["higher-is-better", "lower-is-better", "exact-match"] },
    "baseline": { "type": "number" },
    "interpret": { "type": "string" },
    "staleness_ttl_minutes": { "type": "integer", "default": 15 }
  }
}
```

### competitor-scan
```json
{
  "type": "object",
  "required": ["type"],
  "properties": {
    "type": { "const": "competitor-scan" },
    "competitor_ids": { "type": "array", "items": { "type": "string" } },
    "registry_path": { "type": "string", "description": "Path to competitor-registry.json (default: .evolver/{name}/competitor-registry.json)" },
    "interpret": { "type": "string" },
    "staleness_ttl_minutes": { "type": "integer", "default": 1440 }
  }
}
```

### changelog
```json
{
  "type": "object",
  "required": ["type", "repo"],
  "properties": {
    "type": { "const": "changelog" },
    "repo": { "type": "string", "description": "owner/repo for GitHub releases API" },
    "since_tag": { "type": "string", "description": "Last known release tag; fetch everything after" },
    "format": { "type": "string", "enum": ["github-releases", "file"], "default": "github-releases" },
    "file_path": { "type": "string", "description": "Local CHANGELOG.md path when format=file" },
    "interpret": { "type": "string" },
    "staleness_ttl_minutes": { "type": "integer", "default": 720 }
  }
}
```

## New file: feedback-sources.md

Documents:
- Each source type with a concrete example `loop.json` snippet
- How to write an effective `interpret` field for each type
- Staleness TTL guidance per source
- How the tick normalizes all outputs into `{signal, severity, count, examples[]}` LearningSignal format
- **Model routing per signal type**: `gh-issues` count-delta → `small`; sentiment classification → `medium`; competitor scan synthesis → `frontier`
- How staleness_ttl_minutes prevents redundant API calls within a single loop session

## Acceptance criteria

- [ ] `loop-definition.schema.json` validates via `python3 -m json.tool` after changes
- [ ] New source types are additive — existing `command`, `gh-query`, `file`, `url` entries unchanged
- [ ] `skills/process/pmpo-evolver/references/feedback-sources.md` exists with one example per new type
- [ ] Staleness TTL defaults documented in the reference
