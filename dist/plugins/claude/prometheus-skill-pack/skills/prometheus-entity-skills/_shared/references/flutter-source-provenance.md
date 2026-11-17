# Flutter source provenance boundary

Load this reference before claiming that Flutter/Dart source was imported, adapted, licensed, or made publishable for 3.0.

The machine-readable authority and receipt are [`release/flutter-source-authority.json`](../../../release/flutter-source-authority.json) and [`release/flutter-source-provenance.json`](../../../release/flutter-source-provenance.json). Require all three commands before a source-lineage claim:

```bash
pnpm run verify:flutter-source-provenance
pnpm run test:flutter-source-provenance
pnpm run bdd:flutter-source-provenance
```

## Canonical owner

`packages/entity_graph_flutter` is the sole canonical Dart graph package. The filtered KnowMe tree at `provenance/imports/knowme-flutter` is non-buildable, non-workspace, and non-public provenance. Never add it to a workspace, package export, example alias, build, analyzer invocation, pub.dev candidate, or generated API ledger.

Adapt only paths marked `adapt` in the provenance receipt, and adapt them into the canonical package. A `reference` decision permits learning and documentation, not runtime copying. A `reject` decision remains excluded. `hybrid-mobile-architecture-src` is reference-only; do not invent a runtime library from its templates, scripts, or documentation.

## Public and release claims

This provenance change has no public runtime export impact, so no JavaScript public API ledger changes. It proves license, attribution, filtered history, explicit path dispositions, and one destination owner. It does not prove Dart runtime behavior, Riverpod 3 compatibility, Flutter rendering, Android/iOS behavior, accessibility, pub.dev authority, or stable 3.0.0 readiness.

For Dart implementation claims, wait for `v3-dart-graph-riverpod`. For rendered Flutter claims, wait for `v3-flutter-riverpod-a2ui-example`. For publication, require the later native registry, release certification, and stable publication evidence.
