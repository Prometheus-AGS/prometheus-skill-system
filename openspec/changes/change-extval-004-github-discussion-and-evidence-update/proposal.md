# change-extval-004-github-discussion-and-evidence-update

**Phase:** phase-external-validation  
**Type:** community + documentation  
**Status:** proposed  
**Priority:** P1 (enables G5; depends on changes 1–3)

## Summary

Create a GitHub Discussion calling for first-user feedback and add a
"Phase: external-validation" section to `docs/production-readiness-report.md`
with placeholder rows for G1–G4 evidence.

## Dependencies

Must run after change-extval-001, change-extval-002, and change-extval-003 are
complete, so the discussion can link to all supporting artifacts.

## Deliverables

- GitHub Discussion on `Prometheus-AGS/prometheus-skill-system` (via `gh` CLI)
- Updated `docs/production-readiness-report.md` with evidence placeholder section

## Tasks

- [ ] Draft discussion title and body (links to QUICK_START, SOVEREIGN_SYNC_TESTING, sycophancy corpus)
- [ ] Create discussion via `gh api` or `gh discussion create` with `help wanted` label
- [ ] Add `## External Validation Phase` section to production-readiness-report.md
- [ ] Add placeholder table for G1–G4 outcomes (Status: PENDING for each)
- [ ] Add link to the GitHub Discussion from the report
- [ ] Commit updated report to main
