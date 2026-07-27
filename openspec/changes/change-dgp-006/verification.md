### change-dgp-006 — Skills-catalog generator
`/opsx:new change-dgp-006`
`site/scripts/generate-skills-catalog.mjs`: walk `skills/*/*/SKILL.md`
frontmatter (name/description/tags/category — same fields
`scripts/validate-skills.js` parses), emit MDX catalog pages (index by
category + per-skill entries) into a generated docs instance; wire into
`site` build script (`prebuild`); **extend the search-local route-base
configuration to include the catalog instance**. Excludes
`skills/imported/` submodules.
Acceptance: catalog lists every non-imported skill (count matches validator's
count); regenerating is idempotent; search index covers the catalog route
base and returns a known skill name; build green.
Agent: build. | library: cand-006


## Evidence (implemented 2026-07-27)
- 140 skills generated = validator count (find skills -name SKILL.md -not -path imported = 140).
- Idempotent: consecutive runs identical output. prebuild/prestart wired.
- search-index.json contains "adversarial-review" (catalog route base indexed).
- build exit 0; /docs/catalog/ + per-category routes in build output.

## Vet fixes applied (round 1, PASS)
- CRLF/BOM-tolerant frontmatter regex + WARN on missing/unparseable frontmatter.
- lstatSync walk: symlinks skipped, stat errors logged and skipped (symlink-farm repo).
- localeCompare pinned to 'en'. package.json trailing newline restored.
- Count contract documented: "every non-imported SKILL.md file" (validator walks the same set; 140=140 by construction, not coincidence).
- metadata.category unread by design — grouping is path-based (skills/<category>/); noted.
