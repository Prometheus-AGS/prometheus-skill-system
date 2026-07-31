# Two defects found while creating this child phase — 2026-07-31

## 1. `kbd-new-child.sh` fails with an unbound variable

```
kbd-new-child.sh: line 153: child_label: unbound variable
```

Deterministic, not environmental. The runtime-authority branch uses
`${child_label}` at **line 156**:

```sh
--exact-next-work "/kbd-assess ${child_label}"
```

but the variable is not assigned until **line 234**:

```sh
child_label="$(kbd_node_chain "${cur_tokens[@]}" "$name")"
```

Under `set -u` the earlier reference aborts. The child phase directory,
`goals.md`, `progress.json`, and the waypoint flip were therefore created by
hand.

**Not patched here.** It is an installed skill under `~/.claude/skills/`, where
an edit is destroyed by the next install and invisible to git — the same class
of mistake as editing a plugin cache.

## 2. A stale runtime store silently overwrites the waypoint

`current-waypoint.json` carries `generatedBy: "kbd-runtime"`. Per
`runtime-authority.sh`, that single field makes a `prometheus kbd` runtime store
authoritative for the file:

```sh
[ "$(jq -r '.generatedBy // empty' "$waypoint")" = "kbd-runtime" ]
```

That store is pinned to a **stale run**:

```
Run:       docusaurus-github-pages-site-20260729T012334Z  revision 475
Lifecycle: Completed
Lease:     expired 2026-07-29 01:35:56 UTC
```

So the waypoint reports `phase: adversarial-review-for-creation` and
`170/229` — a phase closed two phases ago and a counter belonging to a different
run. Hand edits to those fields revert.

### Why it matters

Every KBD skill instructs an agent to read `current-waypoint.json` **first** to
establish position. Two of its fields are projections of a completed run with an
expired lease, and nothing reports the disagreement — the file looks
authoritative precisely because it names itself so.

### What is still trustworthy

- `phases/<name>/progress.json` — correct (`uar-host-execution` is 7/16)
- Waypoint fields the runtime does **not** project — `childPointer`, `next`,
  `parentResumeCommand`, `status` all persisted correctly

### Related, same class

Two defects already recorded in the parent phase:

- `kbd-reflect` never writes `.phase`, so it names a phase two transitions back
- `kbd-next-phase` writes a **self-referential** `next` while
  `exactNextCommand` in the same file is correct

**Three of four are in installed skills.** The fourth — the stale runtime store —
is data, not code, and may simply need the run closing out or the
`generatedBy` marker dropped for this repo. That is a decision for whoever owns
the runtime, not something to guess at here.
