# C15.3 next implementation slice: reviewed skill coverage and host authority

2026-10-08. Planning handoff only; no source or canonical task transition. Source inspected: UAR checkout `3daa3ef89aaebe0da379ca42b1582172d21f127a`, its locally available `origin/main` `8f9f08630f951e5c30ddff75059f7eb91c538f20`, and The Boss C15 checkout at `dca532dd86bf895cc115490b9abbf069f457d10c`. The UAR `origin/main` change since `3daa3ef` is a merge of that branch and has no new skill-registry, deployment-catalog, or collaboration-binding code. Remote freshness was not asserted; no fetch was performed. **Chronology correction:** handoff provenance source is now implemented in full pack `609ab9a`, mini `7d8e098d8f0056a9c172bf85fc12b0bee865043e`, and Boss `96492a21bc`; Delivery 10's packaged operation passed. The next production slice is lock/host coverage only.

## Existing contracts to retain

- UAR `src/uar/runtime/skills/builtin_loader.rs` hashes the raw `SKILL.md` into `Skill.artifact_digest`; `src/uar/domain/skills.rs` stores exact version, digest, installed location, entrypoint and required tools. `src/uar/api/skills/deployment_catalog.rs` returns those values to an authenticated administrator and keeps location private. `src/uar/compiler/collaboration/bindings.rs` checks exact installed identity at binding and revalidates it on use. Package-definition locks and capability declarations in `src/uar/compiler/collaboration/validation/{schema,graph}.rs` are a different contract; they do not attest plugin payload coverage.
- The Boss `src/main/ai/runtime/uar/UarTeamSkillCatalogAdapter.ts` already binds exact selected SkillRefs from that catalog. `UarTeamAuthoringAdapter.ts` already rejects a required tool absent from a member's selected tool set. `uarTeamHostScope.ts` resolves the exact installed private binding and member tool set; `UarTeamHostService.ts` validates live invocation membership and delegates effect decisions to `UarHostToolAdmission` and `UarAuthorityProvider` (local or Flint). A reviewed skill must constrain this authority; it must never create a permit.
- The full pack's ordinary Claude/Codex `plugin.json` declares a plugin name/version and skill directory, not file hashes or grants. A separate full-pack installed generation has `manifest.json` plus Ed25519 `manifest.sig.json`, verified against a local trust store by `scripts/install-plugin-generation.js`. Mini's generated `plugin.json` and `skill-system.json` do not provide that signed-generation contract. Do not call either ordinary plugin descriptor a reviewed lock.
- The Boss `src/main/utils/prometheusPack.ts` copies its packaged mini payload to writable user data and reuses it when `release-manifest.json` matches. It does not presently attest every copied file. `UarSidecarService.ts` points UAR `UAR_BUILTIN_SKILLS_DIR` at that writable copy. This is the exact integrity boundary: a packaged inventory alone proves neither the current runtime bytes nor a permission grant.

## Smallest independently useful increment

In the existing Team Settings skill selector, identify each required skill as **reviewed and deployable**, **installed but unreviewed**, or **blocked** with a reason. A built-in mini skill becomes reviewed only when its selected `SkillRef` is covered by a build-pinned payload inventory and UAR's current `SKILL.md` and required runtime closure still match that inventory. Before deployment, recheck the same identity and ensure the member's required tool names are within the exact private host tool scope. At the actual tool call, retain the existing host admission, human challenge and local/Flint revalidation. A content match never implies approval.

This slice must carry a real reviewed *closure*, not just the top-level `SKILL.md`: executable scripts, libraries, references and other files invoked by that skill are covered by relative path and digest. Build it from the pack's existing distribution inventory/source and pin its own digest in the packaged application. A user-writable file or a client-supplied `SkillRef` cannot be the trust root. Preserve exact skill/version/config in UAR's existing portable `SkillRef`, while source attestation, installed path and effective host scope stay in private binding/runtime state. Support both application-owned mini and installed full-pack payloads: mini is checked against the packaged application baseline; full-pack generations must pass their existing Ed25519 signature/trust-store and target-receipt verifier *and* current installed-byte checks. An ordinary full-pack `plugin.json` or mini `skill-system.json` is insufficient. Do not invent a signature for mini or weaken the full-pack trust-store contract.

Proposed ownership and interfaces:

| Owner | Narrow production surface |
| --- | --- |
| Mini/full pack | Generate or expose a versioned, canonical per-skill closure inventory from the actual distributed files. Mini's built-in inventory is anchored by the application's packaged expected digest. Full-pack installation consumes its existing signed-generation manifest, Ed25519 trust store and signed target receipts via `scripts/install-plugin-generation.js`; the reviewed record binds that generation to current bytes. Keep pack metadata separate from portable team definitions. |
| The Boss packaging/main | Pin the mini inventory digest with the build; verify copied runtime files against the packaged inventory before claiming reviewed status. Verify an installed full-pack generation with its existing verifier and trust root before claiming it reviewed. Extend `UarTeamSkillCatalogAdapter.ts` to combine verified coverage with UAR's catalog, and `UarTeamAuthoringAdapter.ts`/`uarTeamHostScope.ts` to reject a stale reviewed lock or required tool outside the installed member scope. The existing picker `UarTeamMemberEditor.tsx`, typed `uarTeams.ts` DTO and all locales show status and repair path. `UarTeamHostService.ts` and `UarHostToolAdmission.ts` remain the only effect authority. |
| UAR runtime | Extend private `Skill`/`SkillDeploymentCatalogEntry` and binding receipt with optional verified coverage identity/status, without changing portable `SkillRef` or requiring The Boss for other hosts. Recheck exact current installed bytes and coverage at deployment/admission through `src/uar/compiler/collaboration/bindings.rs` and its existing revalidation path. Hosts lacking a trusted verifier expose unreviewed status; they must not self-attest via request JSON. |

**Concrete private contract:** reuse UAR's authenticated, owner/workspace-scoped `/api/v1/collaboration/deployment-bindings:preflight` and existing `DeploymentBinding.extensions`; add a required namespaced `urn:prometheus:uar:reviewed-skill-coverage:1` entry to the *private binding*, alongside the existing Boss host-workspace extension. Its value lists each selected exact SkillRef identity, installed-location hash, source class (`boss-packaged-mini` or `signed-full-generation`), verified manifest/generation digest, trust-root/signer identity when applicable, and a digest of the complete relative-path closure inventory. Boss main constructs it after reading trusted packaged bytes or successfully invoking the full-pack verifier; renderer and portable package never supply it. UAR parses it in `bindings.rs` on preflight/install, compares every required SkillRef and current installed bytes/closure to the attested source, binds the result into `EffectiveBindingReceipt`, and rechecks at run admission. The existing owner/workspace/admin authenticated channel is the transport; do not add a standalone attestation endpoint, scheduler or approval store. The binding's `contentDigest` already covers extensions. Binding authentication alone does not prove the manifest: UAR must verify the supplied bytes against a configured trusted packaged baseline or full-pack signature verifier; if that verifier is unavailable, the result is unreviewed/blocked for a required reviewed skill. Neither a portable `SkillRef` nor arbitrary binding JSON may self-certify. For the effective tool criterion, compare required tools against `uarTeamHostScope.ts`'s resolved `memberTools`, then let `UarTeamHostService.ts`/`UarHostToolAdmission.ts` decide each real invocation with live local/Flint authority. This verifies scope, not a standing permit.

## Pack closure inventory and full verifier contract

Each staged full generation and each staged mini plugin payload writes
`reviewed-skill-closures.json` at its payload root. It is public metadata only:
it contains no file body, credential, absolute location, or authorization grant.
The document uses `prometheus-reviewed-skill-closures-v1`, lowercase
`sha256:<hex>` digests, and RFC 8785 JSON canonicalization. It contains a sorted
deduplicated `files` table (`path`, `sha256`) and sorted skills with parsed-only
`identity` (`id`, `name`, `version`, `artifactDigest`), `entrypoint`, `roots`, and
a `closure` of sorted paths plus its digest. An absent frontmatter id or version is
`null`; the pack never invents a portable SkillRef identity. `artifactDigest` is
the raw `SKILL.md` byte digest. Each skill root is recursively enumerated, so an
added, deleted, linked, or edited file invalidates its closure. Every shared
runtime root actually shipped in that package is included conservatively in every
skill's closure, covering out-of-directory runtime support without trusting a
handwritten per-skill list. `inventoryDigest` is the digest of the document before
that field is added.

Mini has no signing addition. Boss pins the exact packaged inventory bytes/digest
with its application and compares the writable copied payload to that baseline.
For a full installation, the host invokes its independently packaged trusted copy
of `scripts/verify-reviewed-skill-coverage.js` with `--plugin-root`, `--home`, and
`--trust-store`. That wrapper first invokes the existing
`scripts/install-plugin-generation.js --reviewed-coverage` verification contract, which verifies the
active generation, manifest signature against the existing Ed25519 store, all
target receipts, and installed projections. Only on success does it emit a JSON
record with the inventory, generation and manifest digests, signer id, trust-store
digest, and receipt digests. Boss must package the wrapper and its installer
dependency closure as immutable application resources; a script inside the
writable generation is not a trust root.

The immutable verifier package carries `skill-system.json` but not duplicate
source target trees. In `--verify` and `--reviewed-coverage` modes,
`readSkillSystem` still validates its schema, all fourteen targets, each target
lifecycle value, and release/minimum-version relationship, while deliberately
omitting only source-tree existence and non-empty probes. Existing installation,
rollback, and uninstall paths continue to require those source trees. The signed
active generation, signed target receipts, and installed projections remain the
verification material for verifier mode.

## C15.3 handoff provenance already implemented

Full pack `609ab9a`, mini `7d8e098d8f0056a9c172bf85fc12b0bee865043e` and Boss `96492a21bc` have implemented the handoff provenance source. Delivery 10's actual packaged operation passed. Preserve its source and operation evidence; **do not rerun passing handoff behavior** in the next skill-lock operation. Do not claim installed acceptance before its receipt exists.

## One completed-boundary operation

Using a packaged Mac ARM64 application, create a reusable coding team with one reviewed mini skill and one reviewed installed full-pack skill. Observe the picker status, deploy and run the selected skills. Alter a disposable copied skill closure file, then attempt a new deployment: the reviewed claim must be rejected while the existing run remains pinned. An altered full-pack generation signature/target receipt must fail the established trust check. A required tool outside the member host scope must fail binding; a permitted write still enters the existing host approval path. Preserve exact build, launched-app and operation receipts. Reuse Delivery 10's successful handoff evidence rather than repeating it. Windows x64 uses the same shipped contract at its next installed release gate. Do not run per-edit/unit tests.

## Completion boundary and limits

Whole C15.3 may close only when the new reviewed mini/full-pack lock and effective host-grant coverage succeed **and** the prior handoff implementation and Delivery 10 operation receipts are present. It does not claim C15.2's eight-harness qualification, D-KNOWME/D-FORGE host coverage, or stronger model/cost metadata. Content hashes establish equality to a trusted baseline; Ed25519 proves a signer under an explicit trust store; neither grants runtime tool authority. Signed full-pack generation consumption remains an explicit C15.3 requirement, not a silent deferral.
