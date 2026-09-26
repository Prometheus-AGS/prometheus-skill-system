# Independent routing review — completed correction confirmation

Date: 2026-09-26. Reviewer: catalog subagent, independent of the routing and project-installation implementation author. Verdict: **PASS for the three previously blocking routing findings**, after one batched correction and one confirmation cycle. This is a bounded routing review, not certification of the complete release.

## Confirmed corrections

| Finding | Evidence after correction | Result |
|---|---|---|
| Ordinary macOS UI requests rejected as Windows paths | Real full and mini route CLIs accepted a POSIX project path and resolved context without the former error. Source now checks drive/UNC forms rather than applying win32.isAbsolute to POSIX paths. | PASS |
| Mini selected excluded native Impeccable from an environment variable alone | Real mini CLI with IMPECCABLE_BIN set to an existing executable selected prometheus-impeccable-core and did not select impeccable. Source requires the bundled sibling impeccable/SKILL.md as well as the configured executable. No native executable was launched. | PASS |
| Pricing/article/changelog surfaces fell back to Operate | Both packaged CLIs returned Persuade for marketing, pricing and campaign; Read for articles and changelogs. | PASS |

Execution host: macOS (darwin), Node v24.16.0. Eleven real CLI requests returned exit code 0 with the expected routes. Fixture project contained a package.json declaring React and Electron. Inputs specified ui=true, operation=new and model=gpt-6; cases varied the surface and the native-engine environment setting.

Production entrypoints exercised:

- Full: node skills/ui-ux/prometheus-ui-ux/scripts/cli.mjs route --input <request.json>
- Mini: node skills/prometheus-ui-ux/scripts/cli.mjs route --input <request.json>

Fixture directory: /var/folders/ln/0wnpd96j26z2qhvx9m6hwt2r0000gn/T/uiux-routing-confirmation-skBh76. Example input: {"project":"<fixture>","ui":true,"operation":"new","surface":"pricing","model":"gpt-6"}.

Compiled routing artifact SHA-256:

- Full: b3bbc60eff4bab01e0216053f4a56c7b06d85342284fd43a51bd0e4a9bdc136d
- Mini: b3bbc60eff4bab01e0216053f4a56c7b06d85342284fd43a51bd0e4a9bdc136d

## Earlier completed-phase source review

The independent source review covered the shared routing/install modules and creator project-install, project-files, project-instructions and UI bindings. It found the three routing blockers above and no additional blocking creator issue. All 67 creator source/runtime/package files compared equal between full and mini at that review boundary. Creator live integration scenarios were owned by the main integration gate; this reviewer did not rerun or certify them.

Catalog integrity was **self-review**, not independent review: this reviewer authored the catalog. At its original completed-phase boundary, 442 recorded asset digests and 383 portable assets passed digest and full/mini byte comparison; manual-only invocation flags were present; the UI additions contained no forbidden interpreter, shell launcher or native binary file. Subsequent catalog refreshes are covered by the main gate, not inferred from those earlier counts.

## Explicit acceptance gaps and scope

Native Windows and Linux execution were not run by this reviewer. A Windows path-string check on macOS is not Windows runtime evidence. Native Impeccable engine execution, Apple/Android builds, rendered UI/device/theme capture, and The Boss installed-image certification were not part of these three routing confirmations. No independent catalog certification is claimed. No new findings were sought outside the requested confirmation scope, and no production source was changed during review.
