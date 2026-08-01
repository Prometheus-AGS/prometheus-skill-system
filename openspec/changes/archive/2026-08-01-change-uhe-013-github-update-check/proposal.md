# Know when an update exists, and initiate it

**Change:** `change-uhe-013-github-update-check`
**Phase:** uar-host-execution
**Goal:** R5

## Why

See `.kbd-orchestrator/phases/uar-host-execution/plan.md` for full rationale,
acceptance criteria, and the two-round adversarial review record.

## Outcome: update check shipped; the failure path is the design

```
test result: ok. 5 passed; 0 failed
  a_network_failure_is_unknown_never_up_to_date ... ok
  absent_local_provenance_is_unknown_not_up_to_date ... ok
  identical_commits_are_up_to_date ... ok
  differing_commits_report_behind_with_both_sides ... ok
```

### The rule, and why the type enforces it

**A check that cannot reach the network reports `unknown`, never `up-to-date`.**

That is not defensive style. "Up to date" is a claim *about the remote*; if the
remote was never contacted the claim is unsupported. Reporting `UpToDate` on a
failed request produces a system that looks healthy precisely when it has
stopped being able to tell — and a user who trusts a green check will never look
again. An honest `unknown` prompts a retry; a dishonest `up-to-date` ends the
conversation.

`UpdateStatus` makes the mistake hard to make: every failure path constructs
`Unknown` **with the reason attached**, and there is no code path from an error
to `UpToDate`. Four distinct failures all land there:

| Failure | Result |
|---|---|
| DNS / connection error | `unknown` + reason |
| GitHub 404 or rate limit | `unknown` + status |
| Unparseable response | `unknown` + reason |
| Local provenance has no commit | `unknown` — knowing the remote but not ourselves is not a basis for any claim |

### Tested with no network

`compare_to_remote(local, remote: Result<RemoteHead, String>)` is **pure** — it
takes what the remote said rather than fetching it. Every branch, including all
failure branches, is reachable in a test with no network and no fixture server.
`fetch_remote_head` is the only part that does I/O, and it is a thin adapter.

`UAR_SKILL_PACK_REPO` overrides the repository, so a fork or a test can point
elsewhere without editing code.

### Two endpoints

| Method | Path | Behaviour |
|---|---|---|
| `GET` | `/update-check` | always `200` — "we could not tell you" is a successful answer; a 5xx would push callers into retry loops for an often-permanent condition (offline device, rate limit) |
| `POST` | `/update` | reports the update path; `503` when status is `unknown` |

**`POST /update` deliberately reports rather than executes.** Updating the pack
means moving a git submodule *the host process is running from*; doing that
under a live server risks reading a half-updated tree, and the safe sequence
(fetch → verify → swap → reload) needs a restart boundary this endpoint does not
own.

It also refuses to hand out steps on an `unknown` — acting on one is how a user
ends up "updating" to the version they already have. **An endpoint that claims to
have updated when it has not is the same failure class as a check that reports
`up-to-date` while offline.**
