---
title: UI/UX routing
description: Install selective UI guidance, preserve design authority, and collect completed-phase evidence with the project team.
---

# UI/UX routing

Use `prometheus-ui-ux` when work changes rendered UI, interaction, styles, tokens, motion or on-screen copy. Describe the affected application and whether you are creating, redesigning, refining or reviewing it. The router returns context to read and a small set of relevant skills. Backend-only work uses the selected project team without loading UI guidance.

The canonical UI catalog contains **41 entries**. The portable project installer and mini pack carry **40**; full distribution also includes the optional native Impeccable integration. The full pack's generated distribution index currently lists **215 skills** across all domains. These are different inventories: the site catalog currently enumerates 194 source manifests excluding imported submodules, while the distribution index enumerates 215 packaged skills. See the [generated catalog](/docs/catalog/ui-ux) and [distribution index](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/SKILLS.md).

## Add UI guidance to a project

Run these commands from the full pack checkout, replacing the project path with an existing authorized directory. For installed skills, use the actual skill directory reported by your harness. Node.js 22 or newer runs the shipped helpers without a TypeScript installation or network service.

```text
node skills/ui-ux/prometheus-ui-ux/scripts/cli.mjs install --project "/path/to/project" --dry-run
node skills/ui-ux/prometheus-ui-ux/scripts/cli.mjs install --project "/path/to/project"
node skills/ui-ux/prometheus-ui-ux/scripts/cli.mjs install --project "/path/to/project" --check
```

The installer copies portable skills into `.agents/skills` and `.claude/skills`, installs a protocol if none exists, and adds managed pointers to both instruction entrypoints. It preserves project protocol overrides, unrelated prose and in-project linked entrypoints. It checks marker pairs and path containment before writing and retains recovery records for changed instructions. Malformed markers or escaping links must be resolved before installation; repeatedly forcing a bootstrap is not a repair procedure.

Full-pack context bootstrap, including legacy and v4 layouts, and the existing UI rules injector call this same helper. Use the direct Node commands above when only UI routing is needed. Broader bootstrap has its own shell/tool prerequisites and changes beyond UI routing. In projects whose instruction files are generated, add the routing instructions to `rules/src/routing.md` before regenerating; do not rely on an edit to generated output. Exact [bootstrap and injector examples](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/reference/ui-ux-routing.md) are in the command reference.

## Preserve the project's design decisions

Read the project override `.agents/UI_UX_PROTOCOL.md` when present, otherwise the bundled protocol. Load `PRODUCT.md`, `DESIGN.md`, `.impeccable.md`, the affected application's surface brief, existing components and tokens before choosing direction. A missing design document does not turn an existing application into a blank canvas.

Pro Max supplies focused domain and stack recommendations. Its TypeScript 7 source ships as Node ESM with pinned data; the full and mini portable helpers use identical bytes. Ordinary refinement and review use focused search. Generate a complete design system only when establishing or explicitly replacing direction. Persisted Pro Max output stays under `design-system/<project>/`; Stitch uses `DESIGN.stitch.md`. Neither overrides approved `DESIGN.md` or incumbent tokens.

Full-only Impeccable engine execution requires an explicitly configured, preinstalled `IMPECCABLE_BIN`. The launcher never downloads an engine. When unavailable, read project context directly and continue with permitted guidance. Mini uses `prometheus-impeccable-core`, an explicitly bounded context and craft adaptation. It does not provide native detector hooks, engine heuristics or live-browser parity. The proposed Node engine port remains deferred.

## Route a task

Save this request as `ui-request.json`, using your actual affected paths and model:

```json
{
  "project": "/path/to/project",
  "affected": ["apps/web/src/Settings.tsx"],
  "operation": "refine",
  "surface": "app",
  "model": "gpt-6",
  "focus": "accessibility",
  "ui": true
}
```

```text
node skills/ui-ux/prometheus-ui-ux/scripts/cli.mjs route --input ui-request.json
```

Use returned context paths first, then load the selected skills in order. Missing skills are reported gaps, not authorization to install dependencies. The model value is the actual model identifier; a Codex harness name does not establish that the model is GPT.

| Operation | Direction guidance |
| --- | --- |
| `new` | `gpt-taste` for actual GPT-family models; otherwise `design-taste-frontend`. |
| `redesign` | `redesign-existing-projects` after explicit redesign authorization. |
| `refine` | Preserve identity; no taste implementation or overlay. This is the default. |
| `review` | Objective scope and evidence review through `prometheus-ui-review`; no taste. Reviewer, verifier and auditor roles force this route. |

New work and redesign allow at most one direction implementation and one explicitly chosen overlay. Experimental/v1 taste identities remain distinct. Craft selection is narrow: layout → `better-layout`; typography → `better-typography`; color → `better-colors`; copy → `better-writing`; accessibility → `better-accessibility`; motion → `better-ui`; broad work → `better-interface`.

Surface modes are Persuade for marketing, Operate for apps, Read for documentation and Experience for showcases. They provide starting ranges for variance, motion and density; existing design decisions and reduced-motion requirements win.

Platform selection reads the nearest affected application's manifests, including nested applications. React/Vercel guidance applies to relevant components; Expo must match its declared SDK. Flutter, SwiftUI and Android retain their actual toolchain prerequisites and project versions. Tauri and Electron use the project's existing workflow. Portable guidance does not mean an unsupported host can build or profile native applications.

Upstream `interface-review`, `break`, `variant` and `explain-interface` remain user-only. Automatic review uses the Prometheus reviewer and eligible craft guidance. Reading a user-only skill's source is not an invocation workaround.

## Adopt the project team

The UI installer and project-team installer are separate operations. For an existing team, run:

```text
node skills/process/agent-team-creator/scripts/cli.mjs install-project --project "/path/to/project" --dry-run
node skills/process/agent-team-creator/scripts/cli.mjs install-project --project "/path/to/project"
node skills/process/agent-team-creator/scripts/cli.mjs install-project --project "/path/to/project" --check
```

An existing selection in `.agent-team/project-routing.json` wins. Without one, a sole `.agent-team/<id>/team.json` is adopted. Multiple candidates require an explicit `--team <id>` selection; a stale selection is an error. The installed pointer makes relevant selected roles the default for **all code tasks**. UI roles receive conditional routing; backend tasks stay free of UI guidance.

Creator `export` stages a proposal into a new directory. `install-project` installs discovery instructions, the routing record and missing native definitions. Normal project-team creation finishes with installation after inspection. Existing native files, permissions, model settings, concurrency and ownership are preserved; differences are reported for a deliberate merge. Neither command starts a team or activates a service registration.

Use native delegation only when the active harness exposes it. Otherwise follow the selected role instructions sequentially and disclose that limitation. Builder-context review is not independent review. Zed receives the pointer in its first effective existing instruction file as well as the standard entrypoints; external ACP agents retain their own configuration. Zed's parallel-thread UI does not establish a delegation API. See [agent teams](24-agent-teams.md) for creation and handoff.

## Finish the phase, then collect evidence

Complete the entire authorized production phase before verification. Change the request operation to `review` and run:

```text
node skills/ui-ux/prometheus-ui-ux/scripts/cli.mjs phase-boundary --input ui-request.json
```

The response is `status: "evidence-required"` and `executed: false`. It supplies routing, a checklist and a Node hook descriptor. It accepts no evidence fields, captures no screenshots, launches no browser and cannot certify PASS. Exit 0 means the request was processed. Wire it to an existing completed-phase runner, never a per-edit event.

Collect applicable device widths, theme states, real-content captures, keyboard focus and reduced-motion evidence with available platform tools. Obtain independent read-only review in a separate context, then use one batched correction and confirmation cycle. Record reviewer identity, evidence paths, unresolved findings and platform gaps. Missing independent review stays unverified; unresolved blockers stay blocking.

## What the existing evidence establishes

The [delivery record](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/research/ui-ux-routing/DELIVERY.md) separates shipped implementation from acceptance gaps. Recorded local evidence includes macOS and offline Linux Node 22 bootstrap/creator/routing scenarios, full bootstrap/injector coverage, pinned Pro Max comparisons, distribution checks and a separate skill-contract judge. These are prior implementation receipts, not new certification by this guide.

Native Windows execution, live invocation of every supported harness and a full installed Electron application run remain unverified. The native engine port is deferred. Catalog author review is self-review, and skill-contract review does not complete the ten-stage research review/export pipeline. Release certification remains open. No hosted CI result is used as validation evidence.
