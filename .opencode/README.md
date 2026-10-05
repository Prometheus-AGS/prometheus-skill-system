# Full-pack OpenCode adapter

The local `plugin.ts` registers `evolve`, `gitops`, `kbd` and `kbd-close` tools.
Its `shell.env` hook sets `PROMETHEUS_SKILL_PACK=1`; its post-tool hook records a
`post_mutation` lifecycle observation through the shared KBD adapter. That
observation does not block ordinary tool calls.

This TypeScript adapter needs the dependencies declared by this checkout and the
full pack's shared Bash helpers. It is not the mini pack's Node-only runtime.
Skill installation and native plugin registration are separate operations.

Use [the root installation flow](../README.md#install-the-skills) to select the
OpenCode target. For project registration, the source entry point is:

```bash
npx tsx scripts/install-platforms.ts --platform opencode --scope project
```

Run from the repository root and inspect the configuration being changed. Keep
the registered source checkout available. Tool names are plugin API names; they
are not a promise that a particular harness exposes identical slash-command UI.

Read [hooks and lifecycle](../docs/guide/15-hooks-and-lifecycle.md) and
[platform support](../docs/guide/17-platform-support.md). Development validation
runs locally only after the complete production phase; source registration alone
does not certify a live OpenCode session.
