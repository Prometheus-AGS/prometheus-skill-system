# Authoring a plugin for this marketplace

Covers the two ways a plugin can enter `.claude-plugin/marketplace.json`, when
to use each, and the manifest contract. For skill *descriptions* — the thing
that decides whether your skill is ever selected — see
[`skill-authoring-guide.md`](skill-authoring-guide.md).

## Two source forms

### 1. Local path (the 11 first-party plugins)

```json
{
  "name": "prometheus-devops-skills",
  "version": "1.5.0",
  "source": "./skills/devops"
}
```

The plugin lives in this repo. Its `.claude-plugin/plugin.json` lists the skill
directories it contributes. Use this when the code belongs to the pack and
should version with it.

### 2. External git repo + `strict: false` (the skill-bundle pattern)

This is the "extension without bloating the core" path, and it is the answer to
"how do I ship a skill without a PR against the pack's source tree?"

```json
{
  "category": "productivity",
  "name": "artifact-refiner",
  "skills": "./skills",
  "source": {
    "repo": "GQAdonis/artifact-refiner-skill",
    "sha": "dbc49d78748005177626b559a931d16864f1c9a4",
    "source": "github"
  },
  "strict": false
}
```

**What `strict: false` means.** It tells the host not to require a full plugin
manifest in the referenced repo. The repo may contain nothing but `SKILL.md`
files. Without it, a repo lacking `.claude-plugin/plugin.json` fails to load.
`strict: false` is what makes "a git repo of bare skills" a valid plugin.

**Why both current examples pin a `sha`.** A pin makes the install
reproducible — the bundle identity that the hook runtime verifies is computed
over pinned content. The cost is that upstream fixes do not arrive until the sha
is bumped here by hand; there is no `plugins update` command yet (review W8.7).
Pin anyway. An unpinned external plugin means the pack's installed bytes can
change without a commit in this repo.

**Note the correlation:** the only two `strict: false` entries are the only two
sourced from GitHub, and the only two with **no `version` field**. The version
of an external bundle is its `sha`; a `version` string on top of that would be a
second, un-enforced source of truth.

## The manifest contract

Every `plugin.json` is validated against
[`schemas/plugin.schema.json`](../schemas/plugin.schema.json):

```bash
npm run validate:plugins
```

Required: `name` (lowercase kebab-case), `description`, `version` (semver).

The schema is **descriptive, not aspirational** — it was written by running it
against all 27 manifests in the tree and widening it until every genuinely
working form passed. So several fields accept more than one shape:

| Field | Accepted forms |
|---|---|
| `skills` | array of relative paths, **or** a single string path to a `SKILL.md` |
| `mcpServers` | inline object keyed by server name, **or** a string path to a `.mcp.json` |
| `compatibility` | free-text string, **or** an object with `platforms`/`languages`/`frameworks` |
| `author` | string, **or** an object with `name`/`email`/`url` |

If a manifest that demonstrably loads fails validation, widen the schema —
do not "fix" the manifest to satisfy it.

`additionalProperties` is deliberately `true`: a plugin should not fail to load
because it declares a key a newer host understands.

### One trap worth naming

Do **not** set `"hooks": "./hooks/hooks.json"`. Claude Code already auto-loads
`hooks/hooks.json` from the plugin root, and declaring the same default path
again fails plugin load with *"Duplicate hooks file detected."* Only set `hooks`
for a genuinely non-default location. See `CLAUDE.md` → *Canonical hooks path*.

## Versioning

Plugin versions in this repo have drifted from the pack version — the pack is at
`1.7.0` while several plugins remain at `1.5.x`. There is no cascade: bumping the
pack does not bump its plugins. When releasing, bump `package.json`,
`.claude-plugin/plugin.json` entries, `site/package.json`, and each plugin's own
`plugin.json` together, per the publishing checklist in `CLAUDE.md`.

## Before you publish

```bash
npm run validate:plugins       # manifest shape
npm run validate:strict        # skill frontmatter, incl. description quality
npm run build                  # distribution artifacts
npm run check:distribution     # generated artifacts are current
```

All of these run locally. Per `CLAUDE.md`, hosted CI is never validation
evidence for this repo.
