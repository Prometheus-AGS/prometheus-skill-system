# Prometheus Entity Management 3.0 release contract

The machine-readable source of truth for the 3.0 artifact inventory, stability labels, compatibility ranges, protocol boundaries, publication gates, and recovery policy is [`release/v3-release-contract.json`](../../../release/v3-release-contract.json).

Skills must read that contract before making 3.0 packaging or compatibility claims. Do not copy its artifact inventory into a skill: duplicated package lists can drift. In particular:

- stable and experimental surfaces must remain distinct;
- official A2UI rendering and AG-UI event transport are separate concerns;
- Flutter A2UI rendering (`a2ui_flutter`, pre-1.0 and pinned exactly since 5.0; `genui` in 3.0) remains experimental even though the Flutter entity-graph binding is in the release inventory;
- the npm `latest` tag cannot move without the explicit approval gate in the contract;
- a planned example in `examples/coverage.json` is not certified evidence.

## Main CI and dependency claims

The implemented JavaScript workspace baseline is documented in [`release/ci-baseline.md`](../../../release/ci-baseline.md) and mapped as `release.ci.hermetic-main-baseline` in [`examples/coverage.json`](../../../examples/coverage.json). It proves the frozen root pnpm workspace, supported Node matrix, named timeout-bounded gates, and production advisory policy. It does not prove packed consumers, native targets, showcase completeness, documentation deployment, or publication readiness.

Before claiming a dependency is “current” or a production advisory is accepted, read both machine-readable policies:

- [`release/dependency-policy.json`](../../../release/dependency-policy.json) records intentional direct-dependency holds and their owning revisit changes;
- [`security/advisory-policy.json`](../../../security/advisory-policy.json) records remediated advisories and any owner/rationale/expiry-bound exceptions.

Do not describe the repository as having zero risk merely because critical/high advisories are zero. Lower severities remain visible, and the baseline's threshold is a release decision rather than a claim that all dependencies are vulnerability-free.

## Packed npm package claims

The implemented package-artifact gate is documented in [`release/package-contracts.md`](../../../release/package-contracts.md) and mapped as `release.packages.packed-module-contracts` in the coverage ledger. Before claiming that a 3.0 npm candidate is importable, require-able, or correctly typed, require `pnpm run verify:package-contracts` evidence from packed tarballs. A workspace build, TypeScript source alias, or previously published alpha is not equivalent evidence.

The gate proves conditional ESM/CommonJS runtime and declaration routing, package metadata, intentional payloads, strict Publint/ATTW results, and isolated Node/TypeScript consumers. It does not prove framework singleton behavior, showcase correctness, visual evidence, native platform support, registry authority, or publication readiness.

## Framework-neutral core claims

The implemented core-boundary gate is documented in [`release/framework-neutral-core.md`](../../../release/framework-neutral-core.md) and mapped as `release.core.framework-neutral` in the coverage ledger. Require `pnpm run verify:framework-neutral-core` before claiming the packed core is React-free or usable without React types.

Core agents use `createGraphStore()` for isolated instances and `graphStore` for the default vanilla singleton. React agents import callable `useGraphStore(selector)` and `useGraphSyncStatus()` from `@prometheus-ags/prometheus-entity-management`. The deprecated core `useGraphStore` export is a StoreApi migration alias, not a hook. This gate alone does not prove that every binding resolves one physical singleton.

## Binding singleton claims

The implemented six-binding gate is documented in [`release/binding-singleton-contract.md`](../../../release/binding-singleton-contract.md) and mapped as `release.bindings.one-core-singleton` in the coverage ledger. Require `pnpm run verify:binding-singletons` before claiming that packed React, Svelte, Solid, Web Components, Alpine, and HTMX entry points resolve one physical core and observe one reactive graph.

Skills must install `@prometheus-ags/entity-graph-core` explicitly with a binding. Core is a required compatible peer and workspace development dependency, never a binding production dependency or optional peer. Do not suppress an incompatible peer failure or infer singleton identity from matching version strings alone.

This gate remains headless and does not certify native Tauri/Flutter behavior, browser/device rendering, showcases, visual evidence, documentation deployment, registry authority, or stable publication. Those statements remain blocked until their owning changes produce evidence.

## Shared example coverage claims

The implemented presentation-neutral contract is documented in [`examples/shared/README.md`](../../../examples/shared/README.md), declared as `release.examples.shared-semantic-contract` in the coverage ledger, and taught to agents in [`example-coverage-contract.md`](example-coverage-contract.md). Require `pnpm run verify:example-coverage` before claiming that a stable capability or artifact is mapped to the shared 3.0 scenario vocabulary.

The verifier proves deterministic fixtures, expected semantic outcomes, mapping completeness, and honest evidence-state transitions. It is deliberately headless. Do not translate a passing semantic scenario into a framework claim; require the owning showcase's runtime, platform, accessibility, and visual receipts. All five showcases now have implemented evidence entries, each bounded by its own commands, paths, and exclusions. This does not complete documentation, release certification, or publication.

## Official A2UI bridge claims

The implemented official bridge is documented in [`release/a2ui-protocol-bridge.md`](../../../release/a2ui-protocol-bridge.md), taught in [`a2ui-protocol-bridge.md`](a2ui-protocol-bridge.md), and mapped as `release.protocol.a2ui-official` in the coverage ledger. Require `pnpm run test:a2ui-bridge`, `pnpm run verify:a2ui-bridge`, and `pnpm run bdd:a2ui-bridge` before claiming official v0.9.1 rendering or the graph action policy is implemented.

The package root is official A2UI; alpha AG-UI chat/state APIs live under `@prometheus-ags/a2ui-react/ag-ui`. Protocol validation is not tenant/entity/field authorization. The bridge gate does not prove the separate A2A boundary, the full agentic showcase, Flutter rendering, docs deployment, registry authority, or stable promotion.

## A2A v1 JSON-RPC claims

The implemented protocol boundary is documented in [`release/a2a-conformance-agent.md`](../../../release/a2a-conformance-agent.md), taught in [`a2a-conformance-agent.md`](a2a-conformance-agent.md), and mapped as `release.protocol.a2a-jsonrpc-v1` in the coverage ledger. Require `pnpm run test:a2a-conformance`, `pnpm run verify:a2a-conformance`, and a successful pinned `pnpm run test:a2a-tck` receipt before claiming the A2A v1 JSON-RPC lifecycle is implemented.

The package root is official A2A v1 JSON-RPC; pre-v3 slash methods live only under `@prometheus-ags/entity-graph-a2a/legacy`. Authentication and caller-scoped application policy run before graph changes. Protocol validity never grants application authority. Do not infer support for excluded REST/gRPC bindings, push notifications, extension signing, a rendered agentic showcase, Flutter, docs deployment, registry authority, or stable promotion.

## Flutter source provenance claims

The implemented source-lineage gate is documented in [`release/flutter-source-provenance.md`](../../../release/flutter-source-provenance.md), taught in [`flutter-source-provenance.md`](flutter-source-provenance.md), and mapped as `release.flutter.source-provenance` in the coverage ledger. Require `pnpm run verify:flutter-source-provenance`, `pnpm run test:flutter-source-provenance`, and `pnpm run bdd:flutter-source-provenance` before claiming that the KnowMe Flutter source has licensed, history-preserving provenance.

`packages/entity_graph_flutter` is the sole canonical Dart graph package. The filtered import is non-buildable, non-workspace, and non-public. This gate has no public runtime export impact and does not certify Dart/Riverpod behavior, Flutter rendering, Android/iOS, accessibility, pub.dev authority, documentation deployment, or stable promotion.

The main React runtime export ledger remains [`library-exports.json`](library-exports.json). Sync uses [`sync-library-exports.json`](sync-library-exports.json), the A2UI root and compatibility subpath use [`a2ui-library-exports.json`](a2ui-library-exports.json), and the A2A root and legacy subpath use [`a2a-library-exports.json`](a2a-library-exports.json). A release-contract change does not alter a ledger unless its publishable entry point changes.

## Recoverable release-candidate claims

The implemented RC boundary is documented in
[`release/release-candidate-pipeline.md`](../../../release/release-candidate-pipeline.md),
taught in [`release-candidate-pipeline.md`](release-candidate-pipeline.md), and
mapped as `release.pipeline.recoverable-rc` in the coverage ledger. Require
`pnpm run test:release-pipeline`, `pnpm run bdd:release-pipeline`, and
`pnpm run verify:release-pipeline` before claiming the contract-derived package
order, tarball-only consumer proof, protected-tag rehearsal, or restartable
matching/absent/conflicting registry recovery.

That verifier records no registry mutation. A separate staging command is
fail-closed to GitHub Actions OIDC, `npm-rc`, an exact SHA, and `next`; checked-in
code cannot certify external trusted-publisher or reviewer settings. Never
translate this implemented gate into npm `latest`, a GitHub Release, production
docs deployment, or stable 3.0.0. Those remain owned by
`v3-release-certification` and `v3-stable-publication`.
