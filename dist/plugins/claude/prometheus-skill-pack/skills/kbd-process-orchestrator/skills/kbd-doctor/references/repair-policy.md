# kbd-doctor repair policy

## Safe actions

Doctor may automate only safe, reversible actions. In practice that means:

- dry-run planning;
- diagnostics and JSON output;
- focused reinstall/reload flows that call the repository-owned installers instead of duplicating shell logic;
- managed MCP config reconciliation for known client sections only;
- backup planning before any approved mutating pass, followed by a rescan.

`--yes` suppresses prompts only for actions in that safe/reversible set. It does not widen doctor's authority.

## Manual-only actions

Doctor must not automatically:

- rotate credentials or expose tokens;
- overwrite unknown MCP client sections;
- delete warehouse content;
- reset dirty submodules;
- advance dependency pins outside the parent repository;
- use `sudo`;
- remove unknown LaunchAgents.

Doctor also must not:

- recreate a missing checkout tree;
- overwrite unknown hook customizations;
- rebuild from a dirty submodule or reset local changes to make a rebuild possible.

These findings stay red/manual until a human resolves them explicitly.
