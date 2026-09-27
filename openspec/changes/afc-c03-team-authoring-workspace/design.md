## Context

See `proposal.md` for motivation and `specs/agent-team-management/spec.md` for observable behavior. This is the full-pack child of initiative `afc-c03-lossless-definitions-and-collaboration-document-profile` (C03.1-C03.3, REC-054, REC-055).

The existing creator has three separate representations:

1. a schema-v1 project `Team` stored at `.agent-team/<team-id>/team.json`;
2. an inline UAR authoring envelope accepted under `package` by `uar-package-*` commands; and
3. canonical Draft 0.1.0-draft.1 documents compiled by `uar-package.mts`.

`guidance.mts` can ask create/revise/deploy intake questions, but the answers do not form a resumable definition graph. `validation.mts` validates the flat schema-v1 team. `uar-package.mts` performs hand-written top-level checks and selected reference checks rather than consuming the complete official schema set. The copied UAR schemas are pinned to Draft 0.1.0-draft.1. The CLI reads JSON from `--input <request.json>` and this compatibility surface must remain usable.

The official `0.1.0-draft.2` files are provider-owned and are not present at planning time. Implementation therefore begins with an immutable UAR source checkpoint; this plan does not guess draft.2 fields.

## Goals / Non-Goals

**Goals:**

- Give the creator one canonical, resumable authoring graph shared by guided workspace and inline requests.
- Confine all workspace mutation to one explicit project/team root with revision checks and recoverable writes.
- Validate exact `0.1.0-draft.2` document shapes and cross-document graph semantics before versioning or package creation.
- Create immutable next versions without altering a prior source snapshot or dependency lock.
- Preserve existing team and Draft 0.1 source through an auditable field-by-field migration.
- Ship identical source, compiled payload, and generated Claude/Codex copies, then exercise the packaged path against one real persistent UAR instance.

**Non-Goals:**

- Do not implement UAR runtime execution, team activation, private RepresentationGrant storage, deployment-binding resolution, or UAR schemas in this repository.
- Do not change BossFang adapters, the mini-pack, sibling repositories, installed user skills, or project native-agent files as part of authoring.
- Do not make schema-v1 `team.json` the canonical `0.1.0-draft.2` document or infer draft.2 semantics from flat role prompts.
- Do not introduce another daemon, remote schema fetch at execution time, or runtime npm dependency.

## Decisions

### 1. One neutral authoring graph feeds both input modes

Add a typed neutral `AuthoringGraph` and deterministic compiler. Workspace commands load it from files; existing commands continue accepting an inline object in the JSON request supplied through `--input`. Both modes call the same normalization, schema-validation, graph-validation, digest, lock, and package compiler functions.

The CLI does not silently switch modes. A request containing both a workspace selector and inline authoring content fails as ambiguous. Inline mode performs no project writes. This preserves current automation while preventing two implementations from drifting.

Alternative considered: translate workspace files into the current `package` object at the CLI boundary. Rejected because it would retain today's incomplete validation as a second authority and make field diagnostics depend on the entry path.

### 2. The mutable workspace is bounded beneath the existing team directory

The owned layout is:

```text
.agent-team/<team-id>/
  team.json                         # existing schema-v1 compatibility manifest
  authoring/
    workspace.json                  # revision, target profile, base, question state
    definitions/<kind>/<id>.json    # mutable portable-definition drafts
    imports/<source-digest>.json    # non-secret preserved legacy source
    reports/<operation-id>.json     # deterministic migration/validation receipts
    versions/<semver>/snapshot.json # immutable accepted source and dependency closure
```

The project root and team identity are explicit request fields. All descendant names are derived from validated IDs and normalized by the existing `projectFile` confinement logic. Existing-link escape, broken links, raw relative paths, and cross-team paths fail before mutation. Workspace writes reuse project recovery receipts and add expected-revision comparison. Immutable version snapshots use exclusive creation; a pre-existing target version is never replaced even when bytes happen to match.

Alternative considered: a user-selected workspace directory. Rejected because it expands the write boundary and complicates project routing, recovery, and distribution acceptance.

### 3. Guided authoring is a stable question graph

Extend guidance into stable question nodes with prerequisites and JSON-pointer targets. Questions cover team identity and purpose, agent and nested-team references, member cardinality and responsibility, coordinator, communication edges, task acceptance, routing, workflows and dependencies, ownership, limits, budget, I/O contracts, required capabilities, and extensions.

Answers are stored as data in `workspace.json`; prompts remain presentation text. Reopening a workspace derives the same pending nodes from the answer graph. The compiler never fills unanswered mandatory nodes with defaults. Explicit profile defaults may be proposed only as answers that remain visible and editable before acceptance.

Alternative considered: an ordered interactive wizard transcript. Rejected because branching and nested-team revisions cannot be resumed or compared reliably from conversational order.

### 4. Official schemas validate shape; a semantic pass validates the graph

At the UAR checkpoint, copy the exact official `0.1.0-draft.2` schema set from the recorded UAR commit and record every schema digest in `schemas/uar/README.md`. Generate or package a self-contained validator from those files so the shipped Node.js runtime retains no npm install requirement. The validator must apply referenced nested schemas and explicit formats, not only top-level required keys.

After schema success, a semantic pass checks:

- unique `(kind,id,version)` identities and exact digest resolution;
- agent versus nested-team member kinds, cardinalities, coordinator existence, and role references;
- communication endpoints and allowed modes;
- team nesting and workflow dependency cycles;
- workflow step dependencies and required terminal/output contracts;
- exact manifest entrypoints, files, locks, and dependency closure; and
- required capabilities/extensions against the compiler's declared support.

All errors use deterministic JSON pointers. Diagnostics include the pinned profile and schema-source revision and never include credential or grant values.

Alternative considered: continue expanding `requiredByKind` and manual allowlists in `uar-package.mts`. Rejected because that already permits nested schema drift and does not prove consumption of the provider contract.

### 5. Next-version maintenance copies source, then recomputes the closure

The next-version operation requires an explicit immutable base and a target version with greater semantic-version precedence. It copies the source graph into a new mutable draft, applies operator changes, validates the complete graph, and then writes a new immutable snapshot. Stable IDs remain stable. Changed definitions receive explicitly chosen new versions; unchanged definitions retain their existing exact references. Any adopted dependency revision updates its identity/version/digest tuple and all affected locks before the snapshot is accepted.

Package and definition digests are always derived after canonical validation. No `latest` alias participates in compilation, and an installed or snapshotted version is never edited in place.

Alternative considered: mutate the current workspace version and rely on UAR immutable-conflict rejection later. Rejected because local authoring would already have lost the prior source and diagnostics.

### 6. Migration is a separate, loss-accounted adapter

Implement explicit adapters for schema-v1 `team.json` and Draft 0.1.0-draft.1 authoring envelopes. The adapter writes a preserved source record plus a report containing, for every input field: source pointer/profile, target pointer/profile, disposition (`exact`, `translated`, `optional-unsupported`, `required-unsupported`), reason, and effective target reference when available.

Unknown optional values remain in the preserved source record but do not become behavior. Any required-unsupported field blocks snapshot and package creation. Private authority or secret-shaped input is rejected at the trust boundary; its report contains the pointer and classification, never the value. If original bytes cannot safely be copied, the receipt records their external path and digest instead.

Alternative considered: keep legacy data only in a generic `extensions` object. Rejected because storage alone does not show whether the value is effective and would permit a required field to be silently ignored.

### 7. Source remains canonical; compiled and distribution trees are generated once

Expected source ownership is limited to:

- `skills/process/agent-team-creator/runtime/src/{cli,types,guidance,project-files,validation,uar-package}.mts`;
- new focused authoring, schema-validation, graph-validation, and migration `.mts` modules under the same directory;
- authoring request assets, schema snapshots/provenance, `SKILL.md`, and authoring/UAR references;
- integration sources under `runtime/test-src/` written only after production implementation is complete.

The runtime TypeScript build refreshes `scripts/*.mjs`. The existing root distribution generator then refreshes the Claude and Codex payloads; generated files are not hand-edited. A generator code change is needed only if the existing recursive skill copy demonstrably omits a new source artifact.

### 8. One final gate crosses the packaged creator and live UAR boundary

After all production code, schemas, guidance, compiled payload, and distributions are complete, add and run one local integration gate. It uses the packaged creator from a generated distribution, a temporary authorized project, and an isolated persistent UAR data root. The scenario:

1. migrates a schema-v1/Draft 0.1 fixture and checks complete diagnostics;
2. resumes guided authoring of agents, a nested team, and a workflow;
3. rejects invalid nested and required-unsupported cases;
4. builds a valid `0.1.0-draft.2` package and a distinct next version without changing the base;
5. sends the package through live UAR capability, preflight, install, and status APIs;
6. restarts or cold-reopens the persistent UAR catalog and reads back the same immutable closure; and
7. inspects emitted bytes to prove no credential values, installed grants, or executable approvals were exported.

The gate does not claim team activation unless the pinned UAR checkpoint actually exposes and supports that contract. No unit or mock-only test is completion evidence.

The provider identities have distinct meanings: `41375cf6cd137a8a825be102c49516211c3fa2e5` supplies the immutable schema bytes, `a64bafbb3d4cc54a22a5eecef2362300a959de62` is the first draft.2 runtime ancestor, and the supplied executable-checkpoint ref is resolved against the supplied UAR root when the gate runs and must equal frozen final C03 production head `48266f6ca704a04aa99aa4e92444a0ae0b62ad1d`. The mini repository owns the single cross-pack runner. It verifies both packaged Codex payloads offline, installs and exports the creator-authored TeamDefinition package, and uses a separate single-Agent provider fixture for UAR's ordinary binding/run path. The gate records the executable byte digest and writes one canonical receipt byte-identically into the mini, full-pack, and UAR evidence directories.

## Risks / Trade-offs

- **The `0.1.0-draft.2` profile is not yet present at the inspected UAR revision** → Block schema implementation on an exact provider commit and digest inventory; do not derive it from Draft 0.1.
- **A mutable workspace can diverge from an immutable snapshot** → Store the selected base and expected revision, validate the full graph, and create snapshots exclusively with content receipts.
- **Nested schemas can validate while semantic references remain invalid** → Keep schema and graph validation as two explicit required stages with one combined diagnostic report.
- **Migration can preserve bytes without preserving behavior** → Require a disposition for every source field and block on every required-unsupported result.
- **Generated trees create a large diff** → Finish source implementation first, build the compiled payload once, regenerate both distributions once, and verify source/package parity in the final gate.
- **The live UAR gate depends on a changing sibling product** → Pin the UAR commit, profile, endpoint contract, and isolated persistence configuration in the receipt; an unavailable or incompatible runtime leaves acceptance blocked rather than downgraded to static validation.

## Migration Plan

1. Record the official UAR `0.1.0-draft.2` commit, schema paths, digests, and supported package API contract. Stop if this provider checkpoint is unavailable.
2. Add the neutral authoring graph and complete validator while leaving schema-v1 `team.json` and existing inline request handling readable.
3. Add bounded workspace operations, guided questions, migration, and immutable next-version maintenance against the same compiler.
4. Update skill guidance/assets/schema provenance and compile the complete TypeScript runtime into the source skill's `.mjs` payload.
5. Regenerate both distributions from canonical source. Do not edit generated files directly.
6. Add and run the one final local live-UAR gate at the completed phase boundary.

Rollback removes the additive authoring commands and generated copies while retaining prior schema-v1 team files and Draft 0.1 inputs. New `0.1.0-draft.2` snapshots are immutable user data; rollback does not rewrite or delete them. Older code must report them unsupported rather than opening them as Draft 0.1.
