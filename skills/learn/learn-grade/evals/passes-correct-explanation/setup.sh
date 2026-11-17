#!/usr/bin/env bash
set -eu
mkdir -p corpus
cat > corpus/binary-search.json <<'JSON'
{ "corpus_id": "cs-fundamentals-01", "subject": "Binary search", "target_level": "novice", "schema_version": "1.1.0",
  "built_at": "2026-11-01T00:00:00Z", "kb_source": "local:corpus", "privacy_mode": true,
  "sources": [
    { "source_ref": "notes/binary-search.md#core", "source_type": "mcp_filesystem", "confidence": 0.9, "is_misconception": false,
      "content_summary": "Binary search finds a target in a sorted array by repeatedly halving the search interval.",
      "key_points": [
        "The input array must already be sorted.",
        "Each step compares the target with the middle element and discards the half that cannot contain it.",
        "Worst-case time is O(log n) comparisons.",
        "If the interval becomes empty the target is absent."
      ], "misconceptions": [] },
    { "source_ref": "notes/binary-search.md#pitfalls", "source_type": "known_misconception", "confidence": 0.9, "is_misconception": true,
      "content_summary": "Common misconception about input order.",
      "key_points": [],
      "misconceptions": ["Binary search works on any array, sorted or not, because it checks the middle element first."] }
  ] }
JSON
