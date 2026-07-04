# Goals — cowork-integration

## Context

Investigate and plan the integration of the `cowork` CLI utility (forked codebase at
`git@github.com:GQAdonis/cowork-skills.git`) into the prometheus-skill-pack as a standard
installation/management CLI. Work will happen in a dedicated worktree outside the skill-pack
directory to allow clean investigation without polluting the main tree.

## Goals

- G-01: Investigate the cowork forked codebase and produce an architecture assessment with a
  clear integration plan for adding it as a standard CLI in the prometheus skill pack.
- G-02: Add explicit support for Zed, Kimi Code CLI, MMX CLI, Kimi Desktop, and MiniMax Desktop
  to the cowork CLI so skills can be installed for all new target platforms.
- G-03: Make cowork aware of how the prometheus-skill-pack is managed so it can be used to update
  the pack, update toolchains, and repair broken installations.
- G-04: Make cowork understand Claude Code plugin and marketplace mechanics in full detail; update
  it to support installing and managing Codex plugins and OpenCode plugins.
- G-05: Integrate the updated cowork CLI into the skill-pack install pipeline and document its
  usage as the primary skill-management utility.
