# Verification — kde-002

1. Generator syntax passes under bash 5 and /bin/bash 3.2.
2. Generated manifest parses. It contains `sessionStart` **only in the
   t1-positive branch**. If t1 shows `kbd-status` misbehaves outside a KBD
   project and no suitable alternative exists, the correct state is NO
   `sessionStart` field plus a recorded finding — this gate passes in that
   branch too. It fails only on a malformed manifest, or on a field emitted
   despite t1 showing it should not be.
3. Package still reports 145 skills; installer idempotent.
4. `npm run validate` → 145 skills, 0 errors.

## Decisive evidence

The field must be observed FIRING, not merely present. Two prior defects in this
repo (the `config.toml [hooks]` path and `{{file:}}` slash commands) both parsed
cleanly and did nothing. Presence is not proof.

t1 is blocking: if `kbd-status` misbehaves outside a KBD project, shipping it
degrades every unrelated Kimi Desktop session.
