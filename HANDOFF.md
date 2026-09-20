# HANDOFF — gomark/rules-architecture-v4

Branch `gomark/rules-architecture-v4`, **stacked on `gomark/kbd-orchestrator-writable`** (commit `37aa5bc`).
Made from the go-mark project (OpenSpec change `c14-skillpack-bootstrap-v4`). Not pushed, not merged, not
installed. The generator, scripts and hooks were proven in go-mark first (its change `c01b`) and copied here.

## What changed

All in `skills/process/prometheus-context-bootstrap/` (+ both `dist/` mirrors, `CHANGELOG.md`):

- `references/rules-src/` — generic v4 sources: `constitution.md` (v3 §A verbatim + A-15…A-17, `__PROJECT__`
  placeholder block), `routing.md` (the full v4 routing table incl. Next.js rows), `tech/` ×7, `domain/` ×4,
  `project/README.md`. No project-specific content.
- `assets/v4/rules/{build.sh,build.py,build.conf,line-limit-allowlist.txt}` — the generator, seeded into a
  project's `rules/`.
- `assets/scripts/check-{file-lines,architecture}.sh`, `assets/hooks/{file-lines-guard,build-guard}.sh`,
  `assets/githooks/commit-msg`.
- `scripts/bootstrap-v4.sh`, `scripts/verify-v4.sh` — new.
- `scripts/bootstrap.sh` — dispatches `--layout v4` to the new script; default path unchanged.
  The settings.json block moved to `scripts/lib/settings.sh` (`wire_settings`): `bootstrap.sh` had grown to
  533 lines across this branch and the one below it, over the 500-line rule this change introduces.
  It is 466 now. Pure move, no behaviour change (verified below).
- `scripts/verify.sh` — runs `verify-v4.sh` when `CLAUDE.md` carries the `prometheus-rules: v4` header;
  otherwise every existing check runs as before.
- `SKILL.md` — new "Layout v4" section.

## How it was verified (temp-dir fixtures, run from this worktree)

- `--layout v4 --dry-run` writes 0 files.
- Fresh project, `--layout v4 --stacks rust,typescript`: `CLAUDE.md` rendered with the v4 header, `AGENTS.md`
  and `GEMINI.md` are symlinks to it, six rule files rendered, `verify.sh` → PASS 14, FAIL 0, WARN 4
  (placeholder §P line, `core.hooksPath` not set, no waypoint, machine-wide skill budget).
- Re-running `--layout v4` reports an edited `rules/src/` file as SKIP and keeps the edit.
- **Legacy unchanged:** `bootstrap.sh` without `--layout`, compared with the parent branch's script — identical
  stdout and identical output trees, on a fresh project and on one carrying the old `.kbd-orchestrator` deny
  rule (REPAIR + MERGE path, identical `settings.json`). `verify.sh` on a legacy project — identical output.
- `diff -rq` of the skill against both `dist/` mirrors: empty.
- Not run: `npm run validate`, `npm run validate:plugins`, cucumber — this worktree has no `node_modules`
  and no submodules. **Run them from the main checkout after merging.**

## Found while doing this — not fixed here

- A seeded `check-file-lines.sh` first judged *this* repo instead of the fixture because it resolved the
  root from the caller's cwd. Fixed (it now resolves from its own location). While wrong, it listed
  skill-pack files far over 500 lines: `crates/prometheus-exec/src/daemon.rs` (963), `doctor.rs` (906), and
  others. If the 500-line rule is to hold estate-wide, this repo has partitioning to do.
- `tier-guard.sh` matches text anywhere in a Bash command, including heredoc bodies: writing a rules file
  that mentions a release build through `cat <<EOF` is blocked. `build-guard.sh` strips heredoc bodies first;
  `tier-guard.sh` should too.
- `adversarial-review/scripts/build-review-packet.sh` diffs the working tree (`git diff HEAD`), so with a
  committed change and an unrelated uncommitted edit present it reviews the wrong thing.
- The v4 design lists a `pk-focus` UserPromptSubmit hook and a `forge reflect` Stop hook as existing. Neither
  exists as a hook.

## Owner steps

1. Merge `gomark/kbd-orchestrator-writable` first, then this branch (it is stacked on it).
2. From the main checkout: `npm run build` (regenerates `dist/` properly — the mirrors here were synced by
   hand and should come out byte-identical), `npm run validate`, `npm run validate:plugins`.
3. Reinstall (`npm run install:user`) so `~/.prometheus/plugins/…/current` and the plugin cache pick it up.
4. Decide when `v4` becomes the default layout, and whether to write `migrate.sh --to v4` (not done: moving a
   project's own rules into `rules/src/project/` is a judgement call the script should only report on).
5. Retire the stale rule documents: `~/.claude/AGENT_BASE_RULES.md` and the builder's copy are the untiered
   v2-era text; the canonical v3 is this repo's `AGENT_BASE_RULES.md`.
