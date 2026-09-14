#!/usr/bin/env python3
"""check-manifest-schema.py — validate manifest.json against research-manifest.schema.json.

One of three programs used by check-research-package.sh. Kept as sibling
files rather than heredocs inside the wrapper: the heredoc form hangs
indefinitely on some hosts (observed 2026-09-09), and this script gates
stage 10, so a hang there blocks every package validation.
"""

import json, os, sys
schema = json.load(open(os.environ["SCHEMA"]))
try:
    doc = json.load(open(os.environ["MANIFEST"]))
except Exception as e:
    print(f"not valid JSON: {e}"); sys.exit(3)
try:
    import jsonschema
    errs = list(jsonschema.Draft202012Validator(schema).iter_errors(doc))
    if errs:
        for e in errs:
            print(f"{'/'.join(str(p) for p in e.path) or '<root>'}: {e.message}")
        sys.exit(2)
    print("jsonschema")
except ImportError:
    props = schema["properties"]; required = schema["required"]
    problems = []
    for k in required:
        if k not in doc: problems.append(f"missing required {k}")
    for k in doc:
        if k not in props: problems.append(f"unexpected key {k}")
    tmap = {"string": str, "integer": int, "number": (int, float), "boolean": bool, "array": list, "object": dict, "null": type(None)}
    for k, v in doc.items():
        p = props.get(k, {})
        if "const" in p and v != p["const"]: problems.append(f"{k}: expected const {p['const']!r}")
        if "enum" in p and v not in p["enum"]: problems.append(f"{k}: {v!r} not in enum")
        t = p.get("type")
        if t:
            ts = t if isinstance(t, list) else [t]
            if not any(isinstance(v, tmap[x]) and not (x in ("integer","number") and isinstance(v, bool)) for x in ts):
                problems.append(f"{k}: type {type(v).__name__} not in {ts}")
    if problems:
        print("\n".join(problems)); sys.exit(2)
    print("structural")
