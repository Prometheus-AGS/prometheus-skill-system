# Verification — kde-002

1. Generator syntax passes under bash 5 and /bin/bash 3.2.
2. Generated manifest parses; contains `sessionStart`.
3. Package still reports 145 skills; installer idempotent.
4. `npm run validate` → 145 skills, 0 errors.

## Decisive evidence

The field must be observed FIRING, not merely present. Two prior defects in this
repo (the `config.toml [hooks]` path and `{{file:}}` slash commands) both parsed
cleanly and did nothing. Presence is not proof.

t1 is blocking: if `kbd-status` misbehaves outside a KBD project, shipping it
degrades every unrelated Kimi Desktop session.
