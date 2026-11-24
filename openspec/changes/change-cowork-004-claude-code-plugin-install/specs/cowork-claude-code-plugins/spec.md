## ADDED Requirements

### Requirement: Install a Claude Code plugin from Git

The install subcommand SHALL validate the Git checkout’s plugin name, version, skills, and license from `.claude-plugin/plugin.json` or the root fallback. It SHALL install to the selected user or project scope, merge the installed-plugin registry and `enabledPlugins`, and report installed skill paths.

#### Scenario: Repeat installation of the same valid plugin

- **GIVEN** a Git checkout contains an installable Claude Code plugin
- **WHEN** the operator installs it twice
- **THEN** both runs resolve the same plugin identity and unrelated registry and settings entries remain intact
