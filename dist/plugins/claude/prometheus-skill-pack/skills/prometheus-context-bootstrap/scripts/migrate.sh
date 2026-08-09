#!/usr/bin/env bash
# Migrate a Prometheus Base Rules v3 agent file to the bootstrapped structure.
#
# The mechanical part is safe: v3's skeleton has known rule IDs and a known
# destination for each. The unsafe part is everything a project ADDED to v3
# (G-2 permits stricter local rules) — that cannot be classified by a script,
# so it is never silently dropped and never silently kept. It is listed, with
# line numbers into the archive, for a human to re-place.
#
# Default is a report. Nothing is written without --apply.

set -euo pipefail

SKILL_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REF="$SKILL_ROOT/references"
MAP="$REF/migration-map.tsv"

die()  { printf 'migrate: %s\n' "$*" >&2; exit 1; }

project_path="."
apply=0
profile="mixed"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --path)    project_path="${2:?--path requires a value}"; shift 2 ;;
    --profile) profile="${2:?--profile requires a value}"; shift 2 ;;
    --apply)   apply=1; shift ;;
    -h|--help)
      cat <<'USAGE'
Usage: migrate.sh [--path <root>] [--profile mixed|strict|lean] [--apply]

Reports how a v3 base-rules file maps onto the bootstrapped structure.
Without --apply it writes nothing.

With --apply it: archives the original under .prometheus/knowledge/,
writes .prometheus/MIGRATION-REPORT.md, then runs bootstrap.sh to
generate the new AGENTS.md.

Exit: 0 ok, 1 usage or no source file, 3 no v3 content detected
USAGE
      exit 0 ;;
    *) die "unknown flag: $1 (try --help)" ;;
  esac
done

[[ -d "$project_path" ]] || die "not a directory: $project_path"
project_path="$(cd "$project_path" && pwd)"
[[ -f "$MAP" ]] || die "migration map missing: $MAP"

# Find the source. Prefer AGENTS.md; fall back to CLAUDE.md if it is a real file.
src=""
for c in "$project_path/AGENTS.md" "$project_path/CLAUDE.md" "$project_path/AGENT_BASE_RULES.md"; do
  [[ -f "$c" && ! -L "$c" ]] && { src="$c"; break; }
done
[[ -n "$src" ]] || die "no AGENTS.md, CLAUDE.md, or AGENT_BASE_RULES.md at $project_path"

# Detect v3. Two independent signals, so a file that merely mentions a rule ID
# is not mistaken for the constitution itself.
sig=0
grep -qE 'THE CONSTITUTION|Prometheus Base Rules Set' "$src" && sig=$((sig+1))
ids_found="$(grep -cE '^\*\*[A-G]-[0-9]+ ·' "$src" 2>/dev/null; :)"
[[ "${ids_found:-0}" -ge 5 ]] && sig=$((sig+1))

if [[ "$sig" -lt 1 ]]; then
  printf 'No v3 base-rules content detected in %s\n' "${src#$project_path/}"
  printf 'Nothing to migrate. Run bootstrap.sh directly.\n'
  exit 3
fi

today="$(date -u +%Y-%m-%d)"
archive_rel=".prometheus/knowledge/AGENTS.pre-migration-${today}.md"
archive="$project_path/$archive_rel"
report="$project_path/.prometheus/MIGRATION-REPORT.md"

src_words="$(wc -w < "$src" | tr -d ' ')"

# ---- coverage: every ID in the map, present or absent in the source --------
mapped=0; absent=0
cov_tmp="$(mktemp)"; trap 'rm -f "$cov_tmp" "$cov_tmp".*' EXIT

while IFS=$'\t' read -r id dest note; do
  [[ -z "${id:-}" || "$id" == \#* ]] && continue
  case "$id" in
    APPENDIX-*) pat="APPENDIX ${id#APPENDIX-}" ;;
    §*)         pat="$id" ;;
    *)          pat="$id" ;;
  esac
  if grep -qF "$pat" "$src" 2>/dev/null; then
    printf '| `%s` | present | %s | %s |\n' "$id" "$dest" "${note:-}" >> "$cov_tmp"
    mapped=$((mapped+1))
  else
    printf '| `%s` | absent | %s | %s |\n' "$id" "$dest" "${note:-}" >> "$cov_tmp"
    absent=$((absent+1))
  fi
done < "$MAP"

# ---- tool-owned managed regions -------------------------------------------
# Regions like <!-- agent-rules:start v1 --> ... <!-- agent-rules:end --> are
# written by other tools (kbd-inject-agent-rules, the Zed workspace tool). They
# are self-delimited, so they can be carried over verbatim rather than handed to
# a human. An earlier version scanned only '^## ' headings, so these were
# indistinguishable from prose and were silently orphaned.
regions_tmp="$cov_tmp.regions"
region_names_tmp="$cov_tmp.rnames"
: > "$regions_tmp"; : > "$region_names_tmp"

grep -oE '<!-- [a-z][a-z0-9-]*:(start|begin)' "$src" 2>/dev/null \
  | sed -E 's/<!-- ([a-z0-9-]+):(start|begin)/\1/' \
  | grep -v '^prometheus-base$' | sort -u > "$region_names_tmp" || true
region_count="$(wc -l < "$region_names_tmp" | tr -d ' ')"

while IFS= read -r rn; do
  [[ -z "$rn" ]] && continue
  awk -v n="$rn" '
    $0 ~ ("<!-- " n ":(start|begin)") { inb=1 }
    inb { print }
    $0 ~ ("<!-- " n ":end -->")       { inb=0 }
  ' "$src" >> "$regions_tmp"
  printf '\n' >> "$regions_tmp"
done < "$region_names_tmp"

# ---- residue: headings that are neither canonical v3 nor inside a region ---
# Canonical v3 headings are "## §X." or "## APPENDIX X". Anything else at H2 is
# project-added content that only a human can place — except headings that live
# inside a tool-owned region, which travel with that region.
res_tmp="$cov_tmp.res"
region_headings="$cov_tmp.rh"
grep -E '^## ' "$regions_tmp" 2>/dev/null | sed 's/^## //' | sort -u > "$region_headings" || true

grep -nE '^## ' "$src" 2>/dev/null \
  | grep -vE '^[0-9]+:## (§|APPENDIX)' \
  | grep -vE '^[0-9]+:## Prometheus Base Rules Set' > "$res_tmp.all" || true

: > "$res_tmp"
while IFS= read -r l; do
  [[ -z "$l" ]] && continue
  h="$(printf '%s' "${l#*:}" | sed 's/^## //')"
  grep -qxF "$h" "$region_headings" 2>/dev/null || printf '%s\n' "$l" >> "$res_tmp"
done < "$res_tmp.all"
residue="$(wc -l < "$res_tmp" | tr -d ' ')"

# ---------------------------------------------------------------- report ----
build_report() {
  cat <<EOF
# Migration report — v3 base rules to bootstrapped structure

Generated ${today}. Source: \`${src#$project_path/}\` (${src_words} words).
Archive: \`${archive_rel}\`

The archive is the authority for anything below. Nothing was deleted.

## Coverage

${mapped} of the mapped rule IDs were present in the source; ${absent} were not.
"Present" means the ID appeared in the source and its content is covered by the
destination. It does not mean the destination is byte-identical — most rules
were condensed, and several moved from prose to enforcement.

| v3 ID | In source | Destination | Note |
|---|---|---|---|
$(cat "$cov_tmp")

## What became enforcement rather than prose

These stopped being advisory. A hook now blocks the action, so the prose that
asked for the same behavior is redundant and was not carried over.

| v3 ID | Enforced by |
|---|---|
| A-9 tier discipline | \`.claude/hooks/tier-guard.sh\` |
| A-10 single-writer | \`.claude/hooks/single-writer.sh\` |
| E-1, E-5 sycophancy gate | \`.claude/hooks/sycophancy-gate.sh\` |
| E-2 critic isolation | \`.claude/agents/artifact-critic.md\` |
| §0, F-4 bootstrap and re-anchor | \`.claude/hooks/reanchor.sh\` |

Verify they actually fire. A hook that is installed but not wired into
\`settings.json\` enforces nothing, and \`verify.sh\` checks exactly that.

## Project-added content — REQUIRES A HUMAN

${residue} heading(s) in the source are not part of the canonical v3 skeleton.
A script cannot tell whether these are project rules that must survive or notes
that have expired. They were **not** carried into the new file.

Read each one in the archive and decide: move it into the managed region's
project section, into a \`.claude/rules/\` file, into a skill, or drop it.

EOF
  if [[ "$residue" -gt 0 ]]; then
    echo '| Archive line | Heading |'
    echo '|---|---|'
    while IFS= read -r l; do
      [[ -z "$l" ]] && continue
      printf '| %s | %s |\n' "${l%%:*}" "$(printf '%s' "${l#*:}" | sed 's/^## //')"
    done < "$res_tmp"
  else
    echo 'None found. The source appears to be unmodified v3.'
  fi
  cat <<EOF

## Verify the migration

\`\`\`bash
bash scripts/verify.sh --path .
\`\`\`

Then re-run a fixed task set and compare pass rate against the archive-era
baseline. Word count going down is not evidence that anything improved.
EOF
}

# ----------------------------------------------------------------- output ---
printf 'Source: %s (%s words)\n' "${src#$project_path/}" "$src_words"
printf 'v3 detected: %s rule IDs, %s signal(s)\n' "$ids_found" "$sig"
printf 'Mapped IDs present: %s   absent: %s\n' "$mapped" "$absent"
printf 'Tool-owned regions carried over verbatim: %s\n' "$region_count"
[[ "$region_count" -gt 0 ]] && sed 's/^/  /' "$region_names_tmp"
printf 'Project-added headings needing a human: %s\n' "$residue"

# A second agent file carrying v3 is the same hazard. Detect it now so the
# apply path can archive and remove it rather than leaving a live constitution.
second=""
for c in "$project_path/AGENTS.md" "$project_path/CLAUDE.md"; do
  [[ "$c" == "$src" ]] && continue
  [[ -f "$c" && ! -L "$c" ]] || continue
  cids="$(grep -cE '^\*\*[A-G]-[0-9]+ ·' "$c" 2>/dev/null; :)"
  if [[ "${cids:-0}" -ge 5 ]]; then
    second="$c"
    printf 'Second agent file also carries v3: %s (%s rule IDs) — will be archived and removed\n' \
      "${c#$project_path/}" "$cids"
  fi
done

if [[ "$residue" -gt 0 ]]; then
  echo
  echo 'These will NOT be carried over automatically:'
  sed 's/^/  /' "$res_tmp"
fi

if [[ "$apply" != "1" ]]; then
  echo
  echo 'DRY RUN — nothing written. Re-run with --apply to migrate.'
  exit 0
fi

mkdir -p "$(dirname "$archive")"
cp "$src" "$archive"
build_report > "$report"
rm -f "$project_path/AGENTS.md"

if [[ -n "$second" ]]; then
  cp "$second" "${archive%.md}.$(basename "$second" .md).md"
  rm -f "$second"
  echo "Archived and removed second v3 file: ${second#$project_path/}"
fi

echo
echo "Archived: $archive_rel"
echo "Report:   .prometheus/MIGRATION-REPORT.md"
echo
bash "$SKILL_ROOT/scripts/bootstrap.sh" --path "$project_path" --profile "$profile"

# Carry tool-owned regions across, verbatim, below the managed region.
if [[ "$region_count" -gt 0 && -s "$regions_tmp" ]]; then
  {
    printf '\n'
    cat "$regions_tmp"
  } >> "$project_path/AGENTS.md"
  echo
  echo "Carried $region_count tool-owned region(s) into AGENTS.md verbatim:"
  sed 's/^/  /' "$region_names_tmp"
  echo "Their owning tools can re-inject over them; markers are intact."
fi

echo
echo "Migration applied. Read .prometheus/MIGRATION-REPORT.md before committing —"
echo "${residue} project-added section(s) still need placing by hand."
