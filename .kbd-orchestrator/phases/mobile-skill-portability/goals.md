# Goals: mobile-skill-portability

Seeded from: `ideation-and-decision-tools/reflection.md`
Created: 2026-07-31T09:53:02Z

## Seeded Goals

- **Author the `prometheus:component/*` WIT family** — the decision and its
  ordering constraint are recorded; the authoring is not done. This blocks
  porting, by design.
- **Build the `fabric-integration` skill** — makes the four version invariants
  enforced rather than documented.
- **Mobile FFI bindings** — this pack has no cdylib/staticlib and no uniffi;
  `frf-ffi` (uniffi 0.31.2) is the pattern to copy.
- **Verify `opencode` and `kimi` Tier 1**, and route `zed` to the file-pair
  branch or state why not.

---

## Constraints carried forward

Findings from the previous phase, not preferences. They should survive into
planning or be explicitly overruled with a stated reason.

### 1. The WIT decision is settled; the authoring is not

[`docs/decisions/wit-world-unification.md`](../../../docs/decisions/wit-world-unification.md)
fixes both the shape and the **ordering**: unify before porting a single skill.
Do not reopen the decision — implement it.

The divergence is **four packages across two repositories**, verified on disk
2026-07-31:

| Package | Version | Location |
|---|---|---|
| `uar:skill@0.1.0` | 0.1.0 | `universal-agent-runtime/wit/uar-skill.wit:12` |
| `uar:plugin@0.1.0` | 0.1.0 | `universal-agent-runtime/wit/uar-plugin.wit:12` |
| `knowme:plugin@0.1.0` | 0.1.0 | `knowme_plugin_host/wit/knowme-plugin.wit:17` |
| `knowme:plugin@1.0.0` | **1.0.0** | `knowme_plugin_host/wit/v1/types.wit:14` |

Porting first means every skill is ported twice and parity becomes true by
construction rather than by measurement.

### 2. The four version invariants are documented, NOT enforced

Three of four hold today and **nothing checks any of them**
([`fabric-version-invariants.md`](../../../docs/decisions/fabric-version-invariants.md)).
Each failure mode is silent or misattributed:

| Invariant | Status | Failure if violated |
|---|---|---|
| Loro minor aligned (1.13) | holds | decode error or silently divergent doc **on merge** |
| wasmtime major aligned (46) | holds | `.cwasm` cache useless across majors |
| iroh ≥ 1.0.2 | **enforced** by change-idt-008 | relay DoS (fixed) |
| WIT world version pinned | **not held** | `knowme:plugin` already resolves ambiguously |

`fabric-integration` is what converts these from documented to enforced.

### 3. Cross-repo code needs explicit authorisation

The previous phase was **design and record only** — verified by timestamp that
zero files were touched in `flint-realtime-fabric`, `universal-agent-runtime`, or
`know-me-system`. If this phase writes into those repos, that is a scope change
the user agrees to first, not an inference from "the decision is recorded".

### 4. Tier 1 is verified on `codex` only

`opencode` and `kimi` share the identical `_render_tier1_file_pair` path and are
*expected* to behave the same. Expected is not verified —
[`harness-delivery.md`](../../../skills/process/ideation-mindmap/references/harness-delivery.md)
says so. `zed` is detected in `_detect_harness` but its Tier 1 dispatch
(`ui-surface/scripts/render.sh:174`) routes only `opencode|codex|kimi`, so zed
falls to Tier 0.

### 5. Pre-existing, NOT this phase's to absorb

Two `sovereign-sync` integration tests fail
(`one_projects_token_is_rejected_by_another_project`,
`two_projects_mint_distinct_identities_and_tokens`). Confirmed via `git stash`
that they fail identically without the previous phase's changes. Control-token
derivation, unrelated to iroh or WASM. Fix deliberately or leave deliberately —
do not let them quietly become this phase's failure.

### 6. Two lessons that shaped the previous phase

- **"Demonstrated" is not "enforced."** Acceptance criteria must make the
  property impossible to violate, not show one case where it held.
- **Verify the plan's facts at write time.** Two of the previous plan's factual
  claims had decayed between analyze and execute.

---

## Instructions

Review and refine the goals above before running `/kbd-assess`.
Add, remove, or clarify as needed. When ready:

```
/kbd-assess mobile-skill-portability
```
