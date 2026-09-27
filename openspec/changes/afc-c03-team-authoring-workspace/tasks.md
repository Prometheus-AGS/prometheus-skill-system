## 1. Provider checkpoint and full-pack mirror

- [x] 1.1 Record mini source commit `1557c842bd5fdb3b018a24bac86605a4e8c04233` and accepted UAR provider commit `41375cf6cd137a8a825be102c49516211c3fa2e5`, mirror the versioned draft.2 schemas byte-identically, and retain the draft.1 snapshot for migration; evidence: `evidence/shared-source-receipt.json` records C03.1-C03.3, every shared source/destination path, and its SHA-256 digest.
- [x] 1.2 Mirror the neutral authoring graph, workspace index, stable guidance, source-import, migration-diagnostic, and immutable-maintenance TypeScript and schemas beneath `skills/process/agent-team-creator`; the full pack consumes the frozen mini source instead of creating a second compiler or authority model.
- [x] 1.3 Preserve explicit confined workspace selection and the existing inline `--input <request.json>` carrier through one canonical package capability; workspace files remain split by document and private binding state remains outside portable package source.

## 2. File-backed authoring and canonical validation

- [x] 2.1 Mirror the partitioned `runtime/src/uar-package/` implementation for workspace I/O, provider profile validation, graph validation, migration, compilation, maintenance, package files, binding, and private-authority checks; `uar-package.mts` remains the thin public entry point and no generated `.mjs` was edited.
- [x] 2.2 Mirror bounded workspace initialization, loading, document update, containment and case-collision checks, same-directory atomic writes, and bounded status paging; validation and final runtime execution remain deferred to the single final gate.
- [x] 2.3 Mirror self-contained draft.2 schema and semantic graph validation for a single TeamDefinition entrypoint, kind-correct immutable references, coordinator and workflow roles, acyclic dependencies, limits, exact versions, and exact digests.
- [x] 2.4 Mirror legacy flat-team, AgentArtifact, draft.1, and inline-envelope normalization into the shared draft.2 workspace graph while leaving existing draft.1 compiled packages readable and unchanged.

## 3. Lossless maintenance and UAR deployment boundary

- [x] 3.1 Mirror deterministic field-level migration receipts for exact, translated, optional-unsupported, and required-unsupported values, including source/target pointers and effective target references; required loss blocks build and preflight.
- [x] 3.2 Mirror immutable next-version workspace initialization and definition-identity/JSON-Pointer diffing, including refusal of existing destinations and same-version content changes.
- [x] 3.3 Mirror draft.2 package and binding preflight/install/status adapters through the existing UAR client seam; installation precedes the separate private binding compare-and-swap path.
- [x] 3.4 Mirror the portable authority boundary that rejects credential values, RepresentationGrant records, consent evidence, and installed authority while permitting only opaque private binding references.

## 4. Guidance, documentation, generated parity, and final gate

- [x] 4.1 Mirror creator intake assets, split workspace examples, `SKILL.md`, manifest/UAR references, and manage/handoff boundary guidance for bounded authoring, immutable maintenance, and creator-owned definition changes.
- [x] 4.2 Update full-pack `docs/agent-teams.md` and `docs/guide/24-agent-teams.md` for legacy AgentArtifact staging, draft.2 canonical packages, private bindings, installed authority, and unsupported activation; record the source checkpoint and shared-file digests without claiming generated or runtime acceptance.
- [ ] 4.3 After both repositories' production source is frozen, run the TypeScript 7 build once to refresh `scripts/*.mjs` and the repository distribution generator once for Claude and Codex copies; verify generated payload parity against the recorded shared source checkpoint without hand-editing generated files.
- [ ] 4.4 Run the mini-owned cross-pack gate once against both packaged Codex payloads and the live UAR executable. Resolve the supplied executable-checkpoint ref against the supplied UAR root; treat `41375cf6cd137a8a825be102c49516211c3fa2e5` only as the immutable schema source and `a64bafbb3d4cc54a22a5eecef2362300a959de62` only as the first draft.2 runtime ancestor; require and record the frozen C03 production head established at the completed implementation boundary plus the executable byte digest. Cover offline migration, bounded nested graph authoring, negative graph/private-authority cases, immutable maintenance in both packages, then mutate catalog/grant/binding/run/export once, cold-read the result, and copy one byte-identical receipt to mini, full, and UAR evidence.
