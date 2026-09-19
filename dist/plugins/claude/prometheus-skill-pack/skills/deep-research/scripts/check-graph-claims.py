#!/usr/bin/env python3
"""check-graph-claims.py — validate graph.json, claim labels and citation coverage.

One of three programs used by check-research-package.sh. Kept as sibling
files rather than heredocs inside the wrapper: the heredoc form hangs
indefinitely on some hosts (observed 2026-09-09), and this script gates
stage 10, so a hang there blocks every package validation.
"""

import json, os, re, sys
pkg = os.environ["PKG"]; rc = 0; noted = False
LABELS = {"verified", "unverified", "blocked", "inferred"}
def fail(msg):
    global rc; rc = 1; print(f"[check-research-package] FAIL  {msg}", file=sys.stderr)
def ok(msg): print(f"[check-research-package] PASS  {msg}")
def note(msg):
    global noted; noted = True; print(f"[check-research-package] NOTE  {msg}")
def load(name):
    p = os.path.join(pkg, name)
    if not os.path.exists(p): return None
    try: return json.load(open(p))
    except Exception as e:
        fail(f"{name} is not valid JSON: {e}"); return "BAD"

graph = load("graph.json"); cp = load("checkpoint.json"); m = json.load(open(os.environ["MANIFEST"]))
legacy = isinstance(graph, dict) and "claims" not in graph and "nodes" in graph
if graph is None:
    note("graph.json absent; label checks skipped")
elif graph == "BAD":
    pass
elif legacy:
    note("graph.json is the legacy {nodes, edges} shape (no labels); accepted until change-rah-010 lands; derivation falls back to sources/credibility.json claims, else unverified")
else:
    # Schema validation (python jsonschema when importable, else structural).
    schema = json.load(open(os.environ["GRAPH_SCHEMA"]))
    try:
        import jsonschema
        errs = list(jsonschema.Draft202012Validator(schema).iter_errors(graph))
        if errs:
            claims = graph.get("claims", [])
            for e in errs[:10]:
                path = list(e.path)
                where = "/".join(str(p) for p in path) or "<root>"
                # Name the claim, not only its index, so the failure is actionable.
                if len(path) >= 2 and path[0] == "claims" and isinstance(path[1], int) and path[1] < len(claims) and isinstance(claims[path[1]], dict):
                    where = f"claims[{path[1]}] ({claims[path[1]].get('id')})" + ("/" + "/".join(str(p) for p in path[2:]) if len(path) > 2 else "")
                fail(f"graph.json {where}: {e.message}")
        else:
            ok(f"graph.json validates against research-graph.schema.json ({len(graph.get('claims', []))} claims)")
    except ImportError:
        for i, c in enumerate(graph.get("claims", [])):
            for k in ("id", "text", "label", "critical", "confidence", "sources", "contradicts"):
                if k not in c: fail(f"graph.json claims[{i}] ({c.get('id')}) missing {k}")
            if c.get("label") not in LABELS: fail(f"graph.json claims[{i}] ({c.get('id')}) label {c.get('label')!r} not in {sorted(LABELS)}")
        note("graph.json structural check only (jsonschema unavailable)")
    # Label-specific rules the schema cannot express.
    for i, c in enumerate(graph.get("claims", [])):
        if c.get("label") in ("verified", "blocked") and not c.get("evidence"):
            fail(f"graph.json claims[{i}] ({c.get('id')}) is {c.get('label')} but carries no evidence")
        if c.get("label") != "inferred" and not c.get("sources"):
            fail(f"graph.json claims[{i}] ({c.get('id')}) has no sources but is not inferred")

# citations.json and contradictions.json labels
cit = load("citations.json")
if isinstance(cit, dict):
    entries = cit.get("citations", [])
    bad = [c.get("id") for c in entries if c.get("label") not in LABELS]
    if bad: fail(f"citations.json entries without a valid label: {bad}")
    elif entries: ok(f"citations.json labels valid ({len(entries)})")
con = load("contradictions.json")
if isinstance(con, dict):
    entries = con.get("contradictions", [])
    bad = [c.get("id") for c in entries if c.get("label") not in LABELS]
    if bad: fail(f"contradictions.json entries without a valid label: {bad}")
    elif entries: ok(f"contradictions.json labels valid ({len(entries)})")

# Derivation rule (okf-research-format.md, Verification Status Rules).
if isinstance(graph, dict) and isinstance(cp, dict):
    claims = [] if legacy else graph.get("claims", [])
    if not claims:
        # No graph (direct or shallow scale skips stage 07): the labels live in
        # stage 05's credibility.json and every labelled claim counts as critical.
        cred = load("sources/credibility.json")
        if isinstance(cred, dict):
            for s in cred.get("verified_sources", []) or []:
                for c in s.get("claims", []) or []:
                    claims.append({"label": c.get("label"), "critical": True})
            if claims: note(f"no graph claims; derivation uses {len(claims)} labelled claim(s) from sources/credibility.json")
    stages = cp.get("stages_completed", [])
    integ = cp.get("integrations", {}) or {}
    review = (cp.get("review") or {}).get("verdict", "")
    # Stage 09 gate values: checkpoint.json first, report.md frontmatter when absent.
    fm = {}
    rp = os.path.join(pkg, "report.md")
    if os.path.exists(rp):
        mm = re.match(r"^---\n(.*?)\n---\n", open(rp).read(), re.S)
        if mm:
            for line in mm.group(1).splitlines():
                if ":" in line:
                    k, v = line.split(":", 1); fm[k.strip()] = v.strip().strip('"')
    def gate_value(key):
        v = cp.get(key)
        if v is None:
            raw = fm.get(key)
            if raw in (None, "", "null", "~"): return None
            try: return float(raw)
            except ValueError: return None
        return v
    grade = gate_value("feynman_grade")
    misconceptions_absent = gate_value("misconceptions_absent")
    # "no claim carries a label" is unverified (okf-research-format.md), so
    # count only present labels, not claim entries.
    labels = [c.get("label") for c in claims if c.get("label") is not None]
    critical = [c for c in claims if c.get("critical") is True]
    if "05" not in stages or not labels:
        derived = "unverified"
    elif (any(c.get("label") != "verified" for c in critical)
          or any(l in ("blocked", "unverified") for l in labels)
          or review == "BLOCK"
          or cp.get("blocked_review") is not None
          or (cp.get("scale") == "full" and not integ.get("adversarial_review_used"))
          or not integ.get("feynman_gate_used")
          or (grade or 0) < 0.7
          or (misconceptions_absent or 0) < 1.0):
        derived = "partial"
    else:
        derived = "verified"
    declared = m.get("verification_status")
    if declared != derived:
        fail(f"verification_status declared {declared!r} but derived {derived!r} from claim labels and checkpoint")
    else:
        ok(f"verification_status {declared} agrees with the derivation rule")
sys.exit(rc if rc else (3 if noted else 0))
