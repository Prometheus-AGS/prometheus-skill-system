# Full 1.18 documentation inventory

The [tab-separated inventory](full-1.18-documentation-inventory.tsv) records 419 documents at the merge of Full 1.18.0. Each row identifies the path, publication channel, title, last first-parent change on main, number of distinct local link targets found in that document, unresolved literal targets and review disposition. Use the [release history](../guide/27-release-history.md) for user-facing changes and migration instructions. [Reviewed source revision](https://github.com/Prometheus-AGS/prometheus-skill-system/commit/30e63d55)

## Scope and method

The inventory includes all tracked Markdown at the repository root, Markdown and HTML in `docs/` and `site/`, and every tracked `README.md` at this revision, including packaged copies. This covers 104 authored or managed Pages topics, 44 guide/learning entries, four HTML supplements and 44 projected READMEs among the 419 rows. Submodule contents belong to the pinned upstream projects. Individual skill instructions are indexed separately by the [skill catalog](../../SKILLS.md). Docusaurus builds its browsable catalog from the source skill frontmatter. Historical investigations and internal process READMEs retain the date and scope of their results.

To reproduce this baseline from a checkout containing the merged revision, run:

```bash
python3 scripts/documentation-inventory.py 30e63d55712e0285acdf2c5cf2375a72b7dcb37c > /tmp/full-1.18-documentation-inventory.tsv
diff -u docs/reports/full-1.18-documentation-inventory.tsv /tmp/full-1.18-documentation-inventory.tsv
```

The scan resolved 494 document-relative link targets summed across documents, including extensionless links to Markdown pages. Thirteen documents contained unresolved literal paths. This count is limited to paths present in the Git tree. It excludes fragments, network links, absolute Docusaurus routes and executable behavior. A row with no missing links can still describe a retired feature. Full’s separate local documentation gate builds the site and checks public content, OpenAPI examples, Mermaid diagrams and declared contracts.

## Findings and ownership

| Observation at the reviewed revision | Disposition in the documentation refresh |
| --- | --- |
| Sixteen Sovereign Sync pages directed users to an unpublished Companion checkout. | Point readers to the public Companion project, its installation and control API. Explain addressed pushes and observed restart behavior in the signed-push topic. |
| Eight generated plugin READMEs referred to paths above the installed package. | Use published source or site links in the three source READMEs, then regenerate the Claude and Codex packages. |
| The 1.13 release README and a historical BDD audit linked to existing files from the wrong directory. | Correct relative navigation while preserving the historical results. The remaining three unresolved documents show example paths in a skill template, research excerpt or historical implementation plan. |
| The generated reference and authored operations guide described only docs synchronization and Pages deployment. | Correct the reference generator and authored guide, then regenerate the two managed blocks. The binary and container publisher runs on eligible releases. |
| A published maintenance table gave Node 22.12 as the root package floor. | Align installation prerequisites with the package engine and distinguish the prerequisites of individual components. |
| Git tags, source-version changes, GitHub Releases and native downloads had no common navigation. | Add an era-by-era upgrade guide, repair missing tagged-source changelog entries, and link the user-facing workflows from the release and operations guides. |

The inventory remains fixed to the reviewed merge so the historical findings stay reproducible. To inspect a later revision’s paths and link observations, pass its full commit to the same script. The disposition column documents the Full 1.18 review.
