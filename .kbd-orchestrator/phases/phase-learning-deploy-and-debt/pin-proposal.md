# Final release proposal

The owner-selected full skill-system **dependency** remains `ba5c4516fd72e60c21ab3bb1d09a63105aeb3c61`. Companion's four Cargo consumers now use that SHA. This does not rename it as a memory revision or silently advance it to the future full-pack release commit.

## Concrete protected-file proposal

`companion-versions-proposed.toml` and `companion-versions.patch` propose importing exactly the eleven pin values already present in the preserved original Companion checkout into the isolated candidate's empty `[pins]` authority. The original file and candidate versions.toml are untouched. No new architecture decisions, image upgrade, Node change or real-home configuration change is included. Cargo iroh will be aligned to this existing proposed1.0.3 selection after approval.

Mini's final memory pin must identify the new cache/hook source, not the old v1.10.1 baseline. Its exact protected versions.toml diff will be presented when the certified dependency commit/merge SHA exists; no fabricated or floating SHA is proposed now. Existing knowledge4b3a674, liter12a2fae and SurrealDB3.3.0 image/digest remain selected.

## Release versions and tag names

| Repository | Candidate release | Proposed signed tag | Source target |
| --- | --- | --- | --- |
| Full skill system |1.11.3 |v1.11.3 |Certified, owner-merged final phase source |
| Mini skill package |1.11.3 |v1.11.3 |Certified, owner-merged final phase source |
| Surreal-memory server |1.10.2 |v1.10.2 |Certified, owner-merged cache/identity/accounting and hook source |
| Optional Companion |0.1.0 |v0.1.0 |Certified, owner-merged recovered service/package source in its private remote |

These tag names are approval proposals, not existing tags or release evidence. Preserve tag.gpgsign=true and the configured signing identity; no unsigned-tag workaround. Never retag v1.10.1. All local gates and owner merges precede release tagging/deployment.

`plugin-version-proposal.json` lists one patch increment for every owned full marketplace payload, as required by the release checklist and refreshed shared generation. Vendor/import plugin releases remain their upstream versions. Existing independent binary/API/protocol versions do not cascade with packaging versions; records distinguish them. Full has no root versions.toml: existing `config/release-version-matrix.json`, source package declarations and gitlinks remain authoritative.

## Fresh dependency observations

`release-candidates.json` inventories all12 full/mini direct gitlinks and four isolated production candidates. All12 upstream fetches succeeded at the recorded cutoff. Entity-management071b9e5b and openai-proxyad32f5d contain already merged work ahead of the current consumer pins; include them in the final compatible graph after source reconciliation. Other tool/import selections remain their recorded exact commits. The existing memory lock selects mempalace f8a1b4c86ef28f8f8070142d600e650f187c538d; replace its floating branch selector with that exact existing locked revision rather than an unreviewed dependency jump.

## Release ordering requiring an explicit policy exception

The final signed dependency commit cannot be identified before it is created. Committing requires local gates, while the current whole-phase policy also requires the final parent gitlink/versions metadata before those gates. This is an ordering cycle. The proposed exception to the **Immutable Implementation-First and Integration-Only Policy** changes only release-metadata sequencing:

1. Finish every functional source and packaging implementation across full, mini, memory and Companion; obtain the required protected-file approvals. Keep reviewers/tests dormant until this source freeze.
2. Run one consolidated local integration batch against the recorded actual candidate checkouts and real production entrypoints with scratch homes/data/ports. This initial gate includes the new memory source directly and is not installable-parent certification.
3. Commit and certify the dependency from committed state; publish its review branch after passing gates. The owner merges it. Record the actual signed commit/merged SHA, then request the exact mini versions.toml pin patch approval.
4. Apply the approved parent gitlinks and matching release metadata as a coherent batch. Run final fresh-clone/install/full integration certification against the exact remotely resolvable graph; rerun only affected gates if fixes are needed. Independent cumulative review follows completed source, and all PR merges remain the owner's.
5. Tag approved merged releases and perform the authorized machine/site deployment, with separate source, installation and functional-operation receipts.

This exception permits no test-first work, incomplete functionality, mocks as acceptance, hosted validation, unapproved version-file edits, blind pin substitution, fabricated SHA, premature release claim or agent merge. Without this explicit exception, release closure remains blocked by the cycle rather than being called complete.
