# change-ldd-15-readme-content-renewal

Title: Rewrite full and mini READMEs and audit all owned reader-facing content
Phase: phase-learning-deploy-and-debt
Backend: native-kbd
Repository: full+mini
Gaps: G17
scope:
  - full:README.md
  - mini:README.md
  - full:owned README.md files
  - mini:owned README.md files
  - phase:content-inventory.json
  - phase:content-disposition.md

## Requirements

1. Enumerate all tracked owned README/Markdown/MDX customer-facing content, including guides, skills references, troubleshooting and site sources. Classify canonical owner, source/generated/vendor/history, currentness and keep/rewrite/remove/archive/redirect decision. ALL means every inventoried owned surface receives a disposition, not a representative sample.

2. Rewrite both root READMEs as current product entry points: intended users, full/mini differences, supported platforms, install/update/uninstall, first useful workflow, teams/model choices, service topology, docs links, version/support policy and local contribution gates. Derive counts/versions from final manifests. Remove stale prose such as Full 1.11.0, original port analysis and abandoned plans from current onboarding.

3. Historical material with value moves to clearly labeled archives/decision records outside normal onboarding/navigation; obsolete promises and broken commands are removed or corrected. Preserve license/attribution and upstream-owned files; edit generated content only at its source. Every removal of a public route gets a deliberate redirect/replacement decision.

4. Coordinate all other content changes with 16 and 17 so inventories reconcile without multiple conflicting copies. Completion requires no unreviewed owned item and no factual claim without current source or local evidence.

## Binding constraints

Source work uses isolated worktrees under ../worktrees with declared base/provenance; protect deploy-main/deploy/main and other agents' changes. All tests, builds and reviewers wait for completed production and run only locally under change 12. Use scratch HOME/CODEX_HOME/CORTEX_DATA_DIR/plugin/learning roots; never test against live :23001. One Cargo/rustc process machine-wide; pgrep first. User merges every PR and explicitly approves versions.toml edits, tags and real Codex/Claude changes. No approval is inferred from planning. See the phase plan for ordering, exact task model assignments and final gates.

## Plan assignment and ordering authority

`.kbd-orchestrator/phases/phase-learning-deploy-and-debt/plan.md` is the current ordering and Task model assignments authority. Match the full phase path `phase-learning-deploy-and-debt`, change ID `change-ldd-15-readme-content-renewal` and backend task ID in tasks.json. Earlier dependency prose is superseded by this Plan revision.
