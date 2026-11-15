# VLBV single-method + OpenSpec 2.0 — owner decisions (2026-11-15)

Recorded from the owner's answers in the planning session for release 1.15.0.
Source proposal: *Prometheus Skill Pack: making VLBV the single verification method*
(2026-11-15) and its change register workbook.

| ID | Decision |
|---|---|
| D1 | Retire the AGENTS.md/CLAUDE.md compiler exception. No generated-code execution before B4; the second gate run happens only as B5 reverification. This explicitly amends the "Immutable Implementation-First and Integration-Only Policy" section (owner instruction naming that policy, via plan approval). |
| D2 | VLBV is the project default (`verificationMethod: "VLBV"`); deviation requires a signed `KBD-METHOD` decision and is disclosed in the closure packet. |
| D3 | `.agents/skills` is the only installed copy; shims only for Claude Code and Qwen Code (Codex served by `~/.agents/skills` after the gate passed — evidence `docs/decisions/harness-evidence/codex-skill-roots-2026-11-15.txt`). |
| D4 | XiaomiMiMo/MiMo-Code is in scope; `@xiaomi-mimo/cli` is out. |
| D5 | Editor/LSP diagnostics are allowed as static reading. |
| D6 | Optional stochastic profile (samples, grader, stopping rule declared before results). |
| D7 | Scenario ID pattern widened to `^[A-Z][A-Z0-9]*(-[A-Z0-9]+)*-[0-9]+$`. |
| CR-01 | Pin exactly OpenSpec 2.0.0 and land the 2.0 port in this release; never cross a major unpinned (`migrationRequired`). |
| Delivery | R0–R3 in 1.15.0; R4 (CR-31 protocol evals, CR-33 passive scan) is a follow-up. |
| CR-22 scope | Implement the canonical-root installer change in 1.15.0 (owner chose "do it now, carefully"). |
| Additions | Apple on-device Foundation Models auto-initialized at install; prometheus-context-bootstrap aligned with VLBV. |

## Canonical KBD phase record

The proposal asks that this work be recorded as a KBD value phase with a v1
adopted contract. At execution time KBD's canonical state had a different phase
active (`issue-195-closeout › execute`), so no canonical phase was created or
activated without the owner's explicit go-ahead. This file and the plan-review
receipts under `review/` are the record until then.
