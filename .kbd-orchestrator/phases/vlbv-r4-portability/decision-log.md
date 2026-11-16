# Decision log — vlbv-r4-portability

## D-R4-01 — Successor authorization signed by the agent at owner instruction (2026-11-16)

The start-successor manifest (`issue-195-closeout/start-successor.json` + `.sig`) was
generated from canonical state and signed with the registered owner key
(`~/.ssh/id_ed25519`, principal travis@tribehealthsolutions.com) **by the agent**, on the
owner's explicit chat instruction "Sign it for me and continue". The manifest `reason`
field states this. The signature therefore proves owner authorization of this scope,
not a human act at the keyboard. The key has no passphrase and is not held by an agent.

## D-R4-02 — Successor id

First id `vlbv-r4-portability-evidence` was consumed by a pending phase that
`kbd-new-phase.sh` created before its activation was refused (it runs `phase create`
before `phase activate`). That pending phase remains in canonical state, unused.
The successor started as `vlbv-r4-portability` (revision 2502).

## D-R4-03 — Owner decisions on assessment open questions (2026-11-16, in chat)

1. Auth material: **allowed.** The runner copies each harness's minimum auth file
   into a temporary scratch home outside the repo, deletes it after every run, and
   verifies that real harness state is unchanged.
2. Stochastic profile: **3 samples per case per harness**, declared before results.
3. Grader: **cross-model via liter-llm.** MiniMax-M3 grades; k3.1-pro or glm-5.3
   grades when the harness under test uses MiniMax.
