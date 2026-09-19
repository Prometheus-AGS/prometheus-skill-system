# Thread contracts

Normative shapes for the artifacts a research thread writes, and for the ledger
that tracks every dispatch. Introduced by `change-drt-001`; consumed by the
scheduler (`drt-002`), the director and worker agents (`drt-003`), and the
deterministic merge (`drt-004`).

**Why these shapes are fixed here, before anything is built.** The merge folds
threads back into the *existing* stage 02/03/04 artifacts, and the driver's
validators for those stages do not change. If a thread artifact cannot be
mechanically turned into `sources/url-list.json`, `sources/chunk-<n>.json`, and
`sources/registry.json` in their current shapes, threading stops being a
refactor and becomes a rewrite. Every field below exists to make that fold
possible without touching a validator.

---

## Layout

```
<package_id>/
  threads/
    index.json                 # every dispatch, including the ones that failed
    <tid>/
      brief.json               # what the director handed this worker, and nothing else
      dossier.md               # facts, long, cited inline — the lossless handoff
      reflections.md           # why each search happened, in order
      sources.json             # what this worker actually fetched
      claims.json              # what it asserts, each with a verbatim quote
      chunks/
        chunk-<n>.json         # fetched page text, already in the stage 03 shape
```

`<tid>` is `t` followed by two or more digits (`t01`, `t02`, … `t14`). It is a
path component: no `/`, no `\`, never `.` or `..`.

---

## The two agents

Added by change-drt-003.

| Agent | Tools | Duty |
|---|---|---|
| `agents/research-director.md` | `Read, Grep, Glob, Write` | cycle loop, writes briefs, reads dossiers, maintains the coverage/gap ledger in `plan.md`. **Never searches or fetches.** |
| `agents/research-worker.md` | `WebSearch, WebFetch, Read, Write` (only under `threads/<tid>/`) | one brief, fresh context, reflection before every search after the first, writes the dossier |

The director's restriction is not decoration. A director with search tools starts
answering the question instead of decomposing it; once it has read three pages it
has a hypothesis, and every later brief is shaped by whichever page loaded first.
Removing the tools forces the alternative — writing a brief self-contained enough
that someone else can look for you.

Because `tools:` frontmatter is advisory on some harnesses, the rule is enforced
mechanically: `merge-threads.sh` refuses any dossier source absent from that
thread's `sources.json`, so a source the director fetched itself fails the run.
The allowlist is the intent; the merge is the enforcement.

## `brief.json` — the only thing a worker receives

A worker gets its brief and nothing else: no plan, no sibling dossier, no chat
history. That isolation is what keeps each thread's context small enough to stay
accurate, and it is asserted by `tests/dispatch-smoke.sh`.

```json
{
  "thread_id": "t03",
  "task": "Investigate reported production failure modes of Qdrant above 10M vectors, including memory pressure, snapshot and restore, and cluster rebalancing, from 2025 onward.",
  "must_cover": ["failure symptoms", "root causes", "mitigations", "affected versions"],
  "avoid": ["pricing", "benchmarks already covered by t01"],
  "recency_months": 18,
  "excluded_domains": ["reddit.com"],
  "budget": { "cycles": 8, "minutes": 12, "max_sources": 12 }
}
```

| Field | Required | Meaning |
|---|---|---|
| `thread_id` | yes | Matches the directory name |
| `task` | yes | One or two sentences, self-contained. A worker that needs the plan to understand its task has been briefed badly |
| `must_cover` | no | Checklist the worker answers against before stopping |
| `avoid` | no | Ground a sibling already covered. **This is the one thing a two-level topology can do that a fully isolated one cannot**: the director has read the other dossiers, so it can say "not this". It does not break isolation, because the worker still receives only its brief |
| `recency_months`, `excluded_domains` | no | Search constraints |
| `budget` | yes | Bounds. The scheduler enforces `minutes`; the worker honours `cycles` and `max_sources` |

---

## `dossier.md` — facts, deliberately long

The dossier is the **lossless handoff**. Everything downstream reads it, so
compression here is information lost forever.

Rules a worker follows when writing one:

- **Facts only.** No title, no sections, no conclusions, no analysis.
- **As long as necessary.** Several pages is expected and correct. The cost of
  tokens is lower than the cost of a fact that never reached synthesis.
- **Context with every fact**, so nothing can be misattributed later.
- **Flag anything that looks untrustworthy or contradicts another statement.**
  Stage 06 resolves contradictions; the dossier only has to surface them.
- **Cite every fact inline** as `[src:<sha8>]`, where `<sha8>` is the `id` of an
  entry in this thread's own `sources.json`.
- Remove obvious duplicates and irrelevancies. Nothing else.

Default cap: **6000 tokens**. Onyx caps its equivalent at 10000
(`research_agent.py:93`) with an inline note that around 5000 performs better in
practice; 6000 sits between the measured sweet spot and the hard ceiling.

A citation naming a source absent from this thread's `sources.json` is a
**CRITICAL** merge failure (`drt-004`). That rule is what enforces the
director's no-search constraint on harnesses that ignore `tools:` frontmatter.

---

## `reflections.md` — the reasoning, made durable

Append-only. One entry before every search after the first.

```markdown
## 2026-09-08T12:04:11Z — before search 2
Found: three vendor posts on memory pressure, all pre-2025.
Missing: post-2025 incident reports, and anything on snapshot restore.
Next query: "qdrant snapshot restore failure 2026" — targets the gap directly.
```

Onyx's equivalent is a `think_tool` whose entire response is
`"Acknowledged, please continue."` (`dr_mock_tools.py:112`) — it exists only to
force a reasoning turn, and is then discarded. Writing the same reflection to a
file costs nothing extra and yields both the reasoning-forcing effect **and** an
audit trail the provenance sidecar can cite.

---

## `sources.json` — what this worker fetched

```json
[
  {
    "id": "a1b2c3d4",
    "url": "https://qdrant.tech/documentation/guides/administration/",
    "title": "Administration — Qdrant",
    "fetched_at": "2026-09-08T12:03:58Z",
    "chunk_ids": ["chunk-1", "chunk-2"]
  }
]
```

`id` is the first 8 hex characters of `sha256(canonical_url)`. Canonicalisation
(strip tracking parameters, normalise the trailing slash) uses the **same shared
helper** as stage 02's dedupe, so two copies of the rule cannot drift apart.

The merge unions these by canonical URL into `sources/registry.json` and
`sources/url-list.json` in their existing shapes.

---

## `claims.json` — assertions, each with a quote

```json
[
  {
    "id": "claim-9f86d081884c7d65",
    "text": "Qdrant requires roughly 1.5x the raw vector size in RAM when HNSW is fully in memory.",
    "source_id": "a1b2c3d4",
    "quote": "plan for approximately 1.5x the raw vector footprint in RAM",
    "chunk_id": "chunk-2",
    "thread_id": "t03"
  }
]
```

| Field | Rule |
|---|---|
| `id` | `claim-` + `sha256("<package_id>:<normalised text>")` truncated to 16 hex. **The rule already in use** by `build-graph.sh` and `detect-contradictions.sh` (change-rah-010); the merge reuses it rather than inventing a second scheme |
| `quote` | **Verbatim**, at most 40 words, appearing in the referenced chunk |
| `source_id` | Must exist in this thread's `sources.json` |

**Why `quote` is required.** Stage 05's rule is "verify meaning, not topic
overlap" and "read before you label". Without a verbatim span that rule can only
be applied by judgement. With one it becomes mechanical: the fetched chunk either
contains the quote or the claim is labelled `unverified`. This is the field that
turns a prose instruction into a check.

---

## `threads/index.json` — a row for every dispatch

```json
{
  "schema_version": "1.0.0",
  "package_id": "vector-db-failure-modes-20260908-a1b2",
  "director_cycles": 2,
  "budget_hit": false,
  "threads": [
    {
      "thread_id": "t01",
      "task": "Investigate production failure modes of Qdrant above 10M vectors",
      "status": "complete",
      "cycle": 1,
      "started_at": "2026-09-08T12:01:00Z",
      "ended_at": "2026-09-08T12:09:22Z",
      "sources_count": 9,
      "claims_count": 24,
      "reason": null
    },
    {
      "thread_id": "t02",
      "task": "Compare snapshot and restore behaviour across versions",
      "status": "timeout",
      "cycle": 1,
      "started_at": "2026-09-08T12:01:00Z",
      "ended_at": "2026-09-08T12:13:00Z",
      "sources_count": 3,
      "claims_count": 5,
      "reason": "exceeded thread budget of 12 minutes; partial dossier preserved"
    }
  ]
}
```

`status` is one of `complete`, `partial`, `failed`, `timeout`.

**Every dispatch gets a row, including the ones that went wrong.** A thread that
dies still produces a row with `status: failed` and a reason, so the ledger has
no silent holes. This mirrors the invariant Onyx maintains at the protocol level
— every `tool_use` gets a response, even a synthetic failure one — applied here
to files instead of messages. A director reading the ledger can then re-dispatch
a `partial` or `failed` thread knowingly, rather than never learning it existed.

---

## What a validator checks

`tests/thread-contracts.sh` asserts, against fixture threads:

1. `brief.json` and `threads/index.json` validate against their schemas.
2. Every `claims.json` entry has a non-empty `quote` of at most 40 words.
3. Every claim's `source_id` exists in the same thread's `sources.json`.
4. Every dossier citation `[src:<sha8>]` resolves to a source in the same thread.
5. Every `threads/index.json` row carries a status from the enum, and a `reason`
   whenever the status is not `complete`.
