# Round-1 response — producer corrections before re-review

Round 1 judged a packet this session BUILT WRONG. Two of the four findings are
consequences of that defect, one is out of scope for reasons the judge could not
see, and one is upheld. Re-review the corrected packet.

## F1 (CRITICAL, session records absent) — UPHELD AGAINST THE PACKET, not the commit.
You were right that the packet was inconsistent. The cause was mine: the round-1
`git show` passed the SAME sha twice (a literal `dcfeb92` plus a subshell that
resolved to `dcfeb92`) and applied a pathspec, so the docs commit `1005a1f` was
absent and `dcfeb92`'s own file list was filtered. Ground truth, re-verified:
  git ls-files --error-unmatch <both records>  -> both YES, both in dcfeb92
  git status --porcelain -- .prometheus        -> empty
The corrected packet contains BOTH commits, unfiltered.

## F2 (CRITICAL, duplicate records) — REJECTED, with reasons you lacked.
Your proposed fix ("a one-file delete, not a follow-up project") is wrong here
because `index.md`/`log.md` are not this repo's to edit. CLAUDE.md records an
ownership split decided 2026-07-01:
  | index.md / log.md maintenance, body cross-links | prometheus-knowledge-rs (pk-librarian) |
The wiki is maintained by the `pk` CLI shipped from a SEPARATE repository. The
duplicate is a pk ingest defect (one session, 9db42325, emitted twice 4s apart).
Hand-deleting a record and its index/log lines here would (a) edit files another
repo owns and (b) likely be re-emitted by the next pk run. Corpus check: 176
karpathy records, ZERO body-identical duplicate groups — this is a one-off, not
rot. Correct venue is a prometheus-knowledge-rs issue, recorded as phase debt.

## F3 (WARNING, created_at rewrite) — OUT OF SCOPE, and not producer-authored.
The rewrite arrived in the working tree from pk; this change only staged it.
Same ownership split: frontmatter format/writer is pk-store/pk-core's concern.
OKF v0.1 (vendored at shared/references/okf-v0.1.md) states no created_at
immutability rule, and mandates permissive consumption. Real, but belongs
upstream. Recorded as debt.

## F4 (WARNING, unreliable diff source) — UPHELD, and it is the root cause of F1.
build-review-packet.sh:196 uses `git diff HEAD`, empty for committed work. Known
as the c400 defect, still unfixed, now recorded as phase debt with this as a
second concrete instance.

Judge the CORRECTED packet. Do not soften findings because the producer pushed
back — F1 and F4 were correct and are upheld.
