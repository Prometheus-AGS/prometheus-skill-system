#!/usr/bin/env bash
# Generated from verification.md for change-tlm-004-hook-timeout-seconds. Run by `kbd-apply verify` via tasks.json.
# Repository files are checked in the change's worktree; phase evidence lives in the main checkout.
set -euo pipefail
ROOT="${TLM_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tlm-004}"
cd "$ROOT"
# Base branch: must contain origin/main 20d97f2 (PRs #111, #121-#123).
git merge-base --is-ancestor 20d97f2 HEAD || { echo "base branch does not contain 20d97f2" >&2; exit 1; }
node -e 'const r=f=>JSON.parse(require("fs").readFileSync(f,"utf8")).hooks;const m=h=>{const o={};for(const g of Object.values(h).flat())for(const x of g.hooks){const s=x.args?x.args.join(" "):x.command;const id=s.match(/--hook (\S+)/)[1];o[id]=x.timeout}return o};const a=m(r("hooks/hooks.json")),b=m(r("hooks/codex-hooks.json"));for(const [id,t] of Object.entries(b)){if(t!==undefined&&(t>600||t<1))process.exit(1);if(a[id]!==t)process.exit(2)}for(const t of Object.values(a))if(t!==undefined&&(t>600||t<1))process.exit(3)'
! grep -rn "CODEX_MIN_HOOK_TIMEOUT_MS" scripts
tlm4_bak="$(mktemp)"; cp shared/harnesses/hook-contract.json "$tlm4_bak"; trap 'cp "$tlm4_bak" shared/harnesses/hook-contract.json; rm -f "$tlm4_bak"' EXIT
python3 -c 'import json;p="shared/harnesses/hook-contract.json";d=json.load(open(p));d["events"][0]["hooks"][0]["timeout"]=5000;open(p,"w").write(json.dumps(d,indent=2)+"\n")'
if node scripts/generate-harness-adapters.js >/dev/null 2>&1; then echo "negative control failed: generator accepted a 5000-second timeout" >&2; exit 1; fi
cp "$tlm4_bak" shared/harnesses/hook-contract.json; rm -f "$tlm4_bak"; trap - EXIT
node scripts/generate-harness-adapters.js >/dev/null
node scripts/generate-harness-adapters.js && node scripts/generate-skill-system-distribution.js && find hooks shared/harnesses/generated shared/scripts/generated dist -type f -print0 | sort -z | xargs -0 shasum -a 256 > /tmp/tlm4-a && node scripts/generate-harness-adapters.js && node scripts/generate-skill-system-distribution.js && find hooks shared/harnesses/generated shared/scripts/generated dist -type f -print0 | sort -z | xargs -0 shasum -a 256 | diff - /tmp/tlm4-a
npm run validate:harness-adapters && node scripts/tests/hook-dispatch.test.mjs && npm run check:distribution && npm run validate:codex
! grep -n -F -e '`timeout` is milliseconds in Codex' -e '5000 ms floor' -e 'Codex reads `timeout` as milliseconds' -e 'CODEX_MIN_HOOK_TIMEOUT_MS' docs/codex-plugin.md CLAUDE.md scripts/lib/hook-config.js scripts/generate-harness-adapters.js
echo "verify OK: change-tlm-004-hook-timeout-seconds"
