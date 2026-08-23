# Design: explicit target source-tree lifecycle

`skill-system.json` owns the canonical target matrix, so each target now declares
`sourceTreeLifecycle` as either `required` or `install-only`. The required set is the five
tracked harness trees currently present in this repository: OpenCode, Cursor, Codex, Devin,
and Agents. The remaining targets are installation destinations and therefore declare
`install-only`; their absence from source is intentional rather than silent drift.

`readSkillSystem()` is the shared validation boundary used by distribution generation and
both installation paths. It rejects omitted or unknown policies and missing or empty
required trees before staging begins. A dedicated fixture test covers omitted, missing,
empty, install-only-absent, and repeated-validation behavior.

No normalizer is appropriate. These trees do not carry a generated `internal: true`-style
marker or another invariant that could be deterministically re-applied. Validation is the
complete control and is intentionally read-only.
