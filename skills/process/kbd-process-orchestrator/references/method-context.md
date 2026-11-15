# Selected method and recovery references

VLBV is the verification method for every phase. `.kbd-orchestrator/project.json` carries `verificationMethod` (written as `"VLBV"` by `kbd-init`); when it is absent the pack default is VLBV. No per-phase decision is needed. To **deviate** for one phase, record a signed decision whose summary begins with `KBD-METHOD ` followed by JSON containing `phaseId`, the currently bound `contractDigest`, `method`, `cadence` (when generated code executes under that method), `reason`, `retiredMethods` and `references` (`path` plus exact SHA-256). A deviation without `cadence` and `reason` is `BLOCKED`, and `kbd-phase-close-eval.sh` names the deviation in the closure packet. Record it only from explicit operator selection through `prometheus kbd decision record`; replace it using the existing `--supersedes` lineage. Do not infer a deviation from a filename, old command, memory or available tool.

The governing instructions checked for conflicts are the files the active harness actually loads, taken from that harness's profile in `shared/harnesses/capabilities.json` (`profiles.<harness>.rulesFiles`; pass `--harness <id>` or set `PROMETHEUS_HARNESS`). A first-match host such as Zed can be shadowed by an earlier file (`.rules`, `.cursorrules`, …), and Codex `AGENTS.override.md` replaces `AGENTS.md`; both are reported under `effectiveRules.shadowing`. Policy is read from a marked block — `<!-- kbd-policy:begin -->` … `<!-- kbd-policy:end -->` with lines such as `compiler-exception: retired` — and free-text matching is only the fallback for files that predate the block.

`node shared/method-context.mjs --project <root> --phase <id> --write` reads canonical status/disposition and produces `method-context.json`/`method-context.md` beside the phase contract. These are derived readable indexes, never an authority ledger. `--json` prints structured discovery; `--require-resolved` returns exit 2 for `BLOCKED` (missing or ambiguous references) and `CONFLICT` (unacknowledged policy conflict). Status precedence is `BLOCKED > CONFLICT > RESOLVED_REFERENCES`. The helper never executes generated-code verification, installs dependencies, edits the contract, selects a method, retries an effect or changes canonical state.

Init/Assess discover host governing instructions/capabilities and current method. Spec/Plan carry acceptance-bearing calibration references under the contract identity. Execute, child/harness handoff, pause/resume and compaction recover the current index and re-resolve canonical decisions before dependent action. Carry its path through role inputs, selected source/evidence provenance and handoff readInOrder/startPrompt; these fields grant no execution authority.

Define acceptance/scenarios before generation. Finish the complete agreed value-phase production before any generated-code tests, builds, imports, lint, typecheck, hooks or probes; editor/LSP diagnostics read during generation are static reading, not execution. Generated code executes only at B4 (the boundary packet, once) and B5 (an authorized repair) — `references/boundaries.md`. Scenario definition is distinct from executable test authoring. A task/hour/commit/reviewer request, compiler problem or renamed slice never creates a boundary. Surface conflicting host/skill wording; do not silently rewrite operator-owned instructions.

The index retains included value/exclusions, contract/binding, stage operations, retired-method lineage, candidate/evidence freshness, adopted budgets and consumption references, open receipt/blocker obligations and preservation requirements. Reconcile unknown effects before retry; never reset consumption or discard original failures. Resolve latest authorized supersession, not just the hash of an old pointer. Changing acceptance-bearing reference content requires the existing contract amendment authority where applicable.

Describe control as instruction-only, observed, enforced within a named existing boundary, or unsupported. File hashing observes identity, not model obedience. Only actual runtime commands enforce their named mutations. This integration provides no generic shell fence, universal tool interception, release authority or automatic successor.

## Policy conflict acknowledgement

Before dependent work, show each conflict's path, current SHA-256 and issue. An explicit operator resolution is recorded through the existing signed decision writer:

```text
prometheus kbd --path <project-root> decision record --command-id <unique-id> --id <decision-id> --summary 'KBD-POLICY-ACK {"phaseId":"<canonical-phase-id>","path":"<policy-path>","sha256":"<current-file-sha256>","issue":"<exact-reported-issue>","note":"<operator resolution and its provenance>"}'
```

Exactly one current, non-superseded acknowledgement must match phase, path, hash and issue, with a nonempty resolution note. Conflicts expose `acknowledged` and `ackDecisionId`. Changed files, retired acknowledgements and ambiguous acknowledgements do not clear the conflict. Replace an acknowledgement using `--supersedes`; retain its history. Acknowledgement records a resolution, never permission to violate governing host policy. In particular, the complete value-phase boundary still applies.

This uses the same instruction-level trust model as a `KBD-METHOD` deviation: canonical decisions carry signed journal provenance, but their summary does not establish a new actor authorization boundary. Do not describe this as universal host enforcement.

## Canonical inline recovery core

The following marked block is the single source for the 11 stage sections. The equality check compares section bytes, including trailing blank lines.

<!-- selected-method-core:start -->
## Selected-method recovery and authority

Before dependent work, run `node shared/method-context.mjs --project <project-root> --phase <canonical-phase-id> --write` from the resolved orchestrator root (or read the phase-local `method-context.md` when Node is unavailable) and act on its `status`:

- `RESOLVED_REFERENCES` — continue with the recovered method, contract and stage authority; preserve the same references in role inputs, dispatch and handoff read order.
- `BLOCKED` — the contract, its binding, a calibration reference, a supersession or a deviation's cadence/reason is missing or changed; resolve it explicitly. Prior commands, memory and tool inventories are not fresh authority.
- `CONFLICT` — a governing rule file the active harness loads grants an exception VLBV forbids (for example a compiler check during generation). Stop before dependent work and show the conflict; continue only after the operator records a `KBD-POLICY-ACK` decision for that file's current hash. Never rewrite operator-owned policy, and never treat an exception as authority for tests, builds, dry-runs or generated-code verification.

VLBV is the method for every phase (project default `verificationMethod: "VLBV"`); a deviation exists only as a signed `KBD-METHOD` decision naming the method, its execution cadence and its reason, and it is disclosed in the closure packet. Define acceptance before generation; generated code executes only at the value-phase boundary (B4) and, after an authorized repair, at B5 — see `references/boundaries.md`. Tasks, hours, commits, reviewer requests and compiler problems create no boundary. Preserve original evidence, consumed budgets and unresolved effects across resume and supersession; reconcile an uncertain effect before any retry. Full rules: `references/method-context.md`.

<!-- selected-method-core:end -->
