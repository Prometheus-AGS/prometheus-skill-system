# Decision log — issue-195-closeout

## 2026-10-27 — Analyze recommendations

Provenance: in-repo inspection at main 6bb66d5c and Context7 (opencode.ai docs). Recommendations for Spec, not contract adoption.

- A: reuse the signed decision surface with a `KBD-POLICY-ACK` payload bound to the policy file SHA-256; build only the status semantics in method-context.mjs (`CONFLICT`, exit 2 under --require-resolved). Marker file rejected.
- B: shrink the 11 sections to an identical short core plus link to references/method-context.md; enforce with an equality check. Bare link and generator include rejected.
- C: implement VER-001/VER-003 bodies in the existing session driver; add CON-001/CON-002/REF-001.
- D/E: OpenCode only with scratch XDG roots, OPENCODE_DISABLE_CLAUDE_CODE=1 and --standalone after an isolation preflight; otherwise UNSUPPORTED.
- G: per-run judge override labelled unverified-producer-unknown; models.toml fix is owner-owned.

Next-stage recommendation: READY to spec. Owner inputs (risk, budgets, ack mechanism, logins, review route) remain open for Plan.

## 2026-10-27 — Spec authoring

Created three native-kbd changes (generator: gen_i195_specs.mjs): i195-01-policy-conflict-ack (2 tasks), i195-02-single-method-reference (2 tasks), i195-03-delivery-qualification (4 tasks). Twelve stable scenarios: CON-001..003, POL-001..003 (reruns), REF-001, PKG-001 (rerun), VER-001, VER-003, SRF-001, SRF-002. Draft adopted-contract.json validates against the schema; state DRAFT_SPEC, not adopted. Budgets are the prior-phase defaults marked PROPOSED.

Adversarial review (glm-5.3 per-run override, unverified-producer-unknown): PASS, 0 CRITICAL / 7 WARNING / 1 SUGGESTION; anti-theater gate PASS. All addressed in the generator: changeId bound into identity rows (was `undefined`), C-01 regeneration owner named (i195-02/2), dist/** tracking stated, review route named, goal B deviation recorded, SRF sensitivity controls added, per-change review covered by i195-03/3 receipt, #195 posting decision recorded in closure-report.md. Not re-vetted (no CRITICAL).

## 2026-10-30 — owner approvals (AskUserQuestion + approved plan)
- Full closeout with evidence for #195 and #196 (scratch Claude login accepted as an owner step).
- Goal B accepted in adapted form: one identical inline core in 11 stage skills + single reference file + `check:selected-method` equality check.
- Posting the acceptance-to-evidence mappings on #195 and #196 and closing both issues is authorized.
- Plan re-vet: round 1 BLOCK (C-01 twice-run missing, 1 CRITICAL) → plan revised → round 2 PASS by glm-5.3 (`review/plan/findings.round2.json`, label unverified-producer-unknown), 4 warnings folded into plan.md.
