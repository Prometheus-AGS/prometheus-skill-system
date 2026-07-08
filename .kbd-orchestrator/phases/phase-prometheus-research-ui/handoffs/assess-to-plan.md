# Handoff: assess → plan

_Written: 2026-07-08 by kbd-assess_

## Summary

5 gaps found across 5 goals. Key issue: the deep-research UI is a pure simulation (no real
HTTP calls to :7891), SKILL.md has zero binary references, CI has no prometheus-research
coverage, and surface-bridge UiIntent shape is incompatible with what prometheus-research
currently sends (HIGH risk — fix in G-03 first or alongside G-02). No open questions; all
change boundaries are clear. Suggested change order: G-04 CI first (small, no deps), then
G-01 SKILL.md (no deps), then G-03 protocol fix (prerequisite for G-02 Tier 2 path), then
G-02 UI (largest), then G-05 smoke test (depends on G-02 being stable). Recommended next
phase: /kbd-plan phase-prometheus-research-ui.
