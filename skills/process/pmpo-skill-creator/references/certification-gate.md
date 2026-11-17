# Drafting with skill-creator, gating with skill-certification

Two other skills do work this one used to do alone, or did not do at all.

## Anthropic's skill-creator: drafting and cases

Anthropic's `skill-creator` skill is good at two things this creator needs: a
description that triggers, and eval cases in the folder layout
`claude plugin eval` reads.

**Detect it.** Look in the session's available skills for `skill-creator`, or a
namespaced form such as `anthropic-skills:skill-creator`. It is an external
skill and may be absent.

**When it is present**, hand it, during Execute:

- the `SKILL.md` description and body, given the `skill_spec` from Specify;
- the eval cases under `<skill_dir>/evals/`: at least two outcome cases and one
  near-miss case.

Take its drafts and stop there. Do **not** follow its run-and-iterate loop: it
spawns with-skill and baseline runs straight away, and in a KBD phase nothing
runs before the boundary. This creator still owns everything skill-creator does
not produce: phase controllers, agents, schemas, scripts, hooks, the plugin
manifest and platform outputs.

**When it is absent**, say so in one line and carry on: draft the description
and body from the templates, and write the cases by the rules in the
`skill-certification` skill's `references/writing-cases.md`. A skill without
cases can be created, but it cannot be certified above C1.

## skill-certification: the gate

Certification runs once, at the boundary, after the static validator and the
adversarial review have passed.

```bash
# Locate the certifier: the pack plugin (nested or flat), then an installed copy.
for _c in \
  "${CLAUDE_PLUGIN_ROOT:-}/skills/testing/skill-certification/scripts/certify.mjs" \
  "${CLAUDE_PLUGIN_ROOT:-}/skills/skill-certification/scripts/certify.mjs" \
  "${PLUGIN_ROOT:-}/skills/skill-certification/scripts/certify.mjs" \
  "$HOME/.agents/skills/skill-certification/scripts/certify.mjs" \
  "$HOME/.claude/skills/skill-certification/scripts/certify.mjs"; do
  [ -f "$_c" ] && { CERTIFY="$_c"; break; }
done
[ -n "${CERTIFY:-}" ] || { echo "skill-certification is not installed"; exit 3; }

# Static levels. Free.
node "$CERTIFY" "<skill_dir>" --out "<phase_dir>/certification"
```

If the certifier is not installed, report the skill as **uncertified** and say
what that leaves unknown. Do not substitute `validate-skill.sh` and call the
result certified: that script covers part of the static level only.

**Trigger and outcome levels cost money.** Tell the operator the mode, the run
count and the cost cap, and wait for a yes:

```bash
node "$CERTIFY" "<skill_dir>" --out "<phase_dir>/certification" \
  --run both --plugin "<plugin-dir>" --runs 5 --max-cost-usd 8
```

`<plugin-dir>` is the plugin users will install the skill through. In the skill
pack that is `dist/plugins/claude/prometheus-skill-pack` after `npm run build`,
or the skill's domain plugin if that is how it will be installed. The catalogue
changes the trigger result, so name the one that was used.

If the operator declines the runs, the skill holds the static level. Say so; do
not describe it as certified for triggering or outcome.

## When a level fails

Read `certificate.json`. For each entry in `decisions`, put the decision to the
operator. Do not choose for them, and do not loop on your own.

Show, for each failed or incomplete level:

1. **What failed**: the `failed` list.
2. **The proposed fix**: the `proposedFix` list.
3. **The impact of each choice**: the three `options`.

| Choice | What you do next |
|---|---|
| **Fix** | Apply the proposed fix, then re-run only the affected level. A static fix re-runs the static levels. A description change re-runs the trigger cases. A body or grader change re-runs both arms |
| **Accept** | Re-run the certifier with `--accept <level>=<the operator's reason>` so the exception is in the certificate. Report the skill at the level it holds |
| **Abandon** | Stop. Leave the files for the operator to remove; say where they are |

Ask once per level, with the choices side by side. If `/pmpo-elicit` is
available use it; otherwise ask inline.

A repair is bounded: after two fix rounds on the same level, stop and put the
remaining findings to the operator again instead of trying a third time.

## What to report at the end

- The level held, and each level's result, including `NOT_EXERCISED` and
  `INCOMPLETE` as they are.
- The catalogue the trigger level was measured in.
- Any recorded exception, in the operator's words.
- Where the certificate and report are.
