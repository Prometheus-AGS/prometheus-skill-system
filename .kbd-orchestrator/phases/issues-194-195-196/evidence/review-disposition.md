# Independent cumulative review: disposition

Four fresh read-only reviewers (general code, Rust, TypeScript/JS/shell, security) examined the uncommitted candidate.
**Independence limit:** the producer is Claude Sonnet 5.5 and the reviewers were Claude-family subagents of the same session;
cross-model independence is NOT established. No reviewer executed code; most findings were verified by reading only.

Verdicts: Rust no CRITICAL / 2 HIGH; security no CRITICAL / 1 HIGH (trust-model); code review APPROVE with warnings;
TypeScript BLOCK (7 HIGH, mostly acceptance-oracle weakness plus one real product bug).

## Fixed (repair attempts 1 and 2; both budgeted attempts are now consumed)
| Finding | Change |
|---|---|
| Rust HIGH: repair event re-checked `expected_revision` on fold replay | `prepare_replay` skips the admission revision check and compares live sources to the recorded originals |
| Rust MED: digest recomputed from live struct on replay | same: struct equality on replay, digests only at admission |
| CLI HIGH/MED: titles containing `/` or `:` no longer resolved | removed the blocking branch; whole-subject id/title match restored |
| TS HIGH: `method-context.mjs` entry guard fails through a symlinked skill dir (installed hooks did nothing) | real-path comparison; PKG-001 now asserts `--write` through a symlink |
| `method-context` policy regex missed this repository's own compiler-check wording | broadened; **status is still not downgraded by a policy conflict** (deferred) |
| `method-context` hard-coded repo-specific stage text | generalized |
| Volatile `generatedAt`/`sourceRevision` made provenance hashes always "changed" | index rewritten only when semantic content changes |
| Security MED: newline/`..` in reference path listed files outside the project in the handoff | reference paths validated in the helper and again in `add_read`; unsafe refs become blockers |
| Security MED: symlinked ancestor let `--write` mkdir outside the project | nearest existing ancestor realpath-checked |
| Security LOW: unfenced `selectedMethod` text | method label validated as a short plain label |
| Helper aborted on a symlinked governing file / malformed reference / `KBD-METHOD null` | recorded as unreadable/blocker instead of throwing |
| Rust MED: "no damaged identity" guard was phase-wide; unlisted third key with the target ID | guard is source-specific; unlisted duplicate target ID refused |
| Rust/code review: `resolve_change_key` dead code | removed |
| Code review MED: `validate_legacy` read non-object rows as IDs | only array strings and object rows carry IDs |
| Code review MED (spec): inspection did not report unmatched receipts | `unmatchedObligations` added to `inspect-identity` |
| TS MED: reconcile alias fallthrough swallowed a corrupt backend | falls through only when the artifact is absent |
| TS MED: captured-missing then created reported "unobserved" | reported `changed` |
| TS MED: handoff phase id could be a leaf slug | canonical phase id preferred |
| Found by the stronger oracle: lifecycle hooks received the retired alias and backend ordinal | hooks get the surviving change and registered task key |
| Code review MED: stale per-change BDD guidance in `workflows/templates/kbd-execute.md`; portable skills stated repo policy as fact; dry-run example | rewritten / made conditional |
| Security: authority model overstated in the guide | guide states the command is operator-only by policy, not a human gate |
| Security LOW: scratch worlds could reach a real local memory service; seeder could write a real journal | memory URLs point at a dead port; seeder refuses paths outside `kbd194-*` scratch dirs |
| TS HIGH: driver exit 0 without contract coverage; missing args resolved to cwd; vacuous PASS; weak identity | iterates the contract; required args; zero assertions is NOT_EXERCISED; exceptions are FAIL; identity hashes the dirty tree, entry points and every assertion file |
| TS HIGH/MED: vacuous oracles (signer, changed reference, alias reconcile, blocked/second completion, "exists", command-id content) | each now matches the specific message or structure; dropped precondition recorded; key generation checked |

## Deferred (with reason; none hidden)
- Security HIGH (operator authority is a submitter-chosen label): existing trust model of every operator command; documented, not changed here.
- Schema-"1" unsigned events bypass operator checks: pre-existing for all operator-only events.
- Obligation prefix matching by `/` `:` for IDs containing those characters; `latest_boundary_receipts` not remapped; claims/gate receipts keyed to old names.
- `migrate --check` aborts on the first refused legacy phase (intended refusal behavior).
- `policyConflicts` does not change `status`/`--require-resolved`.
- Selected-method paragraph copied into about 11 stage skills; `hooks.json` em-dash churn and `$PWD` project root; rustfmt/line-length/function-length style of `change_identity.rs` (fmt/clippy not run).
- Tests not added: `begin-task` through a retired alias, ambiguous/cyclic alias refusal, `reconcile-command` repair path under `KBD_RECONCILE_REPAIR=1`, installer `--targets claude` (the Claude payload is installed through the codex target dispatch).
- Reviewer claims not accepted: that `::` is not a phase-ID convention (child phase IDs such as `adversarial-review-for-creation::mcp-2026-07-28-adoption` exist); the TS finding of three non-equivalent phase directory mappings is unreproduced for current layouts.
- Findings 2-4 of the security review (reference path, symlinked ancestor, schema-1) were fixed or left from reading code and were NOT reproduced dynamically.
- The 8 session scenarios remain BLOCKED (no authenticated session in scratch roots).
- Whether the earlier fixture runs wrote lifecycle entities to the user's real memory service before the dead-port env was added is UNVERIFIED.
