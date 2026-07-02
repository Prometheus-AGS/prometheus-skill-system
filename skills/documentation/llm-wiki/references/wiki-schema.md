# Wiki Schema — layer 3 of the LLM wiki

This is the schema document in the Karpathy three-layer architecture: the
configuration that turns an LLM into a disciplined wiki maintainer. It fixes
the directory structure, page conventions, and operation workflows for every
pk-backed wiki this skill operates on. When practice and this document
diverge, update one of them — silent drift is how wikis die.

The normative format is OKF v0.1 (`shared/references/okf-v0.1.md` in this
repo). This document instantiates it; where they conflict, the OKF spec wins.

## Directory structure

The wiki lives in the pk KB directory (`<project_root>/.prometheus/knowledge/`
inside a project, `~/.prometheus/knowledge/` or `$PK_KB_DIR` otherwise):

```
knowledge/
└── wiki/                      # the OKF bundle root
    ├── index.md               # catalog of everything (content-oriented)
    ├── log.md                 # append-only history (chronological)
    ├── <concept>.md           # root-level concept pages
    └── <group>/               # subdirectories group related concepts
        ├── index.md           # per-directory listing
        └── <concept>.md
```

- The bundle root is `wiki/`. Concept IDs are wiki-relative paths without
  `.md`: `wiki/patterns/actor-model.md` → `patterns/actor-model`.
- Subdirectories are free-form — organize by what the knowledge is about
  (entities, patterns, sources, analyses), not by when it arrived.
- `index.md` and `log.md` are reserved at every level (OKF §3.1) and are
  never concept pages.
- Raw sources live **outside** the bundle (wherever the user keeps them).
  The wiki references them via Citations; it never contains or edits them.

## Page conventions

Every concept page is UTF-8 markdown: YAML frontmatter, then body.

### Frontmatter

```yaml
---
type: Reference            # REQUIRED — non-empty string
title: Actor model in Prometheus
description: One-sentence summary used by index generators and search.
tags: [rust, concurrency]
timestamp: 2026-07-02T06:15:00Z
---
```

- `type` is the only hard requirement (OKF §9). Values used in this wiki,
  in rough priority order: `Reference` (distilled knowledge from sources),
  `Entity` (a person/system/component page), `Concept` (an abstract idea),
  `Analysis` (a filed query answer), `Playbook` (procedural steps),
  `Source` (a page about one raw source). New types are allowed — pick a
  descriptive singular noun and keep using it consistently.
- `title`, `description`, `tags`, `timestamp` should always be present on a
  finished page. pk's writer emits `title`, `tags`, and `timestamp` (plus
  the extension keys `id`, `sources`, `created_at`, `updated_at`,
  `revision` — preserve them when editing); `description` is emitted only
  when set, so add one during ingest integration if pk did not — index
  generation depends on it.
- `timestamp` is the last *meaningful* change, ISO 8601, updated on every
  content edit (not on mechanical link fixes).
- Producer-defined extra keys are fine; never strip keys you don't
  recognize when editing a page (round-trip preservation, OKF §4.1).

### Body

Favor structural markdown — headings, lists, tables — over prose walls.
Conventional sections (use when applicable, omit when empty):

| Section | Use for |
|---|---|
| `# Schema` | Field/column breakdown of a described asset |
| `# Examples` | Fenced code blocks, concrete usage |
| `# Citations` | Numbered external sources for body claims (OKF §8) |

Contradiction convention: when new knowledge conflicts with an existing
claim, do not silently overwrite. State the newer claim, then flag the
conflict inline on **both** pages:

```markdown
> **Conflict:** [older-page](/group/older-page.md) (2026-05-10) claims X;
> the 2026-07-01 source says Y. Unresolved — prefer Y pending verification.
```

### Cross-links

- Bundle-relative form, always: `[customers table](/tables/customers.md)`.
  Stable under subdirectory moves; this is what pk's link scanner indexes.
- A link is an untyped directed edge — the surrounding sentence carries the
  relationship ("joins with", "supersedes", "part of").
- Broken links are legal (OKF §5.3) and meaningful: they mark pages that
  should exist. Lint surfaces them as missing-page candidates rather than
  errors.

## index.md format

Content-oriented catalog, one entry per page, grouped by section. No
frontmatter, except the bundle root MAY carry `okf_version: "0.1"` (the only
index frontmatter OKF permits, §11).

```markdown
# Patterns

* [Actor model in Prometheus](patterns/actor-model.md) - Tokio-native actor pattern with typed mpsc messages.
* [Async guardrails](patterns/async-guardrails.md) - Blocking-guard and shutdown conventions.

# Sources

* [OKF v0.1 spec](sources/okf-v0-1.md) - Vendored spec snapshot with adoption notes.
```

Rules:

- Entry description = the page's frontmatter `description`, verbatim. If
  they diverge, the frontmatter is authoritative — fix the index.
- Every concept page in a directory appears in that directory's `index.md`;
  the root index links subdirectories (`[Patterns](patterns/)` style) rather
  than duplicating their contents once a group grows past ~10 entries.
- Updated on **every** ingest and on any lint fix that adds/removes pages.
  pk's ingest path maintains this automatically (change-okf-005); manual
  edits must keep the same shape.

## log.md format

Append-only, newest first, date-grouped (OKF §7), with grep-able entry
prefixes:

```markdown
# Wiki Update Log

## 2026-07-02
* **Ingest**: [OKF v0.1 spec](sources/okf-v0-1.md) — vendored spec compiled; 3 pattern pages cross-linked.
* **Lint**: 2 orphans linked in, 1 contradiction flagged on [actor-model](/patterns/actor-model.md).

## 2026-07-01
* **Initialization**: bundle created; root index established.
```

- Date headings are ISO `YYYY-MM-DD`. Leading bold word is the operation:
  `**Ingest**`, `**Query**` (when an answer is filed back), `**Lint**`,
  `**Update**`, `**Initialization**`.
- One entry per operation, not per touched file — name the pages that
  changed inside the entry.
- `grep "^## " log.md | head -5` must always yield the most recent activity
  dates; never edit or reorder old entries.

## Operation workflows

### Ingest

1. Read the raw source (never modify it).
2. `pk ingest <file>` (or stdin) — pk compiles the source into an OKF page,
   places it, and updates `index.md`/`log.md`.
3. `pk search` for affected existing pages → update them: revise claims,
   add cross-links both directions, flag contradictions per the convention
   above, bump `timestamp`.
4. Add `# Citations` entries pointing at the raw source.
5. Summarize to the user: pages created/updated, contradictions, gaps.

### Query

1. Read root `index.md` → `pk search` / `pk focus` → read candidate pages.
2. Answer with per-claim citations (wiki links or external URLs).
3. If the answer produced new synthesis, file it back: ingest the answer as
   an `Analysis` page, cross-linked to every page it drew from, and let the
   log record it as a Query entry.
4. If the wiki can't answer, name the gap explicitly as an ingest candidate.

### Lint

1. `pk lint` — mechanical OKF conformance (frontmatter parses, `type`
   non-empty, reserved-file shape).
2. Semantic pass: contradictions, stale claims (`timestamp` vs newer
   ingests), orphans, missing pages (recurring broken links), missing
   cross-references, data gaps.
3. Prioritized findings to the user; apply approved fixes; log the pass.

## Scale notes

Index-first navigation works to roughly hundreds of pages; past that, lean
on `pk search` (TF-IDF) and `pk focus` (retrieval + synthesis) rather than
reading indexes linearly. The wiki is a git-friendly directory of markdown —
version it with the project when project-scoped, and treat `~/.prometheus/`
wikis as machine-local state backed up like any other data.
