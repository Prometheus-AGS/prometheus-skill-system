# Nested phases — arbitrary-depth child loops (v3 `path[]`)

> Extracted from the orchestrator SKILL.md "Nested phases" section. The depth
> model, node-dir resolution, navigation verbs, and the selected-vs-entered
> invariant.

## The `path[]` position chain

The waypoint carries `path: string[]` — the canonical position chain supporting
*any* nesting depth (phase → child → grandchild → …). `path[0]` is the
top-level phase; each later element is a nested child. The on-disk node dir
interleaves `children/`:

```
path = [p0]            → phases/p0
path = [p0, c1]        → phases/p0/children/c1
path = [p0, c1, g2]    → phases/p0/children/c1/children/g2
```

`path[]` is **additive and lazy**: when absent it is synthesized from `[phase]`
or `[phase, childPointer]`, and `parentPhase`/`childPointer` are maintained as
derived (deepest-frame) fields for one release. Resolve a node dir with
`kbd_node_dir` / `kbd_current_node_dir`, and render a breadcrumb with
`kbd_node_chain` (`shared/lib/waypoint.sh`). `maxChildDepth` in `project.json`
(default 4) is the nesting sanity rail.

## Child navigation verbs

| Verb | Effect |
|---|---|
| `/kbd-new-child <name>` | create a child under the active node |
| `/kbd-next-child [<name>]` | *select* a sibling for traversal (moves `childPointer`) |
| `/kbd-child-exit --enter` | *descend* into the selected child (so new children nest under it) |
| `/kbd-child-exit` | close the child: write `handoff-out.md`, roll progress up the ancestor chain, pop `path[]`, return to the parent |

On spawn, each child also gets a `handoff-in.md` (the parent→child brief) and a
`scope.json` (its context-isolation contract, advisorily enforced by
`check-child-scope.sh`). On exit, `shared/lib/rollup.sh` recomputes a
`children{}` aggregate block in each ancestor's `progress.json`.

## Selected-vs-entered invariant (READ before manipulating `path[]`)

A child can be *selected* (chosen for traversal) or *entered* (the active
node). The distinguisher:

- **`path[]`'s trailing token EQUALS `childPointer`** → *selected but not
  entered*. The active node is still the child's parent. A `/kbd-new-child`
  here adds a **sibling** (the pointer token is stripped to find the parent).
- **`childPointer` cleared (or differs from `path[]`'s tail)** → *entered*. The
  active node IS the deepest `path[]` node. A `/kbd-new-child` here **nests**
  under it.

Therefore **descent = set `path[]` to the child chain AND clear
`childPointer`** — exactly what `/kbd-child-exit --enter` does. External tools
that write `path[]` directly must follow this rule, or `/kbd-new-child` will
nest or sibling surprisingly.

## Backward compatibility

At the top level, `childPhases`/`childPointer` are still maintained on the
waypoint (v2 contract) so existing tools and tests keep working. The
`__schemaVersion: "3"` in the template is documentation only — no skill reads
it at runtime.
