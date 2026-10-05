# 20 · Updating

Update independently versioned components from approved source commits and preserve recovery records. No umbrella version implies that memory, knowledge, execution and Companion binaries share a version or acceptance state.

## Safe sequence

1. Record approved source/gitlink identities, installed artifacts, selected homes and active/previous plugin generation receipts.
2. Preserve database, knowledge, signed KBD state, private identities, learner store and pending queues with relevant writers stopped.
3. Complete all production changes, then run the required consolidated local integration gate in isolated state.
4. Install approved binaries and activate the verified generation. Apply only selected platform service definitions.
5. Exercise actual installed paths, archive functional evidence and disposition every failure.
6. Push locally certified source for owner review/merge. Documentation publication must identify that same source.

Do not advance a gitlink by guessing a newer upstream commit, rewrite release evidence or erase uncertain queue records to obtain a green status. Protected version/tag changes need owner approval.

## Plugin lifecycle

From the matching source checkout:

```bash
./install.sh --profile skills --targets detected --non-interactive --yes
./install.sh --verify --targets detected --non-interactive
node scripts/install-plugin-generation.js --rollback
node scripts/install-plugin-generation.js --verify
```

Rollback uses the preserved previous complete generation and restores owned projections. Keep the same effective `CODEX_HOME` or selected home throughout; a successful logical store receipt alone does not verify a custom projection. Minimum-active versions are read from `skill-system.json`. Historical migration scripts target their named legacy layout and are not the normal update path.

## Services and recovery

```bash
bash scripts/install-mcp-services.sh --dry-run --restart
```

Inspect the actual service selection before applying `--restart`. Native service installation requires Bash 4+. A macOS LaunchAgent definition change needs bootout/bootstrap; restarting an already loaded job alone retains the old definition. Follow [Services, ownership and recovery](26-service-operations.md) and [Installation and upgrades](/docs/operations/installation-and-upgrades).

A restore must preserve project/device identity, operation receipts and compatible database/model schemas. Reconcile uncertain remote writes before another publication. Companion is separately installed and upgraded; its absence is normal.

Previous: [Installation](19-installation.md) · Next: [Contributing](21-contributing.md).
