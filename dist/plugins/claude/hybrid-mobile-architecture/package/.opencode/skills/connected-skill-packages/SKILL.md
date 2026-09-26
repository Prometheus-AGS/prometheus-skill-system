---
name: connected-skill-packages
description: Install, upgrade, validate, or remove a git-based agent skill package marketplace consumed by the Prometheus Companion. Use when extending the substrate with third-party skill packages, pairing a new device, shipping a package that must be installable, or recovering from a broken marketplace. Triggers on skill package, marketplace, marketplace.json, plugin install, plugin upgrade, plugin remove, connected skill, install contract, hybrid-mobile-architecture, knowme-builder, plugin marketplace add, skill bundle, additional skills.
---
<!-- TJ-ARCH-MOB-001 compliant -->

# Connected Skill Packages

This package is a **producer**. The Prometheus Companion is the **consumer**.
This skill is the contract between them. Satisfy the contract here; the
Companion's install, upgrade, validate, and remove operations then work
without per-package special cases.

## What a connected skill package is

- A git repository the consumer clones into the configured
  `<skill-package-root>/<id>/`
- A `marketplace.json` at the repo root declaring plugins
- A `plugin.json` at the repo root declaring the package
- A skill tree the harness can register
- A compatibility declaration against the consumer's substrate version

## The install contract (4 conditions)

A package is installable only when all four hold. `scripts/verify-skill-manifest.mjs`
checks 1-3 mechanically and asserts the structural precondition of 4.

1. **Valid `marketplace.json`** at the repo root, parseable, with `name`,
   `version`, and a `plugins[]` array.
2. **Valid `plugin.json`** at the repo root, parseable, with `name` and
   `version`. Its `version` must equal the marketplace version.
3. **Every declared skill resolves.** For each entry in the canonical skill
   registry, `skills/<name>/SKILL.md` exists and its frontmatter `name`
   equals its directory name.
4. **Idempotent install.** Installing writes only inside the install path.
   Re-running `git clone` into a fresh path, or `git fetch` plus
   `git reset --hard <sha>` into an existing one, converges to the same
   tree. No global state outside the install path.

Condition 4 is a behavior property. Its failure mode is silent: the install
appears to succeed but the harness cannot find the skills.

## The canonical registry

In this repo the registry is `builder.manifest.json` → `skills[]`, not
`plugin.json`. Every generated artifact derives from it:

```
builder.manifest.json  →  skills/<name>/agents/openai.yaml   (generate-skill-metadata.mjs)
                       →  evals/builder-skills.jsonl          (generate-skill-evals.mjs)
                       →  6 harness mirrors + project templates (sync-harness-skills.mjs)
```

Adding a skill means adding it to the registry and to
`templates/activation-manifest.json`, then regenerating. Hand-editing a
generated artifact produces drift that `git diff --exit-code` rejects.

## The four operations

| Operation | Consumer behavior | Producer obligation |
|---|---|---|
| Install | Refuse if the path exists; else `git clone --branch <ref> --depth 1`, verify the contract, register with the harness | Contract holds at the cloned ref |
| Upgrade | `git fetch origin <ref>`, `git reset --hard <sha>`, re-verify, re-register | Tag releases; the consumer pins tags, not branch HEAD |
| Validate | Re-run the contract check plus schema, anti-pattern, and compatibility checks | `scripts/verify-skill-manifest.mjs` exits 0 on a clean tree |
| Remove | Unregister from the harness, remove the install path, mark the record disabled rather than deleting it | Nothing outside the install path to clean up |

Branch HEAD is unstable. Tag releases and let the consumer default to the
latest tag.

## Harness-aware registration

The consumer reads a harness selector and registers against the matching
mirror. This repo already mirrors every registered skill into `.agents/`,
`.claude/`, `.codex/`, `.kimi/`, `.kimi-code/`, `.opencode/`, and
`templates/project-skills/`. Keep the mirrors generated, never hand-written:
`node scripts/sync-harness-skills.mjs` writes them and `--check` fails on drift.

## Anti-patterns

- Auto-pulling a package when a new commit lands. Upgrades are operator-initiated.
- Installing without running the validation gate first.
- Installing anywhere other than the configured `<skill-package-root>/<id>/`.
- Shipping a package with no manifest verifier — the consumer refuses it.
- Hiding the parsed manifest from the operator.
- Running the harness install with elevated privileges. It is user-level.

## Verification

```bash
node scripts/verify-skill-manifest.mjs          # the 4-condition contract
node scripts/check-skill-contracts.mjs         # frontmatter, resources, evals
node scripts/sync-harness-skills.mjs --check    # mirror drift
```

All three must exit 0 before the package is publishable.
