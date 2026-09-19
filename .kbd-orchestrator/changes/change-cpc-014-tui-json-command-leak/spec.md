# change-cpc-014-tui-json-command-leak

**Title:** Stop register-slash-commands.sh from writing command entries into tui.json
**Repository:** `prometheus-skill-pack`
**Phase:** control-plane-to-companion
**Backend:** native-kbd

## Why

`scripts/register-slash-commands.sh` lists `~/.config/opencode/tui.json` in `OPENCODE_JSONS` alongside the two real opencode config files. Every registration run injects the full `command` map (144 entries at time of writing) into `tui.json`, which is OpenCode's TUI settings file (`$schema: https://opencode.ai/tui.json`) and has no `command` concept. OpenCode currently tolerates the unknown key, so this is latent corruption rather than a boot failure — but it bloats the file, rides along in every backup, and any future strict validation of tui.json turns it into a startup failure of the same class as the dangling `{file:...}` incident.

Discovered while repairing the 2026-09-09/10 opencode boot failure (stale `sync-*` commands pointing at deleted skills). The stray entries were already cleaned from the live `~/.config/opencode/tui.json` on 2026-09-10; this change stops the script from re-adding them.

## What Changes

- Remove `~/.config/opencode/tui.json` from `OPENCODE_JSONS` in `scripts/register-slash-commands.sh`.
- On install (not just uninstall), sweep any existing `command` key out of `tui.json` so machines already polluted are repaired on the next registration run.

## Scope

Files this change may create or edit:

- `scripts/register-slash-commands.sh`

## ADDED Requirements

### Requirement: tui.json is never a command target
WHEN `scripts/register-slash-commands.sh` runs in install or uninstall mode, THEN `~/.config/opencode/tui.json` contains no `command` key afterwards, and both opencode.json files receive the command entries as before.

#### Scenario: clean machine
- **WHEN** the script runs on a machine whose tui.json has no `command` key
- **THEN** tui.json is left byte-identical (or at most re-serialized without new keys)

#### Scenario: polluted machine
- **WHEN** the script runs on a machine whose tui.json still carries a `command` map from an older version
- **THEN** the key is removed and the remaining TUI settings (e.g. `plugin`) are preserved

## Constraints

- Constraints C-01..C-05 apply; if plugin surfaces or install flow move, `npm run validate:codex` and `docs/codex-plugin.md` ship in the same change.
- Verification is local and command-based, recorded in `verification.md` — no unit tests as delivery evidence.
