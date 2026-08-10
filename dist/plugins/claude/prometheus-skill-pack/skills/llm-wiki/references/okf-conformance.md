# OKF v0.1 Producer Checklist

Distilled from OKF v0.1 §9 (Conformance) plus the sections it references.
Run through this before calling any wiki write "done" — whether the page was
emitted by pk or edited by hand. The spec itself is
`shared/references/okf-v0.1.md`; section numbers below refer to it.

## Hard requirements (a bundle is non-conformant without these)

- [ ] **Frontmatter parses** (§9.1) — every non-reserved `.md` file in the
      tree opens with a `---`-delimited YAML block that a YAML parser
      accepts. No frontmatter, unclosed delimiters, or tab-indented YAML
      all fail this.
- [ ] **`type` present and non-empty** (§9.2) — every frontmatter block has
      a `type` whose value is a non-empty string. `type:` (empty), `type: ""`,
      and a missing key all fail.
- [ ] **Reserved files keep their shape when present** (§9.3):
      - `index.md` (§6): no frontmatter (bundle root MAY carry exactly
        `okf_version: "0.1"`, §11); body is heading-grouped bullet lists of
        `[title](url) - description` entries.
      - `log.md` (§7): date-grouped entries, headings in ISO `YYYY-MM-DD`
        form, newest first.
      - Neither is ever a concept page (§3.1).

## Producer SHOULDs (this wiki treats them as required practice)

- [ ] `title` set, or the filename is a presentable fallback (§4.1).
- [ ] `description` is one sentence — index entries and search snippets
      reuse it verbatim (§4.1, §6).
- [ ] `tags` is a YAML list of short strings, not a comma-joined scalar (§4.1).
- [ ] `timestamp` is ISO 8601 and reflects the last meaningful change (§4.1).
- [ ] `resource` present when the page describes a physical asset; absent
      for abstract concepts (§4.1).
- [ ] Cross-links use the bundle-relative form `/(path)/(page).md` (§5.1).
- [ ] Body prefers structural markdown; conventional headings (`# Schema`,
      `# Examples`, `# Citations`) used when applicable (§4.2).
- [ ] External claims carry a numbered `# Citations` section (§8).
- [ ] Concept IDs are filesystem-safe: no `..` segments, no leading `/`,
      no empty path segments (§2 — path-based IDs must join safely onto the
      bundle root).

## Round-trip obligations (when editing existing pages)

- [ ] Unknown frontmatter keys preserved, never stripped (§4.1 Extensions).
- [ ] Unknown `type` values left as-is — do not "normalize" to a known type
      (§4.1, §9).
- [ ] Broken links left intact unless the fix is the task at hand — they
      legally mark not-yet-written knowledge (§5.3).

## What conformance does NOT require (do not "fix" these)

Per §9, consumers must tolerate all of the following, so their presence is
never a defect to repair mechanically:

- Missing optional frontmatter fields.
- Unknown `type` values.
- Unknown additional frontmatter keys.
- Broken cross-links.
- Missing `index.md` files.

Lint may still *flag* these as health findings (an orphan page, a repeated
broken link that wants a page) — but as suggestions to the user, never as
conformance failures.

## Quick verification

```sh
pk lint --kb-dir <kb>        # mechanical checks: frontmatter, type, reserved files
grep -L '^---' wiki/**/*.md  # any page missing a frontmatter open delimiter
grep "^## " wiki/log.md | head -3   # log recency + heading shape at a glance
```
