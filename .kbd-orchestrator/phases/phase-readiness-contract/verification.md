# VERIFICATION: phase-readiness-contract

Single boundary run, post-implementation only (operator rule 9). Entry: `bash shared/scripts/tests/test-kbd-readiness-gate.sh` from repo root. Real collaborators: `prometheus` CLI 1.12.2 (canonical runtime journal), `jq`, `sha256sum`/`shasum`, bash ≥4, filesystem. Fixture isolation: scratch trees under `$TMPDIR/kbd-rc-fixtures.*`; no live ports/services; negative controls never write real canonical state.

Evidence per scenario: exact command + argv, exit code, observed stdout/stderr excerpt, fixture identity (scratch path + content digests), limitations. Status vocabulary: PASS / FAIL(product) / BLOCKED(verification) / NOT_EXERCISED.

| Scenario | Production entry | Observable oracle | Negative control | Evidence |
|---|---|---|---|---|
| RC-01 ADOPT-READY | `shared/scripts/kbd-contract-adopt.sh .kbd-orchestrator/phases/phase-readiness-contract/adopted-contract.json` then `shared/scripts/kbd-phase-readiness-check.sh --phase phase-readiness-contract` | decision recorded (audit shows id + digest); checker stdout `READY`, exit 0 | checker BEFORE adoption prints `BLOCKED` naming `adoption`, exit 2 | command argv, exit codes, audit excerpt, checker stdout |
| RC-02 BLOCKED-MISSING | checker against scratch fixture phase dir lacking adoption | `BLOCKED` + minimal missing list contains `adoption`; exit 2 | same fixture + adoption → adoption not in missing list | argv, exit, stdout |
| RC-03 REJECT-STALE | checker against fixture with tampered contractDigest / old planRevision | rejection names `contractDigest`/`planRevision`; no files written to real phase dir | untouched fixture passes digest check | argv, exit, stdout, `git status` of real phase dir unchanged |
| RC-04 REJECT-IDENTITY | checker against fixture with wrong phaseId/runId | rejection names `identity` | correct-identity fixture passes identity check | argv, exit, stdout |
| RC-05 REJECT-MALFORMED | `kbd_stage_gate execute` against fixture with empty/garbage plan handoff | exit ≠0, stderr remediation, no handoff/evidence file created | well-formed handoff passes gate | argv, exit, stderr, fixture dir listing |
| RC-06 AMEND | second `kbd-contract-adopt.sh` run with amended contract (`--supersedes`) | checker accepts new digest, rejects old; `prometheus kbd audit` lists both decisions | old digest rejected post-amendment | argv, exit, audit excerpt |
| RC-07 SKILL-GATE | `npm run validate:strict skills/process/kbd-process-orchestrator` | exit 0 after skill edits | broken-frontmatter copy fails validation (dry-run in scratch, never committed) | argv, exit, output tail |

Post-run acceptance rule (operator oracle 5): N scenarios executed ≠ PASS. After phase completion the matrix is re-run through the real production entry; each scenario's attempted/observed/evidence is recorded; the summary is source-bound PASS/FAIL/BLOCKED per scenario. Reflect may only recommend; it never auto-activates follow-ups.

Known limitation at verification time (recorded, not fabricated): token telemetry UNKNOWN (no provider metering) — budget enforcement is time/tool-call fallback only; machine-wide Cargo occupation is irrelevant to this phase (no Rust artifacts) and is NOT_EXERCISED, not a product failure.
