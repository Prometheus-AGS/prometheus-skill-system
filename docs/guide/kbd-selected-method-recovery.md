# Selected method, phase authority and recovery

The existing signed KBD runtime and adopted contract own phase authority. The
phase-local `method-context.md` and JSON companion are derived recovery indexes;
they do not authorize a test, model, shell command, publication or successor.

After an operator selects a method, record a normal canonical decision whose
summary begins with `KBD-METHOD ` followed by JSON. Required selection fields are
`phaseId`, current `contractDigest` and `method`; `retiredMethods` retains the
explicitly retired method names and `references` lists project-relative paths
with exact SHA-256 hashes for calibration or other acceptance-bearing documents.
Do not invent a retirement or method choice from a filename or conditional skill.
Supersede an old selection using the existing decision `--supersedes` mechanism.
Do not alter a bound contract to avoid its existing signed amendment rules.

An example summary shape (values come from the current authorized selection):

```json
{
  "phaseId": "selected-phase",
  "contractDigest": "<current bound SHA-256>",
  "method": "VLBV",
  "retiredMethods": [],
  "references": [
    { "path": ".kbd-orchestrator/phases/selected-phase/calibration.md", "sha256": "<exact file SHA-256>" }
  ]
}
```

Use the packaged orchestrator helper from the consumer project:

```sh
node "$KBD_ORCHESTRATOR_ROOT/shared/method-context.mjs" \
  --project "$PWD" --phase selected-phase --write
```

`--json` prints the structured view; `--require-resolved` returns exit 2 for a
missing, changed or ambiguous contract/method reference. It is not an execution
readiness or acceptance gate. Without `--write`, discovery is read-only with
respect to phase artifacts. It does not initialize a new project, install tools,
execute generated-code checks, write canonical decisions or retry an effect.

The index records canonical project/run/phase/revision, current contract binding,
method decision/supersession, included value and exclusions, stage operations,
governing instruction identities, host observations, candidate/evidence freshness,
budget and original-consumption references, and open receipt/blocker obligations.
Missing old consumption stays UNKNOWN. Recovery does not reset it or imply that
the last attempt had no effect. Reconcile uncertain effects before retrying.

Lifecycle hooks refresh this derived view at named phase/child/stage edges.
Init/Assess guidance discovers it; Spec/Plan preserve calibration and contract
identity; Execute dispatch, child/harness handoff and pause/resume read it before
dependent action. Bootstrap compaction recovery emits compact pointers and a
labelled snapshot. Team role exports carry the same instructions through existing
role inputs/prompt fields; task handoffs add index/contract/reference hashes to
existing source-file provenance. Receiving inspection compares current identities;
a digest of an old pointer is not proof of latest authorized supersession.

For selected VLBV, define acceptance/scenarios before generation, finish the
complete agreed value-phase production, then separately authorize the actual
phase-end verification. Executable test authoring follows the governing host
policy. A task, hour, commit, reviewer request, compiler problem or renamed slice
does not create that boundary. A host compiler exception is surfaced as a
potential conflict, not permission to run early checks. Never silently rewrite
operator-owned policy to make the conflict disappear.

Control is instruction-only on a host without a named interception mechanism.
File hashes observe content identity; they do not prove model obedience. Existing
signed runtime commands enforce only their named mutations. Unsupported local
harness/session observation remains unqualified. Publication cadence cannot
certify partial implementation, grant release authority or start new work.

This feature does not replace runners, contract adoption, canonical closure or
trusted tool admission. Source and selected packaged/installed surfaces are
qualified only at the complete parent implementation boundary; version labels,
export text and this guide are not acceptance evidence.
