---
id: change-evolver-001
title: "Schema: pmpo-evolver.schema.json + evolution-state extensions"
phase: pmpo-evolver
gaps: [G-02, G-12]
priority: HIGH — foundation; all other changes reference these fields
goals: G2, G5
agent: claude-code
status: done
scope:
  - skills/process/pmpo-evolver/references/schemas/pmpo-evolver.schema.json
  - skills/process/iterative-evolver/references/schemas/evolution-state.schema.json
---

# change-evolver-001 — Schema: pmpo-evolver.schema.json + evolution-state extensions

## Problem

No schema exists for the `pmpo-evolver` strategy router state. The existing `evolution-state.schema.json` lacks fields for `learning_signals[]` (Karpathy perspective) and `perspective` (routing selection). All other changes in this phase depend on these schemas being defined first.

## Solution

Create `skills/process/pmpo-evolver/references/schemas/pmpo-evolver.schema.json` defining the full strategy-router state object. Extend `skills/process/iterative-evolver/references/schemas/evolution-state.schema.json` with additive-only fields: `learning_signals[]` and `perspective`.

## New file: pmpo-evolver.schema.json

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "pmpo-evolver.schema.json",
  "title": "PmpoEvolverState",
  "description": "Strategy router state for the pmpo-evolver skill",
  "type": "object",
  "required": ["evolution_name", "perspective"],
  "properties": {
    "evolution_name": { "type": "string" },
    "perspective": {
      "type": "string",
      "enum": ["competitive", "trend", "unique-product", "idea-validation", "self-learning", "combined", "auto"]
    },
    "perspective_cursor": {
      "type": "object",
      "properties": {
        "current": { "type": "string" },
        "completed": { "type": "array", "items": { "type": "string" } },
        "pending": { "type": "array", "items": { "type": "string" } }
      }
    },
    "competitor_tracking": {
      "type": "object",
      "properties": {
        "registry_path": { "type": "string" },
        "last_scanned": { "type": "string", "format": "date-time" },
        "parity_matrix_path": { "type": "string" }
      }
    },
    "learning_signals": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["source_type", "signal", "severity"],
        "properties": {
          "id": { "type": "string" },
          "source_type": {
            "type": "string",
            "enum": ["gh-issues", "commit-history", "sentiment-feed", "usage-trace", "telemetry-url", "competitor-scan", "changelog", "research-query"]
          },
          "source_ref": { "type": "string" },
          "collected_at": { "type": "string", "format": "date-time" },
          "signal": { "type": "string" },
          "severity": { "type": "string", "enum": ["high", "medium", "low"] },
          "count": { "type": "integer" },
          "examples": { "type": "array", "items": { "type": "string" } },
          "model_used": { "type": "string" }
        }
      }
    },
    "idea_origin": {
      "type": "object",
      "properties": {
        "type": {
          "type": "string",
          "enum": ["competitive", "trend", "operator", "self-learning", "continuation"]
        },
        "rationale": { "type": "string" },
        "first_seen": { "type": "string", "format": "date-time" }
      }
    },
    "evolver_lessons": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["lesson", "confidence", "category"],
        "properties": {
          "lesson": { "type": "string" },
          "origin_cycle": { "type": "integer" },
          "confidence": { "type": "string", "enum": ["high", "medium", "low"] },
          "category": { "type": "string", "enum": ["direction", "threat", "opportunity", "falsified-hypothesis"] }
        }
      }
    },
    "model_routing": {
      "type": "object",
      "properties": {
        "policy": { "type": "string", "enum": ["liter-llm", "harness-native", "frontier-all"] },
        "class_map": { "type": "object", "additionalProperties": { "type": "string" } }
      }
    }
  }
}
```

## Modified file: evolution-state.schema.json (additive fields only)

Add to `properties`:
```json
"learning_signals": {
  "$ref": "pmpo-evolver.schema.json#/properties/learning_signals"
},
"perspective": {
  "type": "string",
  "description": "Which pmpo-evolver perspective drove this evolution cycle"
}
```

These additions are backward-compatible (not required fields).

## Acceptance criteria

- `pmpo-evolver.schema.json` exists and validates via `python3 -m json.tool`
- `evolution-state.schema.json` validates via `python3 -m json.tool` after changes
- The `learning_signals[].source_type` enum matches the types defined in change-evolver-002
- The `perspective` enum matches the modes defined in change-evolver-003's SKILL.md

## Tasks

- [x] 1. `pmpo-evolver.schema.json` exists and validates via `python3 -m json.tool`
- [x] 2. `evolution-state.schema.json` validates via `python3 -m json.tool` after changes
- [x] 3. The `learning_signals[].source_type` enum matches the types defined in change-evolver-002
- [x] 4. The `perspective` enum matches the modes defined in change-evolver-003's SKILL.md
