# Proposal — change-lgv-003-dataset-schema-and-index

Define and document the eval item schema, then build an `index.json` that
lists every eval item produced in change-lgv-001/002 for the harness (
change-lgv-004) to iterate over deterministically.

Schema fields per item: `item_id`, `domain`, `corpus_path`,
`explanation_text`, `ground_truth: {scores: {completeness, accuracy,
clarity, misconceptions_absent}, misconceptions_present: [...]}`,
`review_status: draft|reviewed`.

## Goal
G-01 (completion — schema + index tie the raw content together).
