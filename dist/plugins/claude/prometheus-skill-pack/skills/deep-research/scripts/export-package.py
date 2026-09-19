#!/usr/bin/env python3
"""export-package.py — stage 10 package export.

Invoked by export-package.sh, which passes PKG_DIR, PACKAGE_ID, SCHEMA and
FABRICATED through the environment. Kept as a sibling file rather than a
heredoc inside the wrapper: the heredoc form hangs indefinitely on some hosts
(observed 2026-09-09). Same convention as score-sources.py beside
verify-sources.sh.
"""

import json, os, re, sys, glob, datetime

pkg = os.environ["PKG_DIR"]; package_id = os.environ["PACKAGE_ID"]
defaulted = os.environ.get("FABRICATED", "").split()

def load_json(name):
    p = os.path.join(pkg, name)
    if not os.path.exists(p):
        return None
    try:
        with open(p) as f:
            return json.load(f)
    except Exception as e:
        print(f"[export-package] WARN: {name} is not valid JSON ({e}); treated as absent", file=sys.stderr)
        return None

def frontmatter(path):
    if not os.path.exists(path):
        return {}
    text = open(path).read()
    m = re.match(r"^---\n(.*?)\n---\n", text, re.S)
    if not m:
        return {}
    out = {}
    for line in m.group(1).splitlines():
        if ":" in line:
            k, v = line.split(":", 1)
            out[k.strip()] = v.strip().strip('"')
    return out

def num_or_null(v):
    try:
        return None if v in (None, "", "null") else float(v)
    except ValueError:
        return None

cp = load_json("checkpoint.json") or {}
fm = frontmatter(os.path.join(pkg, "report.md"))
registry = load_json("sources/registry.json")
graph = load_json("graph.json") or {}
contra = load_json("contradictions.json") or {}

def get(key, default, source):
    if key in source:
        return source[key]
    defaulted.append(key)
    return default

now = datetime.datetime.now(datetime.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")

# Slug and provenance sidecar name from the package id (<slug>-<yyyymmdd>-<4hex>).
m = re.match(r"^(.*)-\d{8}-[0-9a-f]{4}$", package_id)
slug = m.group(1) if m else package_id
if not m:
    defaulted.append("package_id(shape)")
prov_name = f"{slug}.provenance.md"
prov_path = os.path.join(pkg, prov_name)
verdict = None
if os.path.exists(prov_path):
    pm = re.search(r"\*\*Verification:\*\*\s*(PASS WITH NOTES|PASS|BLOCKED)", open(prov_path).read())
    verdict = pm.group(1) if pm else None
if verdict is None:
    defaulted.append("verification_verdict")
    verdict = "BLOCKED"

sources_count = len(registry) if isinstance(registry, list) else (len(registry.get("sources", [])) if isinstance(registry, dict) else 0)
if registry is None:
    sources_count = len(glob.glob(os.path.join(pkg, "sources", "chunk-*.json")))
    defaulted.append("sources_count(from chunks)")
claims = graph.get("claims", []) if isinstance(graph, dict) else []
contras = contra.get("contradictions", []) if isinstance(contra, dict) else (contra if isinstance(contra, list) else [])
resolved = sum(1 for c in contras if c.get("resolved") is True)

stages = get("stages_completed", [], cp)
if not isinstance(stages, list):
    stages = []
integ = cp.get("integrations", {}) if isinstance(cp.get("integrations"), dict) else {}

vstatus = fm.get("verification_status") or get("verification_status", None, cp)
if vstatus not in ("verified", "partial", "unverified"):
    if vstatus is not None:
        print(f"[export-package] WARN: verification_status {vstatus!r} not in enum; using unverified", file=sys.stderr)
    defaulted.append("verification_status")
    vstatus = "unverified"

manifest = {
    "format": "research-package",
    "format_version": "2.0.0",
    "okf_type": "research-session",
    "package_id": package_id,
    "job_id": get("job_id", package_id, cp),
    "query": get("query", fm.get("query", "(query not recorded)"), cp),
    "depth": get("depth", "deep", cp),
    "scale": get("scale", "full", cp),
    "created_at": get("created_at", now, cp),
    "completed_at": get("completed_at", now if "10" in stages else None, cp),
    "stages_completed": stages,
    "sources_count": sources_count,
    "claims_count": len(claims),
    "confidence": num_or_null(fm.get("confidence", cp.get("confidence"))),
    "verification_status": vstatus,
    "verification_verdict": verdict,
    "feynman_grade": num_or_null(fm.get("feynman_grade", cp.get("feynman_grade"))),
    "feynman_gate_used": bool(integ.get("feynman_gate_used", fm.get("feynman_grade") not in (None, "", "null"))),
    "misconceptions_absent": num_or_null(fm.get("misconceptions_absent", cp.get("misconceptions_absent"))),
    "contradictions_detected": len(contras),
    "contradictions_resolved": resolved,
    "contradictions_unresolved": len(contras) - resolved,
    "surreal_memory_used": bool(integ.get("surreal_memory_used", False)),
    "sycophancy_correction_used": bool(integ.get("sycophancy_correction_used", False)),
    "adversarial_review_used": bool(integ.get("adversarial_review_used", False)),
    "citation_style": get("citation_style", "APA", cp),
    "kb_ids": get("kb_ids", [], cp),
    "model_routing": get("model_routing", {}, cp),
    "files": {
        "report": "report.md",
        "provenance": prov_name,
        "plan": "plan.md",
        "graph": "graph.json",
        "citations": "citations.json",
        "contradictions": "contradictions.json",
        "index": "index.md",
        "sources_dir": "sources/",
    },
}
if os.path.exists(os.path.join(pkg, "sensitivity.json")):
    manifest["files"]["sensitivity"] = "sensitivity.json"
if "confidence" not in fm and "confidence" not in cp: defaulted.append("confidence")
if "feynman_grade" not in fm and "feynman_grade" not in cp: defaulted.append("feynman_grade")
if "misconceptions_absent" not in fm and "misconceptions_absent" not in cp: defaulted.append("misconceptions_absent")
for k in ("surreal_memory_used", "sycophancy_correction_used", "adversarial_review_used"):
    if k not in integ: defaulted.append(k)

with open(os.path.join(pkg, "manifest.json"), "w") as f:
    json.dump(manifest, f, indent=2); f.write("\n")

if defaulted:
    seen = []
    for d in defaulted:
        if d not in seen:
            seen.append(d)
    print("[export-package] defaulted: " + ", ".join(seen), file=sys.stderr)

# Validate against the schema when jsonschema is importable; otherwise a structural check.
schema_path = os.environ["SCHEMA"]
try:
    import jsonschema
    schema = json.load(open(schema_path))
    errs = list(jsonschema.Draft202012Validator(schema).iter_errors(manifest))
    if errs:
        for e in errs:
            print(f"[export-package] SCHEMA: {'/'.join(str(p) for p in e.path) or '<root>'}: {e.message}", file=sys.stderr)
        sys.exit(2)
    print("[export-package] manifest validates against research-manifest.schema.json", file=sys.stderr)
except ImportError:
    # Structural fallback: full key set plus declared type, const, and enum per field.
    schema = json.load(open(schema_path))
    props = schema["properties"]
    problems = [f"missing required {k}" for k in schema["required"] if k not in manifest]
    problems += [f"unexpected key {k}" for k in manifest if k not in props]
    tmap = {"string": str, "integer": int, "number": (int, float), "boolean": bool,
            "array": list, "object": dict, "null": type(None)}
    for k, v in manifest.items():
        p = props.get(k, {})
        if "const" in p and v != p["const"]:
            problems.append(f"{k}: expected const {p['const']!r}")
        if "enum" in p and v not in p["enum"]:
            problems.append(f"{k}: {v!r} not in enum")
        t = p.get("type")
        if t:
            ts = t if isinstance(t, list) else [t]
            if not any(isinstance(v, tmap[x]) and not (x in ("integer", "number") and isinstance(v, bool)) for x in ts):
                problems.append(f"{k}: type {type(v).__name__} not in {ts}")
    if problems:
        for pr in problems:
            print(f"[export-package] STRUCTURAL: {pr}", file=sys.stderr)
        sys.exit(2)
    print("[export-package] PASS WITH NOTES: jsonschema unavailable, structural check only", file=sys.stderr)

# index.md
title = fm.get("title") or f"Research Package: {package_id}"
index = f"""# {title}

**Package:** `{package_id}`
**Created:** {manifest['created_at']}
**Completed:** {manifest['completed_at'] or 'not completed'}
**Confidence:** {manifest['confidence'] if manifest['confidence'] is not None else 'n/a'}
**Verification:** {manifest['verification_status']} ({manifest['verification_verdict']})
**Stages completed:** {' '.join(stages) if stages else 'none recorded'}

## Contents

- [Report](report.md) — Full research synthesis
- [Provenance]({prov_name}) — Sources consulted, accepted, rejected; verdict
- [Plan](plan.md) — Research plan with task ledger, verification log, decision log
- [Graph](graph.json) — Knowledge graph
- [Citations](citations.json) — Source bibliography
- [Contradictions](contradictions.json) — Contradiction log
- [Sources](sources/) — Retrieved content and registry
- [Manifest](manifest.json) — Package metadata (schema-validated)
"""
with open(os.path.join(pkg, "index.md"), "w") as f:
    f.write(index)
