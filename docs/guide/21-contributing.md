# 21 · Contributing

Use the maintained [contributor workflow](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/CONTRIBUTING.md) for setup, skill structure, local gates and pull requests. The root package requires Node.js 20.19.0 or later; team helpers require Node 22 or later. Pin tools and imported sources deliberately rather than advancing all submodules to arbitrary branch heads.

A new skill needs a directory whose name matches its frontmatter, concise instructions, declared prerequisites and accurate helper examples. Strict validation is required at the final boundary. Imported skills and vendored tools have independent owners and release histories; fix their source under the proper ownership, then update the selected pointer with approval where required.

## Complete production before evidence

Finish every planned production change in the active phase before authoring or running tests, formatters, generators or reviewers. A task boundary does not authorize an early gate. Use static inspection during implementation, then run real integration through the production entry point and collaborators on the local machine. Hosted test workflows, mock-only checks and legacy unit totals do not certify delivery.

Protected BDD scenarios require the owner's SSH-signed canonical approval for intentional changes. Agent tools remain unrestricted; integrity is checked from committed Git state at final local certification. Do not delete a failure or rewrite a protected scenario to manufacture a pass.

Generated payloads and catalogs follow [generated ownership](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/generated-output-ownership.md). Reconcile authored sources first, regenerate at the final phase boundary, and verify the actual packaged and installed paths. Source, generation, installed invocation, certification and publication are separate evidence dimensions.

## Submission and publication

Record exact local integration commands and results, selected source identities and outstanding limits. Preserve runtime data, credentials and scratch material outside the commit. Follow local review receipt and owner merge requirements. Passing gates do not independently authorize merging, version changes or publishing.

*Previous: [20 · Updating](20-updating.md) · Next: [22 · Advantages and costs](22-advantages-and-impact.md)*
