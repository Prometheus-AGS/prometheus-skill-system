# Reflect Phase — PMPO Skill Creator

You are the Reflect phase controller. Validate the generated skill against the agentskills.io spec and quality standards.

## Model Selection

Resolve `creator-reflect` through the project model policy. This phase requires
`frontier`; if no policy exists, use the frontier fallback in
`../references/model-routing.md`.

## Objective

Evaluate the generated skill for completeness, correctness, and spec compliance. Produce a validation report that determines whether to loop or terminate.

## Inputs

- `skill_spec` from Specify phase
- `skill_plan` from Plan phase
- `execution_result` from Execute phase

## Procedure

### Step 1: agentskills.io Spec Validation

Validate `SKILL.md` against the spec:

| Check                  | Requirement                                                | Severity |
| ---------------------- | ---------------------------------------------------------- | -------- |
| Frontmatter present    | `---` delimiters with YAML                                 | **FAIL** |
| `name` field           | ≤64 chars, lowercase + hyphens, no leading/trailing hyphen | **FAIL** |
| `description` field    | ≤1024 chars, non-empty                                     | **FAIL** |
| Body content           | Non-empty markdown below frontmatter                       | **FAIL** |
| Line count             | ≤500 lines recommended                                     | **WARN** |
| Progressive disclosure | References use relative paths                              | **WARN** |

### Step 2: JSON Schema Validation

For every `.schema.json` file:

```bash
python3 -c "import json; json.load(open('$f'))"
```

Check:

- Valid JSON syntax
- Has `$schema` field
- Has `type` field
- Required properties are defined
- No circular references

### Step 3: Cross-Reference Integrity

Extract all file references from prompts and SKILL.md:

```bash
grep -roh 'references/[a-zA-Z0-9/_.-]*' prompts/ SKILL.md | sort -u
```

Verify each resolves to an existing file. **FAIL** on any dangling reference.

### Step 4: Script Validation

For every script in `scripts/`:

| Check            | Requirement                       | Severity |
| ---------------- | --------------------------------- | -------- |
| Shebang line     | Starts with `#!/usr/bin/env bash` | **FAIL** |
| Strict mode      | Contains `set -euo pipefail`      | **WARN** |
| Executable       | `[ -x "$f" ]`                     | **FAIL** |
| No syntax errors | `bash -n "$f"`                    | **FAIL** |

### Step 5: Hooks Validation (if present)

Validate `hooks/hooks.json`:

- Valid JSON
- Each hook has `event` field
- Script paths use `${CLAUDE_PLUGIN_ROOT}` or relative
- No references to nonexistent scripts

### Step 6: Plugin Manifest Validation (if claude-code)

Validate `.claude-plugin/plugin.json`:

- Has `name` field
- Has `description` field
- Has `version` field (semver)

### Step 7: Completeness Check

Compare generated files against `skill_plan.file_map`:

```yaml
completeness:
  planned: integer # Files in plan
  generated: integer # Files on disk
  missing: string[] # Planned but not generated
  extra: string[] # Generated but not planned
  coverage: float # generated / planned
```

**FAIL** if coverage < 100%.

### Step 8: PMPO Loop Integrity (standard/full tier)

Check that the PMPO loop is complete:

- All planned phases have controllers in `prompts/`
- Meta-controller references all phases in correct order
- Each phase controller has: objective, procedure, output contract, rules
- Agent files exist for all referenced agents

### Step 9: State Management Check (full tier)

Verify state lifecycle:

- `state-resolve-provider.sh` — provider resolution
- `state-init.sh` — creates initial state with UUID
- `state-checkpoint.sh` — accepts skill name and phase
- `state-finalize.sh` — archives and updates registry
- State directory structure documented in meta-controller

### Step 10: Sycophancy Check on Generated Content

Run sycophancy detection on the generated SKILL.md instruction body:

| Check                                               | Pattern | Severity |
| --------------------------------------------------- | ------- | -------- |
| Unprompted affirmation in instructions              | S-01    | **WARN** |
| No "when NOT to use" section                        | S-03    | **WARN** |
| No edge cases or failure modes                      | S-03    | **FAIL** |
| Self-congratulatory language                        | S-04    | **WARN** |
| Instructions > 500 words with no analytical density | S-07    | **WARN** |

If the Execute phase ran a sycophancy correction pass, verify the corrected
content is present (not the uncorrected original). **FAIL** if uncorrected
sycophantic content was written to disk.

### Step 11: Quality Score

Aggregate checks into a quality score:

| Category         | Weight | Score           |
| ---------------- | ------ | --------------- |
| Spec compliance  | 30%    | pass/fail count |
| Schema validity  | 20%    | valid/total     |
| Cross-references | 15%    | resolved/total  |
| Script quality   | 15%    | pass/fail       |
| Completeness     | 20%    | coverage %      |

**Pass threshold**: ≥95% weighted score with zero FAILs.

## Validation Script

Run the automated validation suite:

```bash
bash scripts/validate-skill.sh dist/<skill_name>/
```

This script performs Steps 1–9 automatically and outputs a JSON report.

## Producer-Model Guard (required before any adversarial review)

This phase dispatches an adversarial review, which can only make its
judge≠producer guarantee if the producer's identity is known. Source the shared
resolver and call the guard **before** building a review packet:

```bash
# Portable across harnesses: repo-relative, then Claude Code, then Codex.
for _lib in \
  "$(cd "$(dirname "$0")" && pwd)/../../../../shared/scripts/lib/kbd-model-resolve.sh" \
  "${CLAUDE_PLUGIN_ROOT:-}/shared/scripts/lib/kbd-model-resolve.sh" \
  "${PLUGIN_ROOT:-}/shared/scripts/lib/kbd-model-resolve.sh"; do
  [ -f "$_lib" ] && { . "$_lib"; break; }
done

kbd_require_producer_model || exit 2   # exit 2, no packet, no findings file
```

A non-zero return is **fatal**. Do not log it and continue: a packet built with
an unknown producer yields a findings file claiming cross-model verification that
never took place. Export the real value instead —
`export KBD_PRODUCER_MODEL="claude-opus-5"` — and never a `:-default`, which
would fabricate the identity rather than supply it.

## Step 12: Adversarial Review (`--mode skill`)

Runs **after** `validate-skill.sh` and **before** the Loop Decision below.

The ordering is load-bearing in both directions. Reviewing *before* validation
would spend a judge call on an artifact already known to be malformed — the
validator is a cheap deterministic checklist and belongs first. Reviewing *after*
the loop decision would let a skill be declared finished before anything judged
it, which is exactly the gap this step closes.

```bash
ADV="${CLAUDE_PLUGIN_ROOT}/skills/process/adversarial-review"
REVIEW_DIR="dist/<skill_name>/.review"
mkdir -p "$REVIEW_DIR"

# The guard above must already have passed. Build the manifest-level packet:
# SKILL.md, frontmatter, script inventory, cross-reference map, validator
# output, and the original intent. --intent is what the skill was ASKED to be;
# without it the judge can only check internal consistency, never whether the
# artifact answers the request.
bash "$ADV/scripts/build-review-packet.sh" \
  --mode skill \
  --target "dist/<skill_name>" \
  --intent "<path to the Specify-phase spec>" \
  --out "$REVIEW_DIR/packet.json" || exit 2

bash "$ADV/scripts/dispatch-judge.sh" \
  --packet "$REVIEW_DIR/packet.json" \
  --out "$REVIEW_DIR/findings.json"
```

Read `verdict` and `cross_model_check` from `findings.json`:

| Field | Meaning for this step |
|---|---|
| `verdict: BLOCK` | at least one CRITICAL finding — enter the retry loop below |
| `verdict: PASS` | no CRITICAL findings — proceed to the Loop Decision |
| `cross_model_check: verified-distinct` | the judge provably differed from the producer |
| `cross_model_check: same-model-collision` | the judge WAS the producer — the review proves nothing; treat as unreviewed and report it |

A `PASS` carrying `same-model-collision` is **not** a passing review. Record it
as unreviewed rather than reporting the skill as judged.

## Step 13: CRITICAL Retry Loop (max 2 rounds)

CRITICAL findings **block** the skill from being reported as ready.

Do not track the round count by hand — ask the loop script, which owns the bound
for both creators:

```bash
STATE="$(bash "$ADV/scripts/review-retry-loop.sh" state \
           --findings "$REVIEW_DIR/findings.json" --round "$ROUND")"

case "$STATE" in
  PROCEED)  # no CRITICAL findings — go to the Loop Decision
            ;;
  RETRY)    # fix every CRITICAL finding, ROUND=$((ROUND+1)), re-run Step 12
            ;;
  CAPPED)   # stop reviewing; the artifact is NOT clean
            bash "$ADV/scripts/review-retry-loop.sh" unresolved \
              --findings "$REVIEW_DIR/findings.json" --round "$ROUND" \
              --out "$REFLECTION_OUTPUT"
            ;;
esac
```

`state` exits `0` PROCEED / `3` RETRY / `4` CAPPED, so the branch works from the
exit code alone when that is more convenient than the printed word.

On `CAPPED`, the script appends an `## Unresolved review findings` section
naming every surviving finding, and the skill is reported as **not clean**. The
cap bounds how long the loop runs; it does not resolve what the judge found.
Silently dropping the findings at the cap is the sycophantic outcome this phase
exists to prevent, so the section is emitted by the script rather than left to
the model to remember.

An unreadable or malformed `findings.json` yields `CAPPED`, never `PROCEED` — a
review that cannot be parsed is not a review that passed.

> This 2-round cap is the **retry** bound and is separate from the sycophancy
> screen's rejection cap inside `validate-skill.sh`. `change-arc-007` makes that
> other cap user-overridable; this one is unaffected.

## Output Contract

```yaml
reflection:
  overall_status: pass | fail | warn
  quality_score: float # 0.0 - 1.0
  checks:
    - category: string
      check: string
      status: pass | fail | warn
      message: string
  missing_files: string[]
  failing_checks: integer
  warning_checks: integer
  recommendation: terminate | loop_execute | loop_plan
  fix_instructions: string[] # What to fix if looping
```

## Loop Decision

| Condition                       | Recommendation | Return To               |
| ------------------------------- | -------------- | ----------------------- |
| Zero FAILs, score ≥ 0.95        | `terminate`    | Finalize                |
| FAILs in generated files only   | `loop_execute` | Execute (with fix list) |
| FAILs in architecture/structure | `loop_plan`    | Plan (with constraints) |
| 3+ iterations with no progress  | `terminate`    | Output with warnings    |

## Rules

1. NEVER mark a skill as passing if any check is FAIL
2. Include fix_instructions for EVERY failing check
3. Distinguish between fixable (execute-loop) and structural (plan-loop) issues
4. Run `validate-skill.sh` before reporting — manual checks supplement, not replace
