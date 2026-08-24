
## Re-verification (2026-08-24) — the ledger's own claims were stale

c405 arrived fully checked, dated 2026-08-23. Two claims could not have been true
as written, so every criterion was re-run rather than accepted.

**2.4 was false when written.** It asserted `origin/main` contained c400-c404, but
c404 was still an open change at that point and was only archived today. At the
time of re-check, `origin/main` (`be3697e`) carried c400-c403 and **zero** c404
files. Now corrected: `origin/main` is `401e051` with all five present.

**2.2 and 2.3 re-verified live, and they hold.** This is the payoff of the whole
phase, so it was tested rather than assumed:

- The installed `~/.claude/skills/adversarial-review/scripts/preflight-models.sh`
  contains the upward-walk resolver (`while [ "$_lib_dir" != "/" ]`) and one
  `resolver_missing` branch.
- Run with **`CLAUDE_PLUGIN_ROOT` unset** — the condition the criterion names —
  it reports `status: ok`, gateway `http://localhost:4000/v1`,
  `distinct_models: 2`, roles judge=`k3` / critic=`MiniMax-M3`, and
  `config_defects: []`.

That closes the loop the phase opened: the resolver bug that silently made every
review same-model (while reporting a nonexistent expired credential) is dead in
the *installed* artifact, not merely in source.

**A caveat worth stating.** The `update-skill-pack.sh --force` run recorded in 2.1
happened on 2026-08-23, before c404's pin reconciliation and before the branch
convergence. I did not re-run the installer — the criterion it supports is about
the installed preflight's behaviour, and that behaviour was re-verified directly
today. If a future change touches the resolver, the install must be re-run rather
than inferred from this record.
