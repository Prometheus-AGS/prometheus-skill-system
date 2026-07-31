# Tasks

- [ ] PROBE FIRST: can a DB constraint target definition->>'origin'?
- [ ] If YES: deliver that expression, add no columns — change still COMPLETES
- [ ] If NO: add origin + enabled columns, backfilled from definition JSONB
- [ ] Provider round-trips whichever form was chosen; existing rows survive with correct values
