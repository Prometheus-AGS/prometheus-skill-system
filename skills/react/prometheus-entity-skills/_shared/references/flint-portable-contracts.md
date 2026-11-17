# Flint portable contracts

Use this reference when an application consumes Flint Realtime Fabric through
the Prometheus entity graph or documents Gate/Forge security requirements. The
full release guide is [`release/flint-portable-contracts.md`](../../../release/flint-portable-contracts.md).

## Runtime API

- `createFlintAdapter(options)` accepts a structural `FlintClientLike` with
  `watchEntities` and `mutateEntity` methods.
- `publishFlintMutation(client, record)` publishes through that client.
- `RealtimeManager` owns batching and graph writes; the Flint adapter must not
  mutate the graph or UI directly.
- Consumers own the Flint SDK, authenticated client construction, and cleanup.

The machine runtime export ledger already contains both functions. This change
does not add a package export, so `library-exports.json` must not be regenerated.

## Client/server key separation

- Client bundles may use only the publishable anonymous-key path.
- The service-role key and `flint_sk_` server keys are server-only.
- Never place service-role material in `VITE_*`, Flutter assets, a Tauri
  webview, example fixtures, logs, or generated snippets.
- Tenant identity comes from verified claims; do not accept an arbitrary UI
  tenant selector as authorization.

## JWT and JWKS compatibility

- Production tokens require `iss` and UUID `tenant_id` claims.
- Asymmetric tokens require `kid`.
- Current RSA JWKS entries have standard `n` and `e` members and are compatible
  with strict RSA consumers.
- Current EC entries lack `crv`, `x`, and `y`; do not claim strict EC support.
- Symmetric keys are not published through JWKS.

## Forge guidance

Provisioning is an external server/operator flow: typed JSON `plan`, reviewed
plan hash, service-role-only `apply`, then `status`/`ddl` inspection. Preserve
namespace allowlists, one transaction, ENABLE/FORCE RLS, per-verb tenant
policies, audit records, and restart-required status for new REST routes.

There is no Prometheus Forge adapter. Do not generate client or hook code that
calls Forge provisioning.

## Evidence commands

```bash
pnpm run test:flint-contracts
pnpm run bdd:flint-contracts
pnpm run verify:flint-contracts
```

Live compatibility uses the manual `.github/workflows/flint-live-contract.yml`
lane with exact Realtime Fabric, Gate, and Forge revisions. Missing or drifting
external inputs fail rather than skip: every supplied root must be a Git
worktree whose `HEAD` equals the declared revision before file hashes are
accepted. The verifier hashes both `HEAD:<path>` and the working-tree bytes, so
a dirty pinned file cannot inherit the commit's evidence.

Client-secret verification scans every repository-owned text-like file and
generated output under `examples/`, including `.next`, `build`, `dist`,
`target`, `.env*`, YAML, TOML, native configuration, and extensionless text.
Binary files are detected and reported separately. Third-party `node_modules`,
Git metadata, and `.gradle` caches are excluded explicitly and enumerated in
the receipt rather than silently counted as inspected example code. Generic
service-role assignments such as `SUPABASE_SERVICE_ROLE_KEY=...` are rejected,
not only Flint-prefixed variable names. JWT values are decoded for credential
classification, so a `service_role` payload cannot be hidden behind a
public-looking variable name.
