# Visual baseline refresh

Refresh a baseline when a completed production change intentionally changes the
rendered UI. Baseline generation records the new expectation; it does not prove
that the change is correct. Review the before/after images and exercise the real
UI workflow against its production collaborators before accepting the change.

## Local workflow

1. Finish the production implementation on the change's isolated worktree and
   branch. Read the existing protected scenarios as requirements. Do not change
   expectations simply to make an incomplete or broken implementation pass.
2. Record the intended visual differences and select the repository's actual
   integration config, browser project and affected test files. Use its pinned
   local Playwright installation and isolated test resources.
3. At the completed implementation boundary, generate the intended baselines,
   inspect every changed image, and run the same integration scenarios with
   snapshot updates disabled. Include meaningful navigation and interaction
   assertions; images alone do not establish functional acceptance.
4. Record the source identity, local commands, results, environment and changed
   asset paths/hashes in the review artifact. Preserve paths containing spaces.
   The refresh operation must not automatically commit, push or merge.
5. After the applicable local integration gates pass, prepare the candidate
   commit. Run final protected-test integrity certification from committed Git
   state. Push and open the review only after all required local gates pass.
   The repository owner reviews and merges the PR.

The following commands illustrate the two distinct operations. Substitute the
actual repository config, project and scenario paths; these are placeholders,
not a gate configured by this skill. Run them only after implementation is
complete and the production UI and collaborating services are available.

```bash
npx playwright test path/to/production-ui.spec.ts --config=path/to/playwright.config.ts --project=chromium --update-snapshots=changed
npx playwright test path/to/production-ui.spec.ts --config=path/to/playwright.config.ts --project=chromium --update-snapshots=none
```

Use the flags supported by the repository's pinned Playwright version. Scope
updates to the intended scenarios; avoid regenerating unrelated assets.
[Playwright snapshot update documentation](https://playwright.dev/docs/test-snapshots#updating-screenshots)
describes baseline generation, not authorization to accept a visual change.

## Protected scenarios and approval

For a change to paths covered by the repository's protected-test policy, obtain
the canonical SSH-signed approval manifest under the `prometheus-test-change`
namespace. The final local `scripts/verify-protected-tests.mjs` certification
compares the committed candidate with its committed base. Inspect the actual
protection rules: do not assume that every PNG is protected, or that unprotected
images exempt accompanying protected scenario changes from approval.

A GitHub label is a review annotation unless independently configured enforcement
says otherwise. It does not replace signed protected-test approval, local
integration evidence, human visual review, or the owner's merge. Hosted CI
results are not validation evidence for Prometheus repositories.

## Review evidence

Record why each expectation changed and show the affected screens. An image
checksum identifies bytes; it cannot establish that those bytes are desirable.
Keep unexpected differences visible, fix production defects in coherent batches,
and rerun the affected final integration gate. Do not delete a failing protected
scenario or blindly update every baseline to obtain a passing result.
