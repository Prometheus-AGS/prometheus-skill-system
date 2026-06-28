# Tasks — change-learn-014

- [ ] Write `skills/learn/learn-certify/SKILL.md` with `--checkpoint` and `--final` modes, prerequisite gate spec, and credential output schema
- [ ] Implement prerequisite gates for `--final` mode: N feynman-artifacts on record, M practice-result files, capstone problem passed via learn-grade
- [ ] Emit OB 3.0 / W3C VC as self-issued JSON-LD signed with did-plc; document key generation and storage convention
- [ ] Implement integrity guardrails: flag credentials where mastery trajectory shows a step-change inconsistent with session history
- [ ] Implement `--issuer` parameter for forwarding to a 1EdTech-compatible endpoint (documented interface, not hardcoded URL)
