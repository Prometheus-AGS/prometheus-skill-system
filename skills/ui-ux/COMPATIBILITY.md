# Pinned UI/UX collection

`catalog.lock.json` is the source inventory: immutable revisions, upstream names,
license evidence, imported asset hashes, invocation restrictions, and adaptations.
The importer is an explicit maintainer operation, never a skill startup hook:
`node scripts/import-uiux-catalog.mjs --mini <mini-checkout>`.
Installed guidance works offline. Refreshes require deliberate source review and
regeneration; do not follow repository HEAD or fetch fresh guidance during routing.
After local routing and Pro Max artifacts are produced, run
`node scripts/refresh-uiux-local-assets.mjs --mini <mini-checkout>`, then
`node scripts/sync-uiux-catalog.mjs --mini <mini-checkout>` for offline distribution.
The catalog's embedded copy is excluded from its own asset hashes to avoid a
recursive digest; the outer packaged distribution records its integrity.

## Distribution matrix

| Collection | Full pack | Mini | Execution prerequisite |
|---|---|---|---|
| Impeccable | Pinned guidance, references and browser assets; Node launcher for an explicitly installed native engine | Native skill excluded; separately named Prometheus core adaptation | Full engine requires `IMPECCABLE_BIN`; no automatic download |
| Taste | Nine selected upstream identities, including v1 and experimental v2 | Same adapted guidance | Existing harness tools only; Stitch requires an already available integration |
| Jakub craft | Seven `better-*` and four manual workflows | Same adapted guidance | Manual workflows retain `disable-model-invocation: true` |
| Vercel React, composition, React Native, web | Pinned skill and transitive rule references | Same adapted guidance | Project version decides applicable APIs; web guideline reference is bundled |
| Expo | Native UI, UI, design system, router, animation | Same guidance; local Node component inspector | Project SDK version; SDK 56+ examples are conditional |
| Flutter | Responsive layout, layout repair, declarative routing, localization | Same guidance | Actual Flutter SDK required for app execution |
| SwiftUI Pro | Pinned portable review/implementation guidance | Same guidance | Building/profiling needs Apple tooling; no Windows execution claim |
| Android | Adaptive and edge-to-edge, explicitly aliased | Same guidance | Declared Compose/navigation prerequisites; Android tools for execution |

`ui-ux-pro-max` is supplied by a separate portable TypeScript implementation and
data import. `prometheus-ui-ux`, `prometheus-ui-review`, and
`prometheus-impeccable-core` are Prometheus-owned routing/adaptation skills, not
upstream imports. Their manifests document their own capabilities.

## Scope and collision decisions

- The upstream `adaptive` and `edge-to-edge` names become
  `android-compose-adaptive` and `android-edge-to-edge`; original identities remain
  in provenance.
- Taste implementations retain `design-taste-frontend`,
  `design-taste-frontend-v1`, `gpt-taste`, and `redesign-existing-projects`.
  The current frontend v2 is explicitly experimental. Overlays remain
  `high-end-visual-design`, `minimalist-ui`, `industrial-brutalist-ui`,
  `full-output-enforcement`, and `stitch-design-taste`.
- No brandkit/image-generation skills were imported merely because they share the
  taste repository. No whole upstream repository is installed.
- Stitch's example and generated output use `DESIGN.stitch.md`. They do not
  supersede incumbent `DESIGN.md` or imply that Stitch is available.
- Vercel's agent-skills repository declares MIT in its README and selected
  frontmatter but has no standalone LICENSE at the recorded revision. Preserve
  that exact declaration as `UPSTREAM-LICENSE-EVIDENCE.md`; do not fabricate an
  upstream copyright notice. The separately pinned web guideline carries its
  own MIT license.
- Existing project tokens, accessibility obligations, tool permissions and
  framework pins override stylistic defaults. APCA advice does not replace WCAG
  2.2 conformance checks. Platform touch sizes and CSS pixel WCAG thresholds must
  not be conflated.

## Adaptations and limitations

Each imported `SKILL.md` carries a project-authority contract. Per-edit checks,
detectors and repeated review loops are replaced by the completed-phase review
boundary. Unix/Python discovery examples are replaced with harness file/Node
inspection. Expo unsolicited feedback submission is removed; its component
inspector is explicit ESM `.mjs`, reads only the installed project's declarations,
and has no network dependency. A skill is not permission to install packages.

The four user-only workflows (`interface-review`, `break`, `variant`,
`explain-interface`) remain available only on user invocation. Automatic review
uses the dedicated Prometheus review workflow; directly reading a manual skill
is not a workaround for its restriction.

This matrix describes inspected source and packaging intent, not native runtime
certification. Native Windows/macOS/Linux execution and actual rendered UI
evidence are recorded by the consolidated phase gate, not inferred from a file
extension or successful download.

## Deferred Impeccable engine port

Dependency: upstream's native engine performs detector analysis, browser
instrumentation and hook administration. Merely replacing its shell launcher
would leave the native dependency intact.

Proposed Node interface:
`node scripts/impeccable-engine.mjs detect --input recorded-dom-style.json --design DESIGN.md`.
The portable input must be a versioned JSON document containing relevant DOM
semantics, computed styles, viewport/device/theme and source locations. A future
live adapter obtains this through the harness's existing browser capability;
there is no extra daemon or invented browser API.

Retain now: context loading, incumbent surface resolution, mode selection,
selective skill routing and completed-phase review integration in the separately
named core adaptation. Omit native detection, live DOM injection/session control,
native hook installation and binary updates. Do not advertise full parity.

Acceptance for a later port: recorded fixture parity with the pinned engine's
supported rule IDs, severities, ignores and actionable locations; explicit
unsupported rules; permission-preserving browser adapter; offline native Windows
Node execution; no native executable, runtime download or extra service; bounded
phase review with one batched confirmation. A shell wrapper alone satisfies none
of these criteria.
