# Selected method and recovery references

The existing signed KBD decision surface selects a phase method. A decision summary begins with `KBD-METHOD ` followed by JSON containing `phaseId`, the currently bound `contractDigest`, `method`, `retiredMethods` and `references` (`path` plus exact SHA-256). Record it only from explicit operator selection through `prometheus kbd decision record`; replace it using the existing `--supersedes` lineage. Do not infer method selection from a filename, old command, memory, available tool or conditional VLBV wording.

`node shared/method-context.mjs --project <root> --phase <id> --write` reads canonical status/disposition and produces `method-context.json`/`method-context.md` beside the phase contract. These are derived readable indexes, never an authority ledger. `--json` prints structured discovery; `--require-resolved` returns exit 2 for `BLOCKED` (missing or ambiguous references) and `CONFLICT` (unacknowledged policy conflict). Status precedence is `BLOCKED > CONFLICT > RESOLVED_REFERENCES`. The helper never executes generated-code verification, installs dependencies, edits the contract, selects a method, retries an effect or changes canonical state.

Init/Assess discover host governing instructions/capabilities and current method. Spec/Plan carry acceptance-bearing calibration references under the contract identity. Execute, child/harness handoff, pause/resume and compaction recover the current index and re-resolve canonical decisions before dependent action. Carry its path through role inputs, selected source/evidence provenance and handoff readInOrder/startPrompt; these fields grant no execution authority.

For selected VLBV, define acceptance/scenarios before generation. Finish the complete agreed value-phase production before generated-code tests, builds, imports, lint, typecheck, hooks or probes used as verification. Scenario definition is distinct from executable test authoring; governing host policy decides the latter. A task/hour/commit/reviewer request, compiler problem or renamed slice never creates a boundary. Surface conflicting host/skill wording; do not silently rewrite operator-owned instructions. A host compiler exception does not imply VLBV permission during generation.

The index retains included value/exclusions, contract/binding, stage operations, retired-method lineage, candidate/evidence freshness, adopted budgets and consumption references, open receipt/blocker obligations and preservation requirements. Reconcile unknown effects before retry; never reset consumption or discard original failures. Resolve latest authorized supersession, not just the hash of an old pointer. Changing acceptance-bearing reference content requires the existing contract amendment authority where applicable.

Describe control as instruction-only, observed, enforced within a named existing boundary, or unsupported. File hashing observes identity, not model obedience. Only actual runtime commands enforce their named mutations. This integration provides no generic shell fence, universal tool interception, release authority or automatic successor.

## Policy conflict acknowledgement

Before dependent work, show each conflict's path, current SHA-256 and issue. An explicit operator resolution is recorded through the existing signed decision writer:

```text
prometheus kbd --path <project-root> decision record --command-id <unique-id> --id <decision-id> --summary 'KBD-POLICY-ACK {"phaseId":"<canonical-phase-id>","path":"<policy-path>","sha256":"<current-file-sha256>","issue":"<exact-reported-issue>","note":"<operator resolution and its provenance>"}'
```

Exactly one current, non-superseded acknowledgement must match phase, path, hash and issue, with a nonempty resolution note. Conflicts expose `acknowledged` and `ackDecisionId`. Changed files, retired acknowledgements and ambiguous acknowledgements do not clear the conflict. Replace an acknowledgement using `--supersedes`; retain its history. Acknowledgement records a resolution, never permission to violate governing host policy. In particular, the complete value-phase boundary still applies to selected VLBV.

This uses the same instruction-level trust model as `KBD-METHOD`: canonical decisions carry signed journal provenance, but their summary does not establish a new actor authorization boundary. Do not describe this as universal host enforcement.

## Canonical inline recovery core

The following marked block is the single source for the 11 stage sections. The equality check compares section bytes, including trailing blank lines.

<!-- selected-method-core:start -->
## Selected-method recovery and authority

Before dependent work, run `node shared/method-context.mjs --project <project-root> --phase <canonical-phase-id> --write` from the resolved orchestrator root (or read the phase-local `method-context.md` when Node is unavailable) and act on its `status`:

- `RESOLVED_REFERENCES` — continue with the recovered method, contract and stage authority; preserve the same references in role inputs, dispatch and handoff read order.
- `BLOCKED` — a method decision, contract, calibration or supersession is missing or changed; resolve it explicitly. Prior commands, memory and tool inventories are not fresh authority.
- `CONFLICT` — governing host policy conflicts with the selected method. Stop before dependent work and show the conflict; continue only after the operator records a `KBD-POLICY-ACK` decision for that file's current hash. Never rewrite operator-owned policy. When host policy grants an exception — for example one narrow, targeted compiler check after a blocking compile error — name it as host policy, state that it conflicts with the selected method, and say it does not authorize tests, dry-runs or generated-code verification.

For selected VLBV, define acceptance before generation and finish the whole agreed value phase before any generated-code verification; tasks, hours, commits, reviewer requests and compiler problems create no boundary. Preserve original evidence, consumed budgets and unresolved effects across resume and supersession; reconcile an uncertain effect before any retry. Full rules: `references/method-context.md`.

<!-- selected-method-core:end -->
