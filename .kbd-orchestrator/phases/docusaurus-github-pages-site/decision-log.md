# Decision Log — docusaurus-github-pages-site

### 2026-07-27T10:05Z — Domain / baseUrl
Options: custom domain (prometheus-skill-pack.prometheusags.ai + CNAME) vs default Pages URL
Decision: default `https://prometheus-ags.github.io/prometheus-skill-system/`, parameterized via `SITE_URL`/`BASE_URL` env (donor pattern) so a custom domain later is env + CNAME only.
Provenance: implicit (reversible; no contested-score condition) | Operator may override before execute.

### 2026-07-27T10:05Z — CI gate depth
Options: donor's full release:check suite vs minimal gate
Decision: minimal first — `onBrokenLinks: 'throw'` + docusaurus build + internal link check; full suite deferred to a later phase.
Provenance: implicit (cost/scope) | recorded in analysis.md.

### 2026-07-27T10:05Z — Brand contract scope
Options: port knowme `--km-*` contract this phase vs hold
Decision: HOLD — out of phase goals (flagged by assess adversarial vet as imported scope). cand-007 recorded as `reference`; plan may carry an optional change gated on operator approval.
Provenance: adversarial-review finding + pending user answer.

### 2026-07-27T10:05Z — G3 dedupe direction
Options: delete site copies vs serve canonical ../docs via plugins vs make site canonical
Decision: donor pattern (cand-003) — multi-instance plugin-content-docs reading canonical root `docs/`; reconcile the diverged `guide/` fork first (only true duplicate; learn/ and sovereign-sync/ are site-original per provenance diff).
Provenance: research (donor config evidence + local diffs).
