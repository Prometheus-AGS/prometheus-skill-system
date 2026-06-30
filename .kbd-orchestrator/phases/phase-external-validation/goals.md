# Goals — phase-external-validation

**Context:** phase-credibility-90 closed at 92% production readiness (sycophancy-corrected, gate score 0.0 strict). The remaining 8% is the P5 structural gap: no external users, no independent validation of learning outcomes under real drift patterns, no third-party endorsement of the anti-sycophancy architectural choices. This gap cannot be closed by writing more code.

This phase transitions the project from internal remediation into external validation mode.

## Goals

- [ ] **G1 — First external user onboarding:** At least one external user (not the maintainer) deploys the prometheus-skill-pack, runs `bash scripts/install-skills-flat.sh`, and exercises the Feynman learning loop (`/learn-goal` → `/feynman-loop` → `/learn-grade`) end-to-end with a real learning topic. Document outcome.

- [ ] **G2 — Self-improving loop validation:** The same or different external user runs `forge enrich` on a real task, accepts or modifies the enriched context, and `forge reflect` is called to record the iteration. Verify that the drift report in `.forge/memory/drift/` actually influences subsequent enrichment resolution (stale skills deprioritized in the next `forge enrich` run).

- [ ] **G3 — sovereign-sync P2P validation:** Two distinct machines (not both on the same host) sync a learner-model domain via sovereign-sync P2P transport. Verify CRDT merge produces consistent state on both nodes.

- [ ] **G4 — Independent anti-sycophancy validation:** A person or automated harness other than the maintainer runs the sycophancy-correction skill against a reflection or grade produced by the system and confirms the gate fires correctly on a known-sycophantic input and passes on a known-honest input.

- [ ] **G5 — Publish external validation evidence:** Capture the outcomes of G1–G4 in a public-facing artifact (GitHub issue, discussion, or a follow-up production readiness report) that can be independently verified. This is the evidence base for a claim above 92%.

## Definition of Done

G1 and G2 are the minimum bar — they validate the two systems the re-audit identified as undemonstrated (learning loop and self-improving loop). G3–G5 raise the ceiling further. All five goals MET = justification for a ≥95% claim with external evidence.

## What this phase does NOT require

- More code changes to forge-rs (unless external user testing surfaces a blocking bug)
- More test additions (44 tests are sufficient for internal validation)
- Another internal self-assessment (the existing 92% claim is accurate for what code can attest)
