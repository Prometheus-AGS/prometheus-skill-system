# Goals — phase-release-documentation-publication

## Fixed scope (authorized by operator, 2026-10-18)

Publish the qualified candidate (commits `ebc8f687` + `9a7c4bbf`, branch
`codex/kbd-phase-readiness`) as the next versioned release with complete
documentation:

1. **Documentation** — README, KBD docs, CHANGELOG, Docusaurus site: planning
   contract, readiness gate, execution flow, evidence-bound closure,
   missing-accounting BLOCKED behavior, budgets, installation, practical
   example. Must disclose shell-layer enforcement, bypassable paths,
   self-reported accounting, and installed-demonstration limits. No implication
   of automatic enforcement in every harness.
2. **Versioning** — inspect tags/metadata/conventions; select next unused
   version. Keep `docs/releases/1.12.2-candidate-qualification.md` intact;
   link it from the new release notes.
3. **Release checks** — docs/site build, required release-policy checks,
   regenerate distribution if needed. Confirm the release includes the
   qualified evaluator and gate. No unrelated behavioral suite reruns.
4. **Delivery** — scoped commits only; push branch; open PR; resolve only
   in-scope CI failures. Report billing/permissions/infrastructure blockers
   exactly. No spending, no bypassing required checks.
5. **Publication** — merge only when required checks/approvals pass; publish
   version + Docusaurus via existing workflow; verify tag/artifact identity,
   docs URL, install instructions. Stop for approval if new credentials,
   permissions, or paid resources are needed.
6. **Issue reconciliation** — close only issues whose acceptance criteria are
   met; keep #184/#185 open with release link and remaining scope. No silent
   requirement discard or redefinition.

## Exit criteria (done means)

- New version published; documentation accessible; release points to the
  intended qualified implementation.
- Final report: PR link, release link, documentation URL, install command,
  known limitations.
- If any delivery step is blocked: report "candidate qualified; publication
  blocked" with the exact unfinished action.

## Exclusions

No force-push, branch-protection bypass, published-version overwrite, unrelated
working-tree changes, successor hardening, mini-pack work, phase-exit
integration, or additional features.

## Budget (new, explicitly granted; separate from previous phase accounting)

60 minutes / 40 tool calls / 1 repair attempt / no-progress limit 2.
Historical consumption preserved, not reset.
