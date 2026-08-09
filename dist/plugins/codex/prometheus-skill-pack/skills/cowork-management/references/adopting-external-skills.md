# Adopting an external skill

`discover-skills.sh` finds candidates. It never installs one. This is the path
from a search hit to an adopted skill, and every step is an explicit operator
decision.

## The rule

> **Never auto-install a discovered skill.** Discovery answers a question;
> installation executes third-party code. A step that installed what it found
> would run unreviewed code as a side effect of asking whether something exists.

This is why `discover-skills.sh` shells out to `cowork search` only, and why its
output fixes every `verdict` to `unevaluated`. A search hit is a lead, not a
recommendation — `stars` is a popularity signal, never a safety one.

## The ordering constraint that shapes this flow

`cowork audit` scans **installed** skills (`--global`, `--project`, `--plugins`).
It takes no repository argument:

```console
$ cowork audit databasus/databasus
error: unexpected argument 'databasus/databasus' found
```

So audit **cannot** vet a candidate before installation. That rules out the
intuitive "audit, then install" order and forces the safer real sequence:
install into a **scoped, disposable** location first, audit there, and promote
only if it passes. `cowork install` also has no `--dry-run`.

## The flow

### 1. Read the source before running it

The cheapest review is the one that needs no execution:

```bash
open "$(python3 -c 'import json;print(json.load(open("candidates.json"))["candidates"][0]["repo_url"])')"
```

Check what it actually does, its licence, when it was last touched, and whether
any `scripts/` it ships would run on your machine. Most candidates end here.

### 2. Install to project scope, never global

Project scope keeps the blast radius to one repository and makes removal a
directory delete:

```bash
cowork install owner/repo --agent claude-code   # lands in .claude/skills/
```

Do **not** use global install for an unvetted skill. `cowork install --uninstall
owner/repo` reverses it.

### 3. Audit what you just installed

```bash
cowork audit --project --format json --output audit.json
```

Read the findings before using the skill for anything. A failing audit means
uninstall — not "note it and continue".

### 4. Verify checksums

```bash
cowork verify
```

This confirms the installed bytes match the lockfile, so a later silent mutation
is detectable. Run it again after any `cowork install --update`.

### 5. Decide, and record the decision

Only now does the candidate earn a real verdict. Record it where the phase can
see it — an adopted skill with no written rationale becomes an unexplained
dependency the next reader cannot evaluate.

## What this flow does not give you

Stated plainly, because the gaps matter:

- **No pre-install vetting.** Steps 3–4 run *after* code is on disk. Step 1 —
  reading the source — is the only pre-execution control, and it is manual.
- **`cowork audit` is a scanner, not a proof.** A clean audit means known
  patterns were absent, not that the skill is safe.
- **`cowork verify` proves integrity, not intent.** It confirms bytes are
  unchanged since install; it says nothing about whether they were ever good.

Treat an external skill as third-party code you have chosen to run, and size the
review to what it can reach.
