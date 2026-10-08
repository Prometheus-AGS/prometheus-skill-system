## Recovery contract

`start.correction` must point to the last finalized failed iteration and a local failure receipt identifying that iteration and its observed operation. An unrelated scope cannot use the correction exception.

`failure resolve` accepts the last finalized failed iteration, a nonempty reason and authority reference, and an existing local evidence file. It records the file hash and an explicit `retired` disposition. This does not mark work successful, increment delivery counts, satisfy KBD completion, or clear publication debt. It only permits the next independently authorized scope to enter after ordinary budget, review, hook, and publication-capacity checks.

The C08 passed gate is linked as independent phase evidence. Its source revisions are not rewritten to match the old candidate; missing build/command logs remain missing. The old candidate remains failed.

The full skill is the source. After its commit, `sync-mini.mjs` copies identical bytes into mini and records the exact full commit and payload digest. The Boss mini gitlink and packaged payload then advance to the committed mini revision. Its pinned native `prometheus` executable retains the revision that built the executable until a native rebuild supplies matching artifacts.

### Frozen multi-source corrective evidence

The frozen-artifact path may reconcile a candidate with multiple already-frozen sources only when the first source is the application source named by the operation evidence and remains clean: its preserved tracked patch and untracked-file list are empty. Every secondary source retains an empty tracked patch and its current reference must exactly equal the candidate reference. A secondary preserved untracked file is admissible only if its preserved copy and the current source file are regular, non-linked files with identical bytes. The reconciliation receipt continues to bind the entire candidate source array; only the separately bound application driver may advance, and it cannot add, remove, reorder, replace, or generally waive dirty inputs. Application packaging, skills, assets, build provenance, and the external-driver boundary remain unchanged.
