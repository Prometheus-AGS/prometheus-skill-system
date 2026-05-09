# Bug Fix Ledger

Quarterly review of bug fixes shipped in the Prometheus stack.
Each entry records what broke, why it broke, and what prevented recurrence.

**Process:** Review quarterly (last week of each quarter). PR author or oncall
creates one entry per notable bug fix. Trivial cosmetic fixes may be omitted.

---

## Q2 2026 — First Review (2026-05-09)

### BF-001 — `unused import std::io::Write` in pk-event-store/migrate.rs

| Field | Value |
|-------|-------|
| Repo | prometheus-knowledge |
| PR/commit | Phase 4 execution, commit during dual-store implementation |
| Severity | LOW (compiler warning, CI pass-through) |
| Introduced | Added `use std::io::Write;` inside function body in addition to existing top-level import |
| Symptom | `cargo check` warning: unused import `std::io::Write` in `apply()` function |
| Root cause | Duplicate `use` statement inside `apply()` body — `Write` was already imported at the module level |
| Fix | Removed redundant inner-function `use std::io::Write;` |
| Recurrence prevention | Stricter CI: `cargo check` with `-D warnings` flag (to be enforced) |

---

### BF-002 — Rust type mismatch in keyword_extract test: String + String vs String + &str

| Field | Value |
|-------|-------|
| Repo | prometheus-knowledge (pk-librarian) |
| PR/commit | SP-002 implementation |
| Severity | MEDIUM (compile error blocked test run) |
| Introduced | `"SurrealDB ".repeat(200) + " PostgreSQL ".repeat(100)` — RHS is `String`, but `+` operator requires `&str` |
| Symptom | `error[E0308]: mismatched types — expected &str, found String` |
| Root cause | `.repeat()` returns an owned `String`; the `+` operator on `String` requires the RHS to be `&str` |
| Fix | Added `&` prefix to second operand: `+ &" PostgreSQL ".repeat(100)` |
| Recurrence prevention | Rust ownership model: always borrow with `&` when using `+` on String |

---

### BF-003 — MIN_SCORE absolute threshold produced empty keyword extraction output

| Field | Value |
|-------|-------|
| Repo | prometheus-knowledge (pk-librarian) |
| PR/commit | SP-002 keyword extraction |
| Severity | HIGH (feature produced wrong output silently) |
| Introduced | Fixed `MIN_SCORE = 0.15` threshold in `extract_from_window()` |
| Symptom | Test with many unique equal-weight tokens produced 0 keywords, fell back to raw prompt truncation (118 words, not 12 keywords as expected) |
| Root cause | All tokens scored ~0.029 (evenly distributed TF), all below the fixed 0.15 floor — resulting in an empty ranked list |
| Fix | Replaced absolute `MIN_SCORE` filter with dynamic cutoff: `top_score * 0.1` |
| Recurrence prevention | Tests now verify keyword count, not just non-empty output. Dynamic threshold adapts to distribution. |

---

### BF-004 — pipeline-enforce.sh grep patterns required literal `"` prefix

| Field | Value |
|-------|-------|
| Repo | prometheus-skill-pack |
| PR/commit | SP-012 pipeline enforcement hook |
| Severity | HIGH (hook silently failed to block out-of-order commands) |
| Introduced | Grep pattern `'"kbd-execute'` required `"kbd-execute` in input, but test inputs passed `kbd-execute` without leading quote |
| Symptom | Tests 2 and 4 of test-pipeline-smoke.sh failed: hook did not block when it should have |
| Root cause | Pattern assumed Bash tool input is always JSON-encoded with a leading `"`, but test harness passes plain strings |
| Fix | Changed grep patterns to bare `'kbd-execute'` and `'kbd-reflect'` |
| Recurrence prevention | Smoke tests now cover the blocking and pass-through cases for both commands |

---

### BF-005 — bdd-video-proof SKILL.md `version` field validation failure

| Field | Value |
|-------|-------|
| Repo | prometheus-skill-pack |
| PR/commit | BDD-004 video proof skill productization |
| Severity | LOW (validation failure, skill would not install) |
| Introduced | `version` field placed under `metadata:` block instead of top-level frontmatter |
| Symptom | `npm run validate:strict` failed: "version field missing at root level" |
| Root cause | AgentSkills.io spec requires `version` at the root frontmatter level alongside `name` and `description`. Placed it under `metadata:` by habit from other formats. |
| Fix | Moved `version: '1.0.0'` to root frontmatter |
| Recurrence prevention | SKILL_TEMPLATE.md explicitly marks the root-level fields; future skills can reference it. |

---

## How to add an entry

1. Copy the BF-NNN template above.
2. Increment the ID.
3. Fill in all fields. "Recurrence prevention" is mandatory — if we can't prevent it, escalate to a team discussion.
4. PR this file with the fix itself (or as a quarterly batch).

## Severity scale

| Level | Definition |
|-------|-----------|
| CRITICAL | Data loss, security breach, or production outage |
| HIGH | Feature produces wrong output silently or CI gate failure |
| MEDIUM | Compile/type error blocking development |
| LOW | Warning, cosmetic, or install-only failure |
