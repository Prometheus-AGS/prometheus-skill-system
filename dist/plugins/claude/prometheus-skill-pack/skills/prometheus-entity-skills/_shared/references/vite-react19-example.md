# Certified React 19 and Vite 8 showcase

Load this reference when generating, reviewing, or making readiness claims
about the React 19/Vite 8 example or the React-first 3.0 release candidate.

## Source and architecture

- Application: `examples/vite-app`
- Route: `/release-showcase`
- UI: `src/pages/release-showcase/release-showcase-page.tsx`
- Hook/ViewModel: `src/features/release-showcase/release-showcase-hooks.ts`
- Store: `src/features/release-showcase/release-showcase-store.ts`
- Service/adapters: `src/features/release-showcase/release-showcase-service.ts`

Preserve `component -> hook -> store -> service/adapter`. Do not move transport,
persistence, realtime, or CRDT work into the page component. Lists retain IDs
and rejoin canonical graph entities.

## Required verification

For the complete source-workspace application claim, require:

```bash
pnpm run bdd:vite-react19
```

The receipt must report:

- React and Vite typechecks passed;
- targeted local/remote/hybrid query and cache-miss Suspense units passed;
- core, React, sync, and Vite production builds passed;
- three expected Playwright tests and zero unexpected tests;
- every scenario declared by the `react-19-vite-8` coverage entry passed;
- runtime DevTools evidence passed;
- zero serious or critical accessibility violations; and
- screenshots and traces have recorded SHA-256 hashes.

## Loro browser loader

In Vite/browser code, pass a statically visible optional-peer loader:

```ts
import {
  createLoroProvider,
  type LoroChannel,
} from "@prometheus-ags/entity-graph-sync";

declare const channel: LoroChannel;
declare const peerId: number;

// Browser bundlers: supply the loader so the runtime-only peer import resolves.
const loadLoro = () => import("loro-crdt");
createLoroProvider({ channel, peerId, loadLoro });
```

`LoroProviderOptions.loadLoro` and core's `LoroModuleLoader` are public type
contracts. Node consumers may omit the callback. Do not convert `loro-crdt`
into a mandatory bundled dependency of core.

## Claim boundaries

The evidence kind is `source-workspace-production-browser` and must retain
`countsAsPackedPackageEvidence: false`. It proves the implemented React/Vite
showcase, not tarball installation, registry publication, remote service
availability, other browsers, SSR, mobile/device behavior, or the full 3.0
portfolio.

For npm RC claims, also load `release-candidate-pipeline.md` and require a new
packed-candidate report plus immutable rehearsal for the exact commit. Core
must stage before the React binding. Use npm `next`; never move `latest` based
on this showcase receipt.
