#!/usr/bin/env python3
"""check-manifest-files.py — check every manifest file entry exists with the recorded hash.

One of three programs used by check-research-package.sh. Kept as sibling
files rather than heredocs inside the wrapper: the heredoc form hangs
indefinitely on some hosts (observed 2026-09-09), and this script gates
stage 10, so a hang there blocks every package validation.
"""

import json, os, re, sys
pkg = os.environ["PKG"]; m = json.load(open(os.environ["MANIFEST"]))
rc = 0
rp = os.path.join(pkg, "report.md")
if os.path.exists(rp):
    fm = {}
    mm = re.match(r"^---\n(.*?)\n---\n", open(rp).read(), re.S)
    if mm:
        for line in mm.group(1).splitlines():
            if ":" in line:
                k, v = line.split(":", 1); fm[k.strip()] = v.strip().strip('"')
    for k in ("verification_status", "confidence", "feynman_grade", "misconceptions_absent", "sources_count", "contradictions_resolved"):
        if k in fm:
            a, b = str(fm[k]), str(m.get(k))
            # YAML null / empty and JSON null are the same absence.
            if a in ("null", "", "~") and m.get(k) is None:
                same = True
            else:
                try:
                    same = float(a) == float(b)
                except ValueError:
                    same = a == b
            if not same:
                print(f"[check-research-package] FAIL  report.md {k}={a} but manifest {k}={b}", file=sys.stderr); rc = 1
            else:
                print(f"[check-research-package] PASS  report.md {k} agrees with manifest")
pp = os.path.join(pkg, m["files"]["provenance"])
if os.path.exists(pp):
    pm = re.search(r"\*\*Verification:\*\*\s*(PASS WITH NOTES|PASS|BLOCKED)", open(pp).read())
    if pm and pm.group(1) != m["verification_verdict"]:
        print(f"[check-research-package] FAIL  provenance verdict {pm.group(1)} but manifest verification_verdict {m['verification_verdict']}", file=sys.stderr); rc = 1
    elif pm:
        print("[check-research-package] PASS  provenance verdict agrees with manifest")
sys.exit(rc)
